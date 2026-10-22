use crate::{
    app_state::AppState,
    engine::{
        graph::{InstrumentId, SynthId},
        instrument_graph::CtrInstrument,
    },
    midi::{MidiFilter, MidiKeyTransform, MidiVelocityTransform},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(set_global_transposition))
        .routes(routes!(global_transposition))
        .routes(routes!(instrument_list))
        .routes(routes!(instrument_append_new))
        .routes(routes!(instrument))
        .routes(routes!(instrument_remove))
        .routes(routes!(instrument_set_synth))
        .routes(routes!(instrument_synth))
        .routes(routes!(instrument_set_enabled))
        .routes(routes!(instrument_enabled))
        .routes(routes!(instrument_set_ignore_key_transform))
        .routes(routes!(instrument_ignore_key_transform))
        .routes(routes!(instrument_set_key_transform))
        .routes(routes!(instrument_key_transform))
        .routes(routes!(instrument_set_ignore_velocity_transform))
        .routes(routes!(instrument_ignore_velocity_transform))
        .routes(routes!(instrument_set_velocity_transform))
        .routes(routes!(instrument_velocity_transform))
        .routes(routes!(instrument_set_channel_filter))
        .routes(routes!(instrument_channel_filter))
        .routes(routes!(instrument_set_cc_filter))
        .routes(routes!(instrument_cc_filter))
        .routes(routes!(instrument_set_ignore_pitch_bend))
        .routes(routes!(instrument_ignore_pitch_bend))
        .routes(routes!(instrument_set_ignore_program_change))
        .routes(routes!(instrument_ignore_program_change))
        .routes(routes!(instrument_set_ignore_channel_aftertouch))
        .routes(routes!(instrument_ignore_channel_aftertouch))
        .routes(routes!(instrument_set_ignore_polyphonic_aftertouch))
        .routes(routes!(instrument_ignore_polyphonic_aftertouch))
        .routes(routes!(instrument_set_use_global_transposition))
        .routes(routes!(instrument_use_global_transposition))
        .routes(routes!(instrument_set_name))
        .routes(routes!(instrument_name))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/global-transposition",
    request_body(
        content = i8,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn set_global_transposition(
    State(app_state): State<AppState>,
    Json(transposition): Json<i8>,
) -> Result<StatusCode, (StatusCode, String)> {
    app_state.engine_handle.set_global_transposition(transposition).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/global-transposition",
    responses(
        (status = OK, body = i8, content_type = "application/json", description = "OK"),
    )
)]
async fn global_transposition(State(app_state): State<AppState>) -> Json<i8> {
    Json(app_state.engine_handle.project().await.instrument_graph.global_transposition)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments",
    responses(
        (
            status = OK,
            body = HashMap<InstrumentId, CtrInstrument>,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn instrument_list(
    State(app_state): State<AppState>,
) -> Json<HashMap<InstrumentId, CtrInstrument>> {
    Json(app_state.engine_handle.project().await.instrument_graph.instruments.clone().into())
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/graph/instrument/instruments",
    responses(
        (
            status = OK,
            body = InstrumentId,
            content_type = "application/json",
            description = "Created"
        ),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_append_new(
    State(app_state): State<AppState>,
) -> Result<Json<InstrumentId>, (StatusCode, String)> {
    Ok(Json(app_state.engine_handle.instrument_append_new().await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}",
    params(InstrumentPath),
    responses(
        (
            status = OK,
            body = CtrInstrument,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<CtrInstrument>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, Clone::clone).await?))
}

//--------------------------------------------------------------------------------------------------

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Path)]
struct InstrumentPath {
    #[param(value_type = usize)]
    instrument_id: InstrumentId,
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    delete,
    path = "/graph/instrument/instruments/{instrument_id}",
    params(InstrumentPath),
    responses(
        (status = NO_CONTENT, description = "Removed"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_remove(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_remove(path.instrument_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/synth",
    params(InstrumentPath),
    request_body(
        content = Option<SynthId>,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(synth): Json<Option<SynthId>>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_synth(path.instrument_id, synth).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/synth",
    params(InstrumentPath),
    responses(
        (
            status = OK,
            body = Option<SynthId>,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_synth(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<Option<SynthId>>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.synth_id).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/enabled",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_enabled(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_enabled(path.instrument_id, enabled).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/enabled",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_enabled(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.enabled).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-key-transform",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(flag): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_ignore_key_transform(path.instrument_id, flag).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-key-transform",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.ignore_key_transform).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/key-transform",
    params(InstrumentPath),
    request_body(
        content = MidiKeyTransform,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(transform): Json<MidiKeyTransform>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_key_transform(path.instrument_id, transform).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/key-transform",
    params(InstrumentPath),
    responses(
        (
            status = OK,
            body = MidiKeyTransform,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_key_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<MidiKeyTransform>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.key_transform.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-velocity-transform",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(flag): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_ignore_velocity_transform(path.instrument_id, flag)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-velocity-transform",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&app_state, path.instrument_id, |s| s.ignore_velocity_transform).await?,
    ))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/velocity-transform",
    params(InstrumentPath),
    request_body(
        content = MidiVelocityTransform,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(transform): Json<MidiVelocityTransform>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_velocity_transform(path.instrument_id, transform)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/velocity-transform",
    params(InstrumentPath),
    responses(
        (
            status = OK,
            body = MidiVelocityTransform,
            content_type = "application/json",
            description = "OK"
        ),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_velocity_transform(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<MidiVelocityTransform>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&app_state, path.instrument_id, |s| s.velocity_transform.clone()).await?,
    ))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/channel-filter",
    params(InstrumentPath),
    request_body(
        content = MidiFilter,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_channel_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(filter): Json<MidiFilter>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_channel_filter(path.instrument_id, filter).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/channel-filter",
    params(InstrumentPath),
    responses(
        (status = OK, body = MidiFilter, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_channel_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<MidiFilter>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.channel_filter.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/cc-filter",
    params(InstrumentPath),
    request_body(
        content = MidiFilter,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_cc_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(filter): Json<MidiFilter>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_cc_filter(path.instrument_id, filter).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/cc-filter",
    params(InstrumentPath),
    responses(
        (status = OK, body = MidiFilter, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_cc_filter(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<MidiFilter>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.cc_filter.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-pitch-bend",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_pitch_bend(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_ignore_pitch_bend(path.instrument_id, enabled).await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-pitch-bend",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_pitch_bend(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.ignore_pitch_bend).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-program-change",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_program_change(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_ignore_program_change(path.instrument_id, enabled)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-program-change",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_program_change(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.ignore_program_change).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-channel-aftertouch",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_channel_aftertouch(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_ignore_channel_aftertouch(path.instrument_id, enabled)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-channel-aftertouch",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_channel_aftertouch(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&app_state, path.instrument_id, |s| s.ignore_channel_aftertouch).await?,
    ))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-polyphonic-aftertouch",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_ignore_polyphonic_aftertouch(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_ignore_polyphonic_aftertouch(path.instrument_id, enabled)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/ignore-polyphonic-aftertouch",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_ignore_polyphonic_aftertouch(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(
        with_instrument(&app_state, path.instrument_id, |s| s.ignore_polyphonic_aftertouch).await?,
    ))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/use-global-transposition",
    params(InstrumentPath),
    request_body(
        content = bool,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error"),
    )
)]
async fn instrument_set_use_global_transposition(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(enabled): Json<bool>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state
        .engine_handle
        .instrument_set_use_global_transposition(path.instrument_id, enabled)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/use-global-transposition",
    params(InstrumentPath),
    responses(
        (status = OK, body = bool, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_use_global_transposition(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<bool>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.use_global_transposition).await?))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/graph/instrument/instruments/{instrument_id}/name",
    params(InstrumentPath),
    request_body(
        content = String,
        content_type = "application/json"
    ),
    responses(
        (status = NO_CONTENT, description = "Value set"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_set_name(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
    Json(name): Json<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    with_instrument(&app_state, path.instrument_id, |_| ()).await?;
    app_state.engine_handle.instrument_set_name(path.instrument_id, name).await;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/graph/instrument/instruments/{instrument_id}/name",
    params(InstrumentPath),
    responses(
        (status = OK, body = String, content_type = "application/json", description = "OK"),
        (status = NOT_FOUND, body = String, description = "Instrument not found"),
    )
)]
async fn instrument_name(
    State(app_state): State<AppState>,
    Path(path): Path<InstrumentPath>,
) -> Result<Json<String>, (StatusCode, String)> {
    Ok(Json(with_instrument(&app_state, path.instrument_id, |s| s.name.clone()).await?))
}

//--------------------------------------------------------------------------------------------------

async fn with_instrument<T>(
    app_state: &AppState,
    id: InstrumentId,
    f: impl FnOnce(&CtrInstrument) -> T,
) -> Result<T, (StatusCode, String)> {
    let project = app_state.engine_handle.project().await;
    let result_opt = project.instrument_graph.instruments.get(id).map(f);
    result_opt.ok_or((StatusCode::NOT_FOUND, "not found".into()))
}
