use super::{
    AawHost, HostRequests, ParameterInfo,
    control::{ClapControlThread, InstanceId, ReconfigureResult},
};
use crate::{
    audio::AudioBlock,
    midi::{MidiEvent, MidiEventKind},
};
use anyhow::anyhow;
use clack_extensions::params::PluginParams;
use clack_host::{
    events::{
        Pckn,
        event_types::{MidiEvent as ClapMidiEvent, ParamValueEvent},
        io::{EventBuffer, InputEvents},
    },
    process::{
        PluginAudioConfiguration, PluginAudioProcessor,
        audio_buffers::{AudioPortBuffer, AudioPortBufferType, AudioPorts, InputAudioBuffers},
    },
};
use std::{
    path::Path,
    sync::{Arc, atomic::Ordering},
};

const MIDI_EVENT_CAPACITY: usize = 256;
const PARAMETER_EVENT_CAPACITY: usize = 64;

pub struct Synth {
    control: ClapControlThread,
    instance_id: InstanceId,
    processor: Option<PluginAudioProcessor<AawHost>>,
    channel_count: usize,
    sample_rate: u32,
    max_block_size: usize,
    output_ports: AudioPorts,
    midi_events: Vec<ClapMidiEvent>,
    parameter_events: Vec<ParamValueEvent>,
    output_events: EventBuffer,
    parameter_changes: Vec<(u32, f64)>,
    steady_time: u64,
    parameters: Vec<ParameterInfo>,
    plugin_name: Option<String>,
    plugin_version: Option<String>,
    requests: Arc<HostRequests>,
}

impl Synth {
    pub async fn load(
        sample_rate: f64,
        path: impl AsRef<Path>,
        plugin_id: String,
    ) -> crate::synths::Result<Self> {
        let max_block_size = crate::MAX_BLOCK_SIZE;
        let control = ClapControlThread::shared()?;

        let config = PluginAudioConfiguration {
            sample_rate,
            min_frames_count: 1,
            max_frames_count: max_block_size.try_into().map_err(anyhow::Error::from)?,
        };

        let loaded = control.load(path.as_ref().to_path_buf(), plugin_id, config).await?;

        Ok(Self {
            control,
            instance_id: loaded.id,
            processor: Some(loaded.processor.into()),
            channel_count: 2,
            sample_rate: sample_rate as u32,
            max_block_size,
            output_ports: AudioPorts::with_capacity(2, 1),
            midi_events: Vec::with_capacity(MIDI_EVENT_CAPACITY),
            parameter_events: Vec::with_capacity(PARAMETER_EVENT_CAPACITY),
            output_events: EventBuffer::with_capacity(PARAMETER_EVENT_CAPACITY),
            parameter_changes: Vec::with_capacity(PARAMETER_EVENT_CAPACITY),
            steady_time: 0,
            parameters: loaded.parameters,
            plugin_name: loaded.name,
            plugin_version: loaded.version,
            requests: loaded.requests,
        })
    }

    pub fn take_parameters(&mut self) -> Vec<ParameterInfo> {
        std::mem::take(&mut self.parameters)
    }

    pub fn plugin_name(&self) -> Option<&str> {
        self.plugin_name.as_deref()
    }

    pub fn plugin_version(&self) -> Option<&str> {
        self.plugin_version.as_deref()
    }

    pub async fn reconfigure(
        &mut self,
        channel_count: usize,
        sample_rate: u32,
        max_block_size: usize,
    ) -> crate::synths::Result<()> {
        if max_block_size == 0 {
            return Err(anyhow!("Block size must be nonzero").into());
        }

        let config = PluginAudioConfiguration {
            sample_rate: sample_rate as f64,
            min_frames_count: 1,
            max_frames_count: max_block_size.try_into().map_err(anyhow::Error::from)?,
        };

        let processor =
            self.processor.take().ok_or_else(|| anyhow!("CLAP processor unavailable"))?;
        let stopped = processor.into_stopped();
        let result = self.control.reconfigure(self.instance_id, stopped, config).await;

        match result {
            ReconfigureResult::Ready(processor) => {
                self.processor = Some(processor.into());
                self.channel_count = channel_count;
                self.sample_rate = sample_rate;
                self.max_block_size = max_block_size;
                self.output_ports = AudioPorts::with_capacity(channel_count, 1);
                Ok(())
            }
            ReconfigureResult::Rejected { error, processor } => {
                self.processor = Some(processor.into());
                Err(error.into())
            }
            ReconfigureResult::ActivationFailed(error) => Err(error.into()),
        }
    }

    pub fn start_processing(&mut self) -> crate::synths::Result<()> {
        let processor =
            self.processor.as_mut().ok_or_else(|| anyhow!("CLAP processor unavailable"))?;

        processor.ensure_processing_started().map_err(anyhow::Error::from)?;

        Ok(())
    }

    pub fn stop_processing(&mut self) {
        if let Some(processor) = self.processor.as_mut() {
            processor.ensure_processing_stopped();
        }
    }

    pub fn silence(&mut self) {
        if let Some(processor) = self.processor.as_mut() {
            processor.reset();
        }
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) {
        if self.midi_events.len() == self.midi_events.capacity() {
            return;
        }

        let channel = event.channel.min(15);
        let data = match event.kind {
            MidiEventKind::NoteOn { key, velocity } => [0x90 | channel, key, velocity],
            MidiEventKind::NoteOff { key, velocity } => [0x80 | channel, key, velocity],
            MidiEventKind::ControlChange { controller, value } => {
                [0xb0 | channel, controller, value]
            }
            MidiEventKind::PitchBend { value } => {
                let value = (value.clamp(-8192, 8191) + 8192) as u16;
                [0xe0 | channel, (value & 0x7f) as u8, (value >> 7) as u8]
            }
            MidiEventKind::ProgramChange { program } => [0xc0 | channel, program, 0],
            MidiEventKind::ChannelAftertouch { value } => [0xd0 | channel, value, 0],
            MidiEventKind::PolyphonicAftertouch { key, value } => [0xa0 | channel, key, value],
        };

        self.midi_events.push(ClapMidiEvent::new(0, 0, data));
    }

    pub fn set_parameter(&mut self, parameter_id: u32, value: f64) -> bool {
        let Some(parameter_id) = clack_host::utils::ClapId::from_raw(parameter_id) else {
            return false;
        };
        if !value.is_finite() || self.parameter_events.len() == self.parameter_events.capacity() {
            return false;
        }

        self.parameter_events.push(ParamValueEvent::new(0, parameter_id, Pckn::match_all(), value));
        true
    }

    pub fn parameter_changes(&mut self) -> impl Iterator<Item = (u32, f64)> + '_ {
        self.parameter_changes.drain(..)
    }

    pub fn parameter_flush_requested(&self) -> bool {
        self.requests.parameter_flush.swap(false, Ordering::AcqRel)
    }

    pub fn flush_parameters(&mut self) {
        if !self.parameter_flush_requested() && self.parameter_events.is_empty() {
            return;
        }

        let Some(processor) = self.processor.as_mut() else {
            return;
        };
        let mut plugin = processor.plugin_handle();
        let Some(parameters) = plugin.get_extension::<PluginParams>() else {
            self.parameter_events.clear();
            return;
        };

        self.output_events.clear();
        let input_events = InputEvents::from_buffer(&self.parameter_events);
        let mut output_events = self.output_events.as_output();
        parameters.flush_active(&mut plugin, &input_events, &mut output_events);
        self.parameter_events.clear();
        self.collect_parameter_changes();
    }

    fn collect_parameter_changes(&mut self) {
        for event in self.output_events.iter() {
            let Some(event) = event.as_event::<ParamValueEvent>() else {
                continue;
            };
            let Some(parameter_id) = event.param_id() else {
                continue;
            };
            if let Some((_, value)) =
                self.parameter_changes.iter_mut().find(|(id, _)| *id == parameter_id.get())
            {
                *value = event.value();
                continue;
            }
            if self.parameter_changes.len() == self.parameter_changes.capacity() {
                break;
            }

            self.parameter_changes.push((parameter_id.get(), event.value()));
        }
    }

    pub fn render_audio_block(&mut self, output: &mut AudioBlock) -> crate::synths::Result<()> {
        let sample_count = output.sample_count_per_channel();
        if sample_count > self.max_block_size {
            return Err(anyhow!("CLAP audio block exceeds configured maximum").into());
        }

        let processor =
            self.processor.as_mut().ok_or_else(|| anyhow!("CLAP processor unavailable"))?;
        let processor = processor.ensure_processing_started().map_err(anyhow::Error::from)?;
        let input_audio = InputAudioBuffers::empty();
        let input_event_buffers = (&self.midi_events, &self.parameter_events);
        let input_events = InputEvents::from_buffer(&input_event_buffers);
        self.output_events.clear();
        let mut output_events = self.output_events.as_output();
        let mut output_audio = self.output_ports.with_output_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_output_only(
                output.samples_mut().iter_mut().map(|channel| &mut channel[..sample_count]),
            ),
        }]);

        let result = processor
            .process(
                &input_audio,
                &mut output_audio,
                &input_events,
                &mut output_events,
                Some(self.steady_time),
                None,
            )
            .map_err(anyhow::Error::from);
        self.midi_events.clear();
        self.parameter_events.clear();
        result?;

        self.collect_parameter_changes();
        self.steady_time = self.steady_time.wrapping_add(sample_count as u64);
        Ok(())
    }
}

impl Drop for Synth {
    fn drop(&mut self) {
        let processor = self.processor.take().map(PluginAudioProcessor::into_stopped);
        self.control.destroy(self.instance_id, processor);
    }
}
