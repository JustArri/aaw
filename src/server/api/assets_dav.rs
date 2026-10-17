use axum::{
    Extension,
    extract::{MatchedPath, OriginalUri, Request},
    response::IntoResponse,
    routing::any,
};
use dav_server::{DavHandler, fakels::FakeLs, localfs::LocalFs};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

pub fn router<S>(route_prefix: &str, assets_dir: impl AsRef<std::path::Path>) -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    #[derive(OpenApi)]
    #[openapi(paths(documentation))]
    struct Api;

    let dav = DavHandler::builder()
        .filesystem(LocalFs::new(assets_dir, false, false, false))
        .locksystem(FakeLs::new())
        .hide_symlinks(true)
        .build_handler();

    let trailing_slash = format!("{route_prefix}/");
    let descendants = format!("{route_prefix}/{{*path}}");

    OpenApiRouter::new()
        .nest(route_prefix, OpenApiRouter::with_openapi(Api::openapi()))
        .route(route_prefix, any(handle_dav))
        .route(&trailing_slash, any(handle_dav))
        .route(&descendants, any(handle_dav))
        .layer(Extension(dav))
}

#[utoipa::path(
    options,
    path = "/",
    operation_id = "assetsWebDav",
    tag = "assets",
    description =
        "WebDAV asset filesystem. Supports GET, HEAD, PUT, PATCH, DELETE, PROPFIND, \
         PROPPATCH, MKCOL, COPY, MOVE, LOCK and UNLOCK for resources below this path.",
    responses(
        (status = OK, description = "WebDAV capabilities"),
    )
)]
#[allow(dead_code)]
#[allow(clippy::unused_async)]
async fn documentation() {}

async fn handle_dav(
    Extension(dav): Extension<DavHandler>,
    matched_path: MatchedPath,
    OriginalUri(original_uri): OriginalUri,
    mut request: Request,
) -> impl IntoResponse {
    let prefix = matched_path
        .as_str()
        .strip_suffix("/{*path}")
        .unwrap_or_else(|| matched_path.as_str())
        .trim_end_matches('/')
        .to_owned();

    *request.uri_mut() = original_uri;

    dav.handle_with(DavHandler::builder().strip_prefix(prefix), request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
    };

    #[tokio::test]
    async fn accepts_collection_urls_with_and_without_trailing_slash() {
        let assets_dir = std::env::temp_dir().join(format!("aaw-dav-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir(&assets_dir).await.unwrap();

        let assets_router: OpenApiRouter = router("/assets", &assets_dir);
        let app = Router::new().nest("/api", Router::from(assets_router));
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        for path in ["/api/assets", "/api/assets/"] {
            let mut stream = TcpStream::connect(address).await.unwrap();
            stream
                .write_all(
                    format!(
                        "PROPFIND {path} HTTP/1.1\r\nHost: {address}\r\nDepth: 0\r\nConnection: close\r\n\r\n"
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();

            let mut response = String::new();
            stream.read_to_string(&mut response).await.unwrap();

            assert!(response.starts_with("HTTP/1.1 207 Multi-Status"), "{response}");
            assert!(response.contains("<D:href>/api/assets/</D:href>"), "{response}");
        }

        server.abort();
        tokio::fs::remove_dir(assets_dir).await.unwrap();
    }
}
