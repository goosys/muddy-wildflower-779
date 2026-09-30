use axum::{http::header::HeaderName, http::HeaderValue, middleware, response::Response};
use std::{collections::BTreeMap, sync::LazyLock};
use tower::ServiceExt;
use tower_http::cors::{Any, CorsLayer};
use worker::{event, Context, Env, HttpRequest};

#[cfg(target_os = "emscripten")]
mod streaming;

const fn main() {}

static SECURITY_HEADERS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../security-headers.json"))
        .expect("security headers must be valid JSON")
});

async fn secure_headers(mut response: Response) -> Response {
    for (name, value) in SECURITY_HEADERS.iter() {
        response.headers_mut().insert(
            HeaderName::from_bytes(name.as_bytes()).expect("valid header name"),
            HeaderValue::from_str(value).expect("valid header value"),
        );
    }
    response
}

fn router() -> axum::Router {
    decorate_router(haikunator_worker::controllers::axum_router())
}

fn decorate_router(router: axum::Router) -> axum::Router {
    router
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(middleware::map_response(secure_headers))
}

async fn buffered_response(response: Response) -> worker::Result<worker::Response> {
    let (parts, body) = response.into_parts();
    // Small API and JSON MCP responses retain the existing buffered conversion.
    let bytes = axum::body::to_bytes(body, 64 * 1024)
        .await
        .map_err(|err| worker::Error::RustError(err.to_string()))?;
    let response = if bytes.is_empty() {
        worker::Response::empty()?
    } else {
        worker::Response::from_bytes(bytes.to_vec())?
    };
    Ok(response
        .with_status(parts.status.as_u16())
        .with_headers(parts.headers.into()))
}

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, ctx: Context) -> worker::Result<worker::Response> {
    #[cfg(target_os = "emscripten")]
    if req.uri().path() == "/mcp" {
        return streaming::respond(req).await;
    }
    let _ = ctx;
    buffered_response(router().oneshot(req).await?).await
}
