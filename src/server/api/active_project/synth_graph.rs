use crate::{
    app_state::AppState,
    engine::{
        self,
        effects::{self, Effects, EffectsContext, RtEffects},
        graph::SynthId,
        synth_graph::{
            CtrSynth, CtrSynthParameters, FluidSynthContext, RtSynth, RtSynthKind,
            SfizzSynthContext,
        },
    },
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
        .routes(routes!(set_effects))
        .routes(routes!(synth_list))
        .routes(routes!(synth_push))
        .routes(routes!(synth_replace))
        .routes(routes!(synth_remove))
        .routes(routes!(synth_parameters))
        .routes(routes!(synth_set_effects))
        .routes(routes!(synth_set_soundfont_program))
        .routes(routes!(synth_set_name))
        .routes(routes!(synth_name))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)] //TODO: remove all parameter_in
struct SynthPath {
    #[param(value_type = usize)]
    synth_id: SynthId,
}

#[derive(Deserialize, ToSchema)]
struct SynthPushRequest {
    #[serde(flatten)]
    params: SynthPushParams,
}

#[derive(Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
enum SynthPushParams {
    Dummy,
    Fluid {
        #[schema(value_type = String)]
        path: PathBuf,
    },
    Sfizz {
        #[schema(value_type = String)]
        path: PathBuf,
    },
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/effects",
    request_body(
        content = Effects,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_effects(
    State(app_state): State<AppState>,
    Json(effects): Json<Effects>,
) -> Result<StatusCode, (StatusCode, String)> {
    let stream_config = app_state.stream_config().await;

    let rt_effects =
        RtEffects::new(&effects, stream_config.channel_count, stream_config.sample_rate)?;

    app_state.engine_handle.set_effects(effects, rt_effects).await?;

    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/synths",
    responses(
        (
            status = OK,
            body = HashMap<SynthId, CtrSynth>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn synth_list(State(app_state): State<AppState>) -> Json<HashMap<SynthId, CtrSynth>> {
    Json(app_state.engine_handle.project().await.synth_graph.synths.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/synths",
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = OK, body = SynthId, content_type = "application/json", description = "Created"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_push(
    State(app_state): State<AppState>,
    Json(req): Json<SynthPushRequest>,
) -> Result<Json<SynthId>, (StatusCode, String)> {
    let stream_config = app_state.stream_config().await;

    let (name, parameters, kind) =
        create_synth(&app_state.assets.read().await.dir, stream_config, req.params).await?;

    let (effects, rt_effects) = effects::default(EffectsContext {
        channel_count: stream_config.channel_count,
        sample_rate: stream_config.sample_rate,
    })?;

    let ctr_synth = CtrSynth { name, parameters, effects };
    let rt_synth = RtSynth { kind, effects: rt_effects };

    Ok(Json(app_state.engine_handle.synth_push(ctr_synth, rt_synth).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/synths/{synth_id}",
    params(SynthPath),
    request_body(
        content = SynthPushRequest,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Created"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_replace(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(req): Json<SynthPushRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_synth(&app_state, path.synth_id, |_| ()).await?;
    let stream_config = app_state.stream_config().await;

    let (_, parameters, kind) =
        create_synth(&app_state.assets.read().await.dir, stream_config, req.params).await?;

    app_state.engine_handle.synth_replace(path.synth_id, parameters, kind).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/synths/{synth_id}",
    params(SynthPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_remove(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.synth_remove(path.synth_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/synths/{synth_id}/parameters",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = CtrSynthParameters,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn synth_parameters(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<CtrSynthParameters>, (StatusCode, String)> {
    Ok(Json(with_synth(&app_state, path.synth_id, |s| s.parameters.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/synths/{synth_id}/effects",
    params(SynthPath),
    request_body(
        content = Effects,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_set_effects(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(effects): Json<Effects>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_synth(&app_state, path.synth_id, |_| ()).await?;
    let stream_config = app_state.stream_config().await;

    let rt_effects =
        RtEffects::new(&effects, stream_config.channel_count, stream_config.sample_rate)?;

    app_state.engine_handle.synth_set_effects(path.synth_id, effects, rt_effects).await?;

    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/synths/{synth_id}/soundfont-program",
    params(SynthPath),
    request_body(
        content = SoundFontProgram,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = BAD_REQUEST, body = String, description = "Not supported by the synth"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_set_soundfont_program(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(program): Json<SoundFontProgram>,
) -> Result<StatusCode, (StatusCode, String)> {
    let project = app_state.engine_handle.project().await;
    let synth = project
        .synth_graph
        .synths
        .get(path.synth_id)
        .ok_or((StatusCode::NOT_FOUND, "not found".into()))?;

    match synth.parameters {
        CtrSynthParameters::Dummy => return Err((StatusCode::BAD_REQUEST, "not supported".into())),
        CtrSynthParameters::Fluid { .. } => {}
        CtrSynthParameters::Sfizz { .. } => {
            return Err((StatusCode::BAD_REQUEST, "not supported".into()));
        }
    }

    drop(project);

    app_state.engine_handle.synth_select_soundfont_program(path.synth_id, program).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/synths/{synth_id}/name",
    params(SynthPath),
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn synth_set_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_synth(&app_state, path.synth_id, |_| ()).await?;
    app_state.engine_handle.synth_set_name(path.synth_id, name).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/synths/{synth_id}/name",
    params(SynthPath),
    responses(
        (
            status = OK,
            body = String,
            content_type = "application/json",
            description = "Value set"
        ),
        (status = NOT_FOUND, body = String, description = "Synth not found"),
    )
)]
async fn synth_name(
    State(app_state): State<AppState>,
    Path(path): Path<SynthPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(with_synth(&app_state, path.synth_id, |s| s.name.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

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

async fn with_synth<T>(
    app_state: &AppState,
    id: SynthId,
    f: impl FnOnce(&CtrSynth) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project().await;
    let result_opt = project.synth_graph.synths.get(id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}

async fn create_synth(
    assets_dir: impl AsRef<std::path::Path>,
    stream_config: AudioOutputStreamConfig,
    params: SynthPushParams,
) -> Result<(String, CtrSynthParameters, RtSynthKind), synths::Error> {
    let (name, (parameters, kind)) = match params {
        SynthPushParams::Dummy => (
            "Dummy Synth".into(),
            engine::synth_graph::prepare_dummy_synth(stream_config.sample_rate),
        ),

        SynthPushParams::Fluid { path } => {
            let name = path
                .file_stem()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("Fluid Synth".into());
            let context = FluidSynthContext {
                assets_dir: assets_dir.as_ref(),
                sample_rate: stream_config.sample_rate as f64,
            };
            (name, engine::synth_graph::prepare_fluid_synth(path, context).await?)
        }
        SynthPushParams::Sfizz { path } => {
            let name = path
                .file_stem()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or("Sfizz Synth".into());
            let context = SfizzSynthContext {
                assets_dir: assets_dir.as_ref(),
                sample_rate: stream_config.sample_rate as f32,
                max_block_size: stream_config.max_block_size,
            };
            (name, engine::synth_graph::prepare_sfizz_synth(path, context).await?)
        }
    };

    Ok((name, parameters, kind))
}
