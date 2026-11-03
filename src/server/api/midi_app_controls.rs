use crate::{
    app_state::AppState,
    engine::midi_app_controls::{CtrMidiEngineControls, StoredMidiEngineControls, prepare_engine},
    files,
};
use axum::{Json, extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(midi_app_controls, set_midi_app_controls))
}

#[utoipa::path(
    get,
    path = "/",
    operation_id = "engine_midi_app_controls",
    responses((status = OK, body = StoredMidiEngineControls, description = "Engine MIDI app controls"))
)]
async fn midi_app_controls(State(app_state): State<AppState>) -> Json<StoredMidiEngineControls> {
    let controls: CtrMidiEngineControls = app_state.engine_handle.midi_engine_controls().await;
    Json(controls.into())
}

#[utoipa::path(
    put,
    path = "/",
    operation_id = "set_engine_midi_app_controls",
    request_body(content = StoredMidiEngineControls, content_type = "application/json"),
    responses((status = NO_CONTENT, description = "Engine MIDI app controls replaced"))
)]
async fn set_midi_app_controls(
    State(app_state): State<AppState>,
    Json(stored): Json<StoredMidiEngineControls>,
) -> StatusCode {
    let (controls, _) = prepare_engine(stored);
    app_state.engine_handle.set_midi_engine_controls(controls.clone()).await;
    files::save(
        &app_state.aaw_root_dir,
        crate::CFG_MIDI_APP_CONTROLS,
        &StoredMidiEngineControls::from(controls),
    )
    .await;
    StatusCode::NO_CONTENT
}
