use axum::{
    body::Body,
    http::{Method, StatusCode, Uri, header},
    response::Response,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/build"]
struct FrontendAssets;

const INDEX: &str = "index.html";

pub async fn serve(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return response(StatusCode::METHOD_NOT_ALLOWED, Body::empty(), None, None);
    }

    let path = uri.path();
    if path == "/api" || path.starts_with("/api/") {
        return response(StatusCode::NOT_FOUND, Body::empty(), None, None);
    }

    let requested_path = path.trim_start_matches('/');
    let asset_path = if requested_path.is_empty() { INDEX } else { requested_path };

    let Some((asset, served_path)) = FrontendAssets::get(asset_path)
        .map(|asset| (asset, asset_path))
        .or_else(|| FrontendAssets::get(INDEX).map(|asset| (asset, INDEX)))
    else {
        return response(StatusCode::NOT_FOUND, Body::empty(), None, None);
    };

    let content_type =
        mime_guess::from_path(served_path).first_or_octet_stream().as_ref().to_owned();
    let cache_control = if served_path.starts_with("_app/immutable/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let body = if method == Method::HEAD { Body::empty() } else { Body::from(asset.data) };

    response(StatusCode::OK, body, Some(&content_type), Some(cache_control))
}

fn response(
    status: StatusCode,
    body: Body,
    content_type: Option<&str>,
    cache_control: Option<&str>,
) -> Response {
    let mut response = Response::builder().status(status);

    if let Some(content_type) = content_type {
        response = response.header(header::CONTENT_TYPE, content_type);
    }
    if let Some(cache_control) = cache_control {
        response = response.header(header::CACHE_CONTROL, cache_control);
    }

    response.body(body).expect("valid embedded frontend response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_index_for_the_root_and_client_routes() {
        for path in ["/", "/instruments", "/settings/appearance"] {
            let response = serve(Method::GET, path.parse().unwrap()).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
        }
    }

    #[tokio::test]
    async fn does_not_handle_unknown_api_routes() {
        let response = serve(Method::GET, "/api/unknown".parse().unwrap()).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn serves_embedded_static_assets() {
        let response = serve(Method::GET, "/favicon.png".parse().unwrap()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
    }
}
