use crate::{app_state::AppState, engine::AudioRecordingState};
use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};
use tokio::fs;
use tokio_util::io::ReaderStream;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(recordings))
        .routes(routes!(download_recording, delete_recording))
}

#[derive(Serialize, ToSchema)]
struct RecordingInfo {
    path: String,
    size: u64,

    #[schema(required)]
    modified_unix_ms: Option<u64>,

    active: bool,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct RecordingPath {
    path: String,
}

#[utoipa::path(
    get,
    path = "/",
    operation_id = "list_recordings",
    responses(
        (status = OK, body = Vec<RecordingInfo>, description = "Recording files"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn recordings(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<RecordingInfo>>, (StatusCode, String)> {
    let active_path =
        if app_state.engine_handle.audio_recording_state().await == AudioRecordingState::Stopped {
            None
        } else {
            let path = app_state.engine_handle.audio_recording_parameters().await.file_path;
            Some(fs::canonicalize(&path).await.unwrap_or(path))
        };
    let mut pending_directories = vec![app_state.recordings_dir.to_path_buf()];
    let mut recordings = Vec::new();

    while let Some(directory) = pending_directories.pop() {
        let mut entries = fs::read_dir(&directory).await.map_err(internal_error)?;
        while let Some(entry) = entries.next_entry().await.map_err(internal_error)? {
            let file_type = entry.file_type().await.map_err(internal_error)?;
            if file_type.is_dir() {
                pending_directories.push(entry.path());
                continue;
            }
            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            let metadata = entry.metadata().await.map_err(internal_error)?;
            let relative = path
                .strip_prefix(&app_state.recordings_dir)
                .expect("recording directory entries must remain below their root");
            let modified_unix_ms = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .and_then(|duration| u64::try_from(duration.as_millis()).ok());
            recordings.push(RecordingInfo {
                path: relative.to_string_lossy().into_owned(),
                size: metadata.len(),
                modified_unix_ms,
                active: active_path.as_deref() == Some(path.as_path()),
            });
        }
    }

    recordings.sort_unstable_by(|left, right| {
        right.modified_unix_ms.cmp(&left.modified_unix_ms).then_with(|| left.path.cmp(&right.path))
    });
    Ok(Json(recordings))
}

#[utoipa::path(
    get,
    path = "/file",
    operation_id = "download_recording",
    params(RecordingPath),
    responses(
        (status = OK, body = String, content_type = "application/octet-stream", description = "Original recording file"),
        (status = BAD_REQUEST, body = String, description = "Invalid recording path"),
        (status = NOT_FOUND, body = String, description = "Recording not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn download_recording(
    State(app_state): State<AppState>,
    Query(query): Query<RecordingPath>,
) -> Result<Response, (StatusCode, String)> {
    let path = resolve_recording_path(&app_state.recordings_dir, &query.path).await?;
    let file = fs::File::open(&path).await.map_err(map_file_error)?;
    let size = file.metadata().await.map_err(internal_error)?.len();
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, "attachment")
        .header(header::CONTENT_LENGTH, size)
        .body(Body::from_stream(ReaderStream::new(file)))
        .map_err(internal_error)
}

#[utoipa::path(
    delete,
    path = "/file",
    operation_id = "delete_recording",
    params(RecordingPath),
    responses(
        (status = NO_CONTENT, description = "Recording deleted"),
        (status = BAD_REQUEST, body = String, description = "Invalid recording path"),
        (status = CONFLICT, body = String, description = "Recording is active"),
        (status = NOT_FOUND, body = String, description = "Recording not found"),
        (status = INTERNAL_SERVER_ERROR, body = String, description = "Internal Server Error")
    )
)]
async fn delete_recording(
    State(app_state): State<AppState>,
    Query(query): Query<RecordingPath>,
) -> Result<StatusCode, (StatusCode, String)> {
    let path = resolve_recording_path(&app_state.recordings_dir, &query.path).await?;
    if app_state.engine_handle.audio_recording_state().await != AudioRecordingState::Stopped {
        let active_path = app_state.engine_handle.audio_recording_parameters().await.file_path;
        let active_path = fs::canonicalize(&active_path).await.unwrap_or(active_path);
        if active_path == path {
            return Err((StatusCode::CONFLICT, "the active recording cannot be deleted".into()));
        }
    }
    fs::remove_file(path).await.map_err(map_file_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn resolve_recording_path(
    recordings_dir: &Path,
    requested: &str,
) -> Result<PathBuf, (StatusCode, String)> {
    let relative = Path::new(requested);
    if requested.is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err((StatusCode::BAD_REQUEST, "invalid recording path".into()));
    }

    let root = fs::canonicalize(recordings_dir).await.map_err(internal_error)?;
    let resolved = fs::canonicalize(root.join(relative)).await.map_err(map_file_error)?;
    if !resolved.starts_with(&root) {
        return Err((StatusCode::BAD_REQUEST, "recording path escapes its root".into()));
    }
    let metadata = fs::metadata(&resolved).await.map_err(map_file_error)?;
    if !metadata.is_file() {
        return Err((StatusCode::NOT_FOUND, "recording is not a file".into()));
    }
    Ok(resolved)
}

fn map_file_error(error: std::io::Error) -> (StatusCode, String) {
    let status = if error.kind() == std::io::ErrorKind::NotFound {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    (status, error.to_string())
}

fn internal_error(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_paths_must_be_nonempty_relative_normal_components() {
        for accepted in ["take.wav", "archive/take.wav"] {
            let path = Path::new(accepted);
            assert!(!accepted.is_empty());
            assert!(!path.is_absolute());
            assert!(path.components().all(|component| matches!(component, Component::Normal(_))));
        }

        for rejected in ["", "/take.wav", "../take.wav", "archive/../take.wav", "./take.wav"] {
            let path = Path::new(rejected);
            assert!(
                rejected.is_empty()
                    || path.is_absolute()
                    || path
                        .components()
                        .any(|component| !matches!(component, Component::Normal(_)))
            );
        }
    }

    #[tokio::test]
    async fn recording_path_resolution_stays_below_root_and_requires_a_file() {
        let directory =
            std::env::temp_dir().join(format!("aaw-recordings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).await.expect("create recordings directory");
        fs::write(directory.join("take.wav"), b"recording").await.expect("write recording");
        fs::create_dir(directory.join("folder")).await.expect("create nested directory");

        let resolved =
            resolve_recording_path(&directory, "take.wav").await.expect("resolve recording");
        assert_eq!(resolved, fs::canonicalize(directory.join("take.wav")).await.unwrap());
        assert_eq!(
            resolve_recording_path(&directory, "../take.wav").await.unwrap_err().0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            resolve_recording_path(&directory, "folder").await.unwrap_err().0,
            StatusCode::NOT_FOUND
        );

        fs::remove_dir_all(directory).await.expect("remove recordings directory");
    }
}
