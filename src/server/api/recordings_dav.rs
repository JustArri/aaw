use axum::{
    Extension,
    body::Body,
    extract::{MatchedPath, OriginalUri, Request},
    response::Response,
    routing::any,
};
use dav_server::{DavHandler, body::Body as DavBody, fakels::FakeLs, localfs::LocalFs};
use http_body::{Body as HttpBody, Frame, SizeHint};
use std::{
    pin::Pin,
    task::{Context, Poll},
};
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
    operation_id = "recordingsWebDav",
    tag = "recordings",
    description =
        "WebDAV recoding filesystem. Supports GET, HEAD, PUT, PATCH, DELETE, PROPFIND, \
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
) -> Response {
    let prefix = matched_path
        .as_str()
        .strip_suffix("/{*path}")
        .unwrap_or_else(|| matched_path.as_str())
        .trim_end_matches('/')
        .to_owned();

    *request.uri_mut() = original_uri;

    dav.handle_with(DavHandler::builder().strip_prefix(prefix), request)
        .await
        .map(|body| Body::new(FusedDavBody::new(body)))
}

struct FusedDavBody {
    inner: DavBody,
    finished: bool,
}

impl FusedDavBody {
    fn new(inner: DavBody) -> Self {
        Self { inner, finished: false }
    }
}

impl HttpBody for FusedDavBody {
    type Data = <DavBody as HttpBody>::Data;
    type Error = <DavBody as HttpBody>::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if self.finished {
            return Poll::Ready(None);
        }

        let result = Pin::new(&mut self.inner).poll_frame(context);
        if matches!(result, Poll::Ready(None) | Poll::Ready(Some(Err(_)))) {
            self.finished = true;
        }
        result
    }

    fn is_end_stream(&self) -> bool {
        self.finished || self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
