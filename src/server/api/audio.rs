use crate::{DEFAULT_MAX_BLOCK_SIZE, audio};
use crate::{
    app_state::AppState,
    audio::params::{DeviceCfgSelection, LatencySelection, SampleRateSelection},
};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(audio_host_outputs))
        .routes(routes!(audio_output_stream))
        .routes(routes!(set_audio_output_stream))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioHostOutputs {
    pub host: AudioHost,
    pub outputs: Vec<AudioOutput>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioHost {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AudioOutput {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub struct AudioOutputStream {
    pub stream: cpal::Stream,
    pub info: AudioOutputStreamInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AudioOutputStreamInfo {
    pub host_id: String,
    pub output_id: String,
    pub channel_count: u32,
    pub sample_rate: u32,
    pub max_block_size: usize,
    pub sample_format: String,

    #[schema(required)]
    pub buffer_size: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct AudioOutputStreamConfig {
    pub channel_count: usize,
    pub sample_rate: u32,
    pub max_block_size: usize,
}

impl From<&AudioOutputStreamInfo> for AudioOutputStreamConfig {
    fn from(value: &AudioOutputStreamInfo) -> Self {
        Self {
            channel_count: value.channel_count as usize,
            sample_rate: value.sample_rate,
            max_block_size: value.max_block_size,
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/outputs",
    responses(
        (
            status = OK,
            body = Vec<AudioHostOutputs>,
            content_type = "aplication/json",
            description = "OK"
        ),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn audio_host_outputs() -> Result<Json<Vec<AudioHostOutputs>>, (StatusCode, String)> {
    let outputs = crate::audio::info::list_hosts()
        .iter()
        .filter_map(|h| {
            let Ok(outputs) = crate::audio::info::list_output_devices(&h.id) else {
                return None;
            };

            Some(AudioHostOutputs {
                host: AudioHost { id: h.id.clone(), name: h.name.into() },
                outputs: outputs
                    .into_iter()
                    .map(|o| AudioOutput {
                        id: o.id.clone(),
                        name: o.name.clone(),
                        is_default: o.is_default,
                    })
                    .collect(),
            })
        })
        .collect();

    Ok(Json(outputs))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/output/stream",
    responses(
        (status = OK, body = Option<AudioOutputStreamInfo>, description = "OK"),
    )
)]
async fn audio_output_stream(
    State(app_state): State<AppState>,
) -> Json<Option<AudioOutputStreamInfo>> {
    Json(app_state.output_stream.lock().await.as_ref().map(|s| s.info.clone()))
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, ToSchema)]
pub struct SetOutputStreamRequest {
    host_id: String,
    output_id: String,
    latency: OutputStreamLatency,
    sample_rate: OutputStreamSampleRate,
}

#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum OutputStreamLatency {
    Automatic,
    Low,
    Stable,
    CustomBufferSize { value: u32 },
}

impl From<OutputStreamLatency> for LatencySelection {
    fn from(value: OutputStreamLatency) -> Self {
        match value {
            OutputStreamLatency::Automatic => Self::Automatic,
            OutputStreamLatency::Low => Self::Low,
            OutputStreamLatency::Stable => Self::Stable,
            OutputStreamLatency::CustomBufferSize { value } => Self::CustomBufferSize(value),
        }
    }
}

#[derive(Clone, Copy, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum OutputStreamSampleRate {
    Automatic,
    Fixed { value: u32 },
}

impl From<OutputStreamSampleRate> for SampleRateSelection {
    fn from(value: OutputStreamSampleRate) -> Self {
        match value {
            OutputStreamSampleRate::Automatic => Self::Automatic,
            OutputStreamSampleRate::Fixed { value } => Self::Fixed(value),
        }
    }
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/output/stream",
    request_body = Option<SetOutputStreamRequest>,
    responses(
        (status = OK, body = Option<AudioOutputStreamInfo>, description = "OK"),
        (status = BAD_REQUEST, body = String, description = "Bad Request"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_audio_output_stream(
    State(app_state): State<AppState>,
    Json(req): Json<Option<SetOutputStreamRequest>>,
) -> Result<Json<Option<AudioOutputStreamInfo>>, (StatusCode, String)> {
    let Some(req) = req else {
        if let Some(stream) = app_state.output_stream.lock().await.take() {
            drop(stream);
        }
        return Ok(Json(None));
    };

    let device_config = audio::output::ResolvedConfig::find(DeviceCfgSelection {
        host: audio::params::HostSelection::Id(req.host_id.as_str()),
        device: audio::params::DeviceSelection::Id(req.output_id.as_str()),
        latency: req.latency.into(),
        sample_rate: req.sample_rate.into(),
    });

    let resolved_config = device_config.map_err(map_audio_error)?;
    let sample_format = resolved_config.sample_format().to_string();
    let stream_cfg = resolved_config.config();

    let stream_info = AudioOutputStreamInfo {
        host_id: req.host_id,
        output_id: req.output_id,
        buffer_size: match stream_cfg.buffer_size {
            cpal::BufferSize::Default => None,
            cpal::BufferSize::Fixed(size) => Some(size),
        },
        channel_count: stream_cfg.channels as u32,
        sample_rate: stream_cfg.sample_rate,
        max_block_size: DEFAULT_MAX_BLOCK_SIZE,
        sample_format,
    };

    if let Some(stream) = app_state.output_stream.lock().await.take() {
        drop(stream);
    }

    let stream = resolved_config
        .start_stream(Arc::clone(&app_state.engine), Arc::clone(&app_state.stream_error))
        .await
        .map_err(map_audio_error)?;

    *app_state.output_stream.lock().await =
        Some(AudioOutputStream { stream, info: stream_info.clone() });

    Ok(Json(Some(stream_info)))
}

//--------------------------------------------------------------------------------------------------

fn map_audio_error(err: audio::output::Error) -> (StatusCode, String) {
    use audio::output::Error;
    let code = match &err {
        Error::HostNotFound => StatusCode::BAD_REQUEST,
        Error::HostHasNoDefaultDevice => StatusCode::INTERNAL_SERVER_ERROR,
        Error::DeviceNotFound => StatusCode::BAD_REQUEST,
        Error::FailedToListDevices(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Error::FailedToListDeviceConfigs(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Error::NoSupportedDeviceConfig => StatusCode::INTERNAL_SERVER_ERROR,
        Error::UnsupportedSampleFormat(_) => StatusCode::BAD_REQUEST,
        Error::FailedToBuildStream(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Error::FailedToStartStream(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Error::EngineReconfiguration(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, err.to_string())
}
