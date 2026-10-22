use crate::{
    DEFAULT_MAX_BLOCK_SIZE,
    audio::{
        self,
        params::{
            DeviceCfgSelection, DeviceSelection, HostSelection, LatencySelection,
            SampleRateSelection,
        },
    },
    engine::{Engine, synth_graph::SynthGraphError},
};
use cpal::{
    BufferSize,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("audio host not found")]
    HostNotFound,

    #[error("audio host has no default device")]
    HostHasNoDefaultDevice,

    #[error("audio output device not found")]
    DeviceNotFound,

    #[error("failed to list audio output devices: {0}")]
    FailedToListDevices(#[source] cpal::Error),

    #[error("failed to list audio output device configs: {0}")]
    FailedToListDeviceConfigs(#[source] cpal::Error),

    #[error("no supported audio output device config matches the requested settings")]
    NoSupportedDeviceConfig,

    #[error("unsupported sample format: {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),

    #[error("failed to build audio output stream")]
    FailedToBuildStream(#[source] cpal::Error),

    #[error("failed to start audio output stream")]
    FailedToStartStream(#[source] cpal::Error),

    #[error("failed to init engine: {0}")]
    EngineReconfiguration(#[source] SynthGraphError),
}

type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, Error>;

pub struct ResolvedConfig {
    device: cpal::Device,
    sample_format: cpal::SampleFormat,
    config: cpal::StreamConfig,
}

impl ResolvedConfig {
    pub fn find(selection: DeviceCfgSelection) -> Result<Self> {
        resolve_device_config(selection)
    }

    pub fn device(&self) -> &cpal::Device {
        &self.device
    }

    pub fn sample_format(&self) -> cpal::SampleFormat {
        self.sample_format
    }

    pub fn config(&self) -> cpal::StreamConfig {
        self.config
    }

    pub async fn start_stream(
        self,
        engine: Arc<Mutex<Engine>>,
        stream_error: Arc<AtomicBool>,
    ) -> Result<cpal::Stream> {
        use cpal::SampleFormat as Fmt;

        let device = &self.device;
        let config = self.config;
        let stream_error2 = Arc::clone(&stream_error);

        let stream = match self.sample_format {
            Fmt::I8 => build_stream::<i8>(device, config, engine, stream_error).await,
            Fmt::F32 => build_stream::<f32>(device, config, engine, stream_error).await,
            Fmt::I16 => build_stream::<i16>(device, config, engine, stream_error).await,
            Fmt::I24 => build_stream::<cpal::I24>(device, config, engine, stream_error).await,
            Fmt::I32 => build_stream::<i32>(device, config, engine, stream_error).await,
            Fmt::I64 => build_stream::<i64>(device, config, engine, stream_error).await,
            Fmt::U8 => build_stream::<u8>(device, config, engine, stream_error).await,
            Fmt::U16 => build_stream::<u16>(device, config, engine, stream_error).await,
            Fmt::U24 => build_stream::<cpal::U24>(device, config, engine, stream_error).await,
            Fmt::U32 => build_stream::<u32>(device, config, engine, stream_error).await,
            Fmt::U64 => build_stream::<u64>(device, config, engine, stream_error).await,
            Fmt::F64 => build_stream::<f64>(device, config, engine, stream_error).await,
            sample_format => Err(Error::UnsupportedSampleFormat(sample_format)),
        }?;

        stream.play().map_err(Error::FailedToStartStream)?;
        stream_error2.store(false, std::sync::atomic::Ordering::Relaxed);

        Ok(stream)
    }
}

async fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    engine: Arc<Mutex<Engine>>,
    stream_error: Arc<AtomicBool>,
) -> Result<cpal::Stream>
where
    T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
{
    let channel_count = config.channels as usize;
    engine
        .lock()
        .await
        .reconfigure(channel_count, config.sample_rate, DEFAULT_MAX_BLOCK_SIZE)
        .map_err(Error::EngineReconfiguration)?;

    let mut scratch = vec![0.0_f32; channel_count * DEFAULT_MAX_BLOCK_SIZE];

    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [T], _| {
                if scratch.len() < output.len() {
                    output.fill(T::from_sample(0.0));
                    return;
                }

                let scratch = &mut scratch[..output.len()];
                engine.blocking_lock().run(scratch);

                for (dst, src) in output.iter_mut().zip(scratch.iter().copied()) {
                    *dst = T::from_sample(src);
                }
            },
            move |_| {
                stream_error.store(true, std::sync::atomic::Ordering::Relaxed);
            },
            None,
        )
        .map_err(Error::FailedToBuildStream)?;

    Ok(stream)
}

fn resolve_device_config(selection: DeviceCfgSelection) -> Result<ResolvedConfig> {
    let host = match &selection.host {
        HostSelection::Default => cpal::default_host(),
        HostSelection::Id(id) => audio::info::host_by_id(id).map_err(|_| Error::HostNotFound)?,
    };
    let device = match &selection.device {
        DeviceSelection::Default => {
            host.default_output_device().ok_or(Error::HostHasNoDefaultDevice)?
        }
        DeviceSelection::Id(id) => find_device_by_id(&host, id)?,
    };
    let (sample_format, config) =
        resolve_preferred_config(&device, &selection.latency, &selection.sample_rate)?;
    Ok(ResolvedConfig { device, sample_format, config })
}

fn find_device_by_id(host: &cpal::Host, id: &str) -> Result<cpal::Device> {
    for device in host.output_devices().map_err(Error::FailedToListDevices)? {
        if let Ok(dev_id) = device.id()
            && dev_id.id() == id
        {
            return Ok(device);
        }
    }
    Err(Error::DeviceNotFound)
}

fn resolve_preferred_config(
    device: &cpal::Device,
    latency_sel: &LatencySelection,
    sample_rate_sel: &SampleRateSelection,
) -> Result<(cpal::SampleFormat, cpal::StreamConfig)> {
    let bufs = buffer_sizes(latency_sel);
    let ranges = list_config_ranges(device)?;

    bufs.iter()
        .find_map(|&buf| {
            ranges
                .iter()
                .cloned()
                .filter(|r| config_matches_sample_rate(r, sample_rate_sel))
                .filter(|r| config_matches_buffer_size(r, buf))
                .max_by(best_config_order)
                .map(|r| resolved_config(r, buf, sample_rate_sel))
        })
        .ok_or(Error::NoSupportedDeviceConfig)
}

fn list_config_ranges(device: &cpal::Device) -> Result<Vec<cpal::SupportedStreamConfigRange>> {
    let configs = device.supported_output_configs().map_err(Error::FailedToListDeviceConfigs)?;
    Ok(configs.collect::<Vec<_>>())
}

fn buffer_sizes(latency_sel: &LatencySelection) -> Vec<BufferSize> {
    match latency_sel {
        LatencySelection::Automatic => vec![
            BufferSize::Default,
            BufferSize::Fixed(256),
            BufferSize::Fixed(512),
            BufferSize::Fixed(1024),
        ],
        LatencySelection::Low => {
            vec![BufferSize::Fixed(64), BufferSize::Fixed(128), BufferSize::Fixed(256)]
        }
        LatencySelection::Stable => {
            vec![BufferSize::Fixed(256), BufferSize::Fixed(512), BufferSize::Fixed(1024)]
        }
        LatencySelection::CustomBufferSize(size) => vec![BufferSize::Fixed(*size)],
    }
}

fn config_matches_sample_rate(
    range: &cpal::SupportedStreamConfigRange,
    sample_rate_sel: &SampleRateSelection,
) -> bool {
    match sample_rate_sel {
        SampleRateSelection::Automatic => true,
        SampleRateSelection::Fixed(sr) => range.contains_rate(*sr),
    }
}

fn config_matches_buffer_size(
    range: &cpal::SupportedStreamConfigRange,
    buffer_size: cpal::BufferSize,
) -> bool {
    use cpal::{BufferSize, SupportedBufferSize as Buf};

    match (buffer_size, range.buffer_size()) {
        (BufferSize::Default, _) | (_, Buf::Unknown) => true,
        (BufferSize::Fixed(n), Buf::Range { min, max }) => *min <= n && n <= *max,
    }
}

fn best_config_order(
    a: &cpal::SupportedStreamConfigRange,
    b: &cpal::SupportedStreamConfigRange,
) -> std::cmp::Ordering {
    sample_format_score(a.sample_format())
        .cmp(&sample_format_score(b.sample_format()))
        .then_with(|| a.cmp_default_heuristics(b))
}

fn resolved_config(
    range: cpal::SupportedStreamConfigRange,
    buffer_size: cpal::BufferSize,
    sample_rate_sel: &SampleRateSelection,
) -> (cpal::SampleFormat, cpal::StreamConfig) {
    let supported = with_selected_sample_rate(range, sample_rate_sel);
    let mut stream = supported.config();
    stream.buffer_size = buffer_size;
    (supported.sample_format(), stream)
}

fn with_selected_sample_rate(
    range: cpal::SupportedStreamConfigRange,
    sample_rate_sel: &SampleRateSelection,
) -> cpal::SupportedStreamConfig {
    match sample_rate_sel {
        SampleRateSelection::Automatic => {
            range.try_with_standard_sample_rate().unwrap_or_else(|| range.with_max_sample_rate())
        }
        SampleRateSelection::Fixed(sr) => range.with_sample_rate(*sr),
    }
}

fn sample_format_score(format: cpal::SampleFormat) -> u8 {
    use cpal::SampleFormat::*;

    match format {
        F32 => 6,
        F64 => 5,
        I16 => 4,
        U16 => 3,
        I24 | I32 => 2,
        U24 | U32 => 1,
        _ => 0,
    }
}
