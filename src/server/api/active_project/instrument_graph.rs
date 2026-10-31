use crate::{
    app_state::AppState,
    engine::{
        self,
        controller_graph::{self, RtInstrumentControllers},
        effects::{self, Effects, EffectsContext, RtEffects},
        graph::InstrumentId,
        instrument_graph::{
            ClapInstrumentContext, CtrInstrument, CtrInstrumentKind, DummyInstrumentContext,
            FluidInstrumentContext, RtInstrument, RtInstrumentKind, SfizzInstrumentContext,
        },
    },
    memory::Siv,
    server::api::audio::AudioOutputStreamConfig,
    synths::{self, SoundFontProgram},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(set_global_effects))
        .routes(routes!(global_effects))
        .routes(routes!(set_global_gain))
        .routes(routes!(global_gain))
        .routes(routes!(instrument_list))
        .routes(routes!(instrument_push))
        .routes(routes!(instrument_replace))
        .routes(routes!(instrument_remove))
        .routes(routes!(instrument_kind))
        .routes(routes!(instrument_set_soundfont_program))
        .routes(routes!(instrument_set_clap_parameter))
        .routes(routes!(instrument_set_name))
        .routes(routes!(instrument_name))
        .routes(routes!(instrument_set_effects))
        .routes(routes!(instrument_effects))
        .routes(routes!(instrument_set_gain))
        .routes(routes!(instrument_gain))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
pub(super) struct InstrumentPath {
    #[param(value_type = usize)]
    pub instrument_id: InstrumentId,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
struct ClapParameterPath {
    #[param(value_type = usize)]
    instrument_id: InstrumentId,

    parameter_id: u32,
}

#[derive(Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
enum InstrumentPushRequest {
    Dummy {
        max_voices: usize,
    },
    Fluid {
        #[schema(value_type = String)]
        path: PathBuf,

        max_voices: usize,
    },
    Sfizz {
        #[schema(value_type = String)]
        path: PathBuf,

        max_voices: usize,
        preload_size: u32,
    },
    Clap {
        #[schema(value_type = String)]
        path: PathBuf,

        plugin_id: String,
    },
}

#[utoipa::path(
    put,
    path = "/global-effects",
    request_body(content = Effects, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Global effects set"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn set_global_effects(
    State(state): State<AppState>,
    Json(effects): Json<Effects>,
) -> Result<StatusCode, (StatusCode, String)> {
    let config = state.stream_config().await;
    let rt = RtEffects::new(&effects, config.channel_count, config.sample_rate)?;
    state.engine_handle.set_global_effects(effects, rt).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/global-effects",
    responses((status = OK, body = Effects, content_type = "application/json", description = "Global effects"))
)]
async fn global_effects(State(state): State<AppState>) -> Json<Effects> {
    Json(state.engine_handle.project().await.instrument_graph.global_effects.clone())
}

#[utoipa::path(
    put,
    path = "/global-gain",
    request_body(content = f32, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Global gain set"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn set_global_gain(
    State(state): State<AppState>,
    Json(gain): Json<f32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.engine_handle.set_global_gain(gain).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/global-gain",
    responses((status = OK, body = f32, content_type = "application/json", description = "Global linear gain"))
)]
async fn global_gain(State(state): State<AppState>) -> Json<f32> {
    Json(state.engine_handle.project().await.instrument_graph.global_gain)
}

#[utoipa::path(
    get,
    path = "/instruments",
    responses((status = OK, body = HashMap<InstrumentId, CtrInstrument>, content_type = "application/json", description = "Instruments"))
)]
async fn instrument_list(
    State(state): State<AppState>,
) -> Json<HashMap<InstrumentId, CtrInstrument>> {
    Json(state.engine_handle.project().await.instrument_graph.instruments.clone().into())
}

#[utoipa::path(
    post,
    path = "/instruments",
    request_body(content = InstrumentPushRequest, content_type = "application/json"),
    responses((status = OK, body = InstrumentId, description = "Created instrument ID"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_push(
    State(state): State<AppState>,
    Json(request): Json<InstrumentPushRequest>,
) -> Result<Json<InstrumentId>, (StatusCode, String)> {
    let config = state.stream_config().await;
    let (name, kind, rt_kind) =
        create_instrument(&state.assets.read().await.dir, &config, request).await?;
    let (effects, rt_effects) = effects::default(EffectsContext {
        channel_count: config.channel_count,
        sample_rate: config.sample_rate,
    })?;
    let (controller, rt_controller) = controller_graph::default_controller();
    let mut controllers = Siv::with_capacity(state.min_controllers.max(1));
    let mut rt_controllers = Siv::with_capacity(state.min_controllers.max(1));
    controllers.push(controller);
    rt_controllers.push(rt_controller);

    let id = state
        .engine_handle
        .instrument_push(
            CtrInstrument {
                name,
                kind,
                controllers,
                effects,
                gain: 1.0,
                last_controller_values: Box::new([None; 128]),
            },
            RtInstrument { kind: rt_kind, effects: rt_effects, gain: 1.0 },
            RtInstrumentControllers { controllers: rt_controllers },
        )
        .await?;
    Ok(Json(id))
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/kind",
    params(InstrumentPath),
    request_body(content = InstrumentPushRequest, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Instrument renderer replaced"), (status = NOT_FOUND, body = String, description = "Instrument not found"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_replace(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(request): Json<InstrumentPushRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&state, path.instrument_id, |_| ()).await?;
    let config = state.stream_config().await;
    let (_, kind, rt_kind) =
        create_instrument(&state.assets.read().await.dir, &config, request).await?;
    state.engine_handle.instrument_replace(path.instrument_id, kind, rt_kind).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/instruments/{instrument_id}",
    params(InstrumentPath),
    responses((status = NO_CONTENT, description = "Instrument removed"), (status = NOT_FOUND, body = String, description = "Instrument not found"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_remove(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&state, path.instrument_id, |_| ()).await?;
    state.engine_handle.instrument_remove(path.instrument_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/kind",
    params(InstrumentPath),
    responses((status = OK, body = CtrInstrumentKind, content_type = "application/json", description = "Instrument renderer"), (status = NOT_FOUND, body = String, description = "Instrument not found"))
)]
async fn instrument_kind(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<CtrInstrumentKind>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&state, path.instrument_id, |instrument| instrument.kind.clone()).await?,
    ))
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/soundfont-program",
    params(InstrumentPath),
    request_body(content = SoundFontProgram, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Program selected"), (status = NOT_FOUND, body = String, description = "Instrument not found"), (status = BAD_REQUEST, body = String, description = "Unsupported instrument kind"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_set_soundfont_program(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(program): Json<SoundFontProgram>,
) -> Result<StatusCode, (StatusCode, String)> {
    let supported = with_instrument(&state, path.instrument_id, |instrument| {
        matches!(instrument.kind, CtrInstrumentKind::Fluid { .. })
    })
    .await?;
    if !supported {
        return Err((StatusCode::BAD_REQUEST, "not supported".into()));
    }
    state.engine_handle.instrument_select_soundfont_program(path.instrument_id, program).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/clap-parameters/{parameter_id}",
    params(ClapParameterPath),
    request_body(content = f64, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Parameter set"), (status = BAD_REQUEST, body = String, description = "Invalid parameter"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_set_clap_parameter(
    State(state): State<AppState>,
    Path(path): Path<ClapParameterPath>,
    Json(value): Json<f64>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .engine_handle
        .instrument_set_clap_parameter(path.instrument_id, path.parameter_id, value)
        .await
        .map_err(|error| match error {
            engine::EngineError::Instrument(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            error => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/name",
    params(InstrumentPath),
    request_body(content = String, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Instrument name set"), (status = NOT_FOUND, body = String, description = "Instrument not found"))
)]
async fn instrument_set_name(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&state, path.instrument_id, |_| ()).await?;
    state.engine_handle.instrument_set_name(path.instrument_id, name).await;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/name",
    params(InstrumentPath),
    responses((status = OK, body = String, content_type = "application/json", description = "Instrument name"), (status = NOT_FOUND, body = String, description = "Instrument not found"))
)]
async fn instrument_name(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&state, path.instrument_id, |instrument| instrument.name.clone()).await?,
    ))
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/effects",
    params(InstrumentPath),
    request_body(content = Effects, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Instrument effects set"), (status = NOT_FOUND, body = String, description = "Instrument not found"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_set_effects(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(effects): Json<Effects>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&state, path.instrument_id, |_| ()).await?;
    let config = state.stream_config().await;
    let rt = RtEffects::new(&effects, config.channel_count, config.sample_rate)?;
    state.engine_handle.instrument_set_effects(path.instrument_id, effects, rt).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/effects",
    params(InstrumentPath),
    responses((status = OK, body = Effects, content_type = "application/json", description = "Instrument effects"), (status = NOT_FOUND, body = String, description = "Instrument not found"))
)]
async fn instrument_effects(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<Effects>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&state, path.instrument_id, |instrument| instrument.effects.clone())
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/instruments/{instrument_id}/gain",
    params(InstrumentPath),
    request_body(content = f32, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Instrument gain set"), (status = NOT_FOUND, body = String, description = "Instrument not found"), (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal server error"))
)]
async fn instrument_set_gain(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(gain): Json<f32>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&state, path.instrument_id, |_| ()).await?;
    state.engine_handle.instrument_set_gain(path.instrument_id, gain).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/instruments/{instrument_id}/gain",
    params(InstrumentPath),
    responses((status = OK, body = f32, content_type = "application/json", description = "Instrument linear gain"), (status = NOT_FOUND, body = String, description = "Instrument not found"))
)]
async fn instrument_gain(
    State(state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<f32>, (StatusCode, String)> {
    Ok(Json(with_instrument(&state, path.instrument_id, |instrument| instrument.gain).await?))
}

pub(super) async fn with_instrument<T>(
    state: &AppState,
    id: InstrumentId,
    function: impl FnOnce(&CtrInstrument) -> T,
) -> Result<T, (StatusCode, String)> {
    state
        .engine_handle
        .project()
        .await
        .instrument_graph
        .instruments
        .get(id)
        .map(function)
        .ok_or((StatusCode::NOT_FOUND, "instrument not found".into()))
}

async fn create_instrument(
    assets_dir: &std::path::Path,
    config: &AudioOutputStreamConfig,
    request: InstrumentPushRequest,
) -> Result<(String, CtrInstrumentKind, RtInstrumentKind), synths::Error> {
    match request {
        InstrumentPushRequest::Dummy { max_voices } => {
            let (ctr, rt) = engine::instrument_graph::prepare_dummy_instrument(
                max_voices,
                DummyInstrumentContext { sample_rate: config.sample_rate as f32 },
            );
            Ok(("Dummy Instrument".into(), ctr, rt))
        }
        InstrumentPushRequest::Fluid { path, max_voices } => {
            let name = file_name(&path, "Fluid Instrument");
            let (ctr, rt) = engine::instrument_graph::prepare_fluid_instrument(
                path,
                None,
                max_voices,
                FluidInstrumentContext { assets_dir, sample_rate: config.sample_rate as f64 },
            )
            .await?;
            Ok((name, ctr, rt))
        }
        InstrumentPushRequest::Sfizz { path, max_voices, preload_size } => {
            let name = file_name(&path, "Sfizz Instrument");
            let (ctr, rt) = engine::instrument_graph::prepare_sfizz_instrument(
                path,
                max_voices,
                preload_size,
                SfizzInstrumentContext {
                    assets_dir,
                    sample_rate: config.sample_rate as f32,
                    max_block_size: config.max_block_size,
                },
            )
            .await?;
            Ok((name, ctr, rt))
        }
        InstrumentPushRequest::Clap { path, plugin_id } => {
            let (ctr, rt) = engine::instrument_graph::prepare_clap_instrument(
                path,
                plugin_id,
                ClapInstrumentContext { assets_dir, sample_rate: config.sample_rate as f64 },
            )
            .await?;
            let name = match &ctr {
                CtrInstrumentKind::Clap { plugin_name, .. } => {
                    plugin_name.clone().unwrap_or_default()
                }
                _ => unreachable!(),
            };
            Ok((name, ctr, rt))
        }
    }
}

fn file_name(path: &std::path::Path, fallback: &str) -> String {
    path.file_stem()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.into())
}

impl From<synths::Error> for (StatusCode, String) {
    fn from(value: synths::Error) -> Self {
        (StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}

impl From<effects::Error> for (StatusCode, String) {
    fn from(value: effects::Error) -> Self {
        (StatusCode::INTERNAL_SERVER_ERROR, value.to_string())
    }
}
