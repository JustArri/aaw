use crate::app_state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use std::path::PathBuf;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(assets))
        .routes(routes!(asset_file_extensions))
        .routes(routes!(refresh_assets))
}

#[derive(Serialize, ToSchema)]
struct ClapAsset {
    #[schema(value_type = String)]
    file_path: PathBuf,

    plugin_id: String,

    #[schema(required)]
    name: Option<String>,

    #[schema(required)]
    version: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct Assets {
    #[schema(value_type = Vec<String>)]
    fluid: Vec<PathBuf>,

    #[schema(value_type = Vec<String>)]
    sfizz: Vec<PathBuf>,

    clap_instruments: Vec<ClapAsset>,
}

#[derive(Serialize, ToSchema)]
struct AssetFileExtensions {
    fluid: &'static [&'static str],
    sfizz: &'static [&'static str],
    clap: &'static [&'static str],
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = OK, body = Assets, content_type = "application/json", description = "OK"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn assets(State(app_state): State<AppState>) -> Result<Json<Assets>, (StatusCode, String)> {
    let assets = app_state.assets.read().await;
    Ok(Json(Assets {
        fluid: assets.fluid_list.clone(),
        sfizz: assets.sfizz_list.clone(),
        clap_instruments: assets
            .clap_instrument_list
            .iter()
            .map(|plugin| ClapAsset {
                file_path: plugin.file_path.clone(),
                plugin_id: plugin.plugin_id.clone(),
                name: plugin.name.clone(),
                version: plugin.version.clone(),
            })
            .collect(),
    }))
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/file-extensions",
    responses(
        (
            status = OK,
            body = AssetFileExtensions,
            content_type = "application/json",
            description = "OK"
        ),
    )
)]
async fn asset_file_extensions() -> Json<AssetFileExtensions> {
    Json(AssetFileExtensions {
        fluid: crate::assets::FLUID_EXTENSIONS,
        sfizz: crate::assets::SFIZZ_EXTENSIONS,
        clap: crate::assets::CLAP_EXTENSIONS,
    })
}

//--------------------------------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/refresh",
    responses(
        (status = NO_CONTENT, description = "Assets refreshed"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error"),
    )
)]
async fn refresh_assets(
    State(app_state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut assets = app_state.assets.write().await;
    *assets = crate::assets::Assets::new(assets.dir.clone())
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

//--------------------------------------------------------------------------------------------------
