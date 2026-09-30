use axum::{
    body::{to_bytes, Body},
    http::{header, Method, Request, StatusCode},
    response::Response,
};
use haikunator_worker::{controllers::axum_router, views::haikunator::GeneratorResponse};
use tower::ServiceExt;

async fn request(method: Method, path: &str) -> Response {
    axum_router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("router response")
}

fn assert_generated_name(name: &str) {
    let parts: Vec<_> = name.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_ne!(parts[0], "");
    assert_ne!(parts[1], "");
    assert_eq!(parts[2].len(), 4);
    assert!(parts[2].chars().all(|character| character.is_ascii_digit()));
}

#[tokio::test(flavor = "current_thread")]
async fn generate_endpoints_return_memorable_names() {
    let json_response = request(Method::GET, "/api/gen").await;
    assert_eq!(json_response.status(), StatusCode::OK);
    assert_eq!(
        json_response.headers()[header::CONTENT_TYPE],
        "application/json"
    );
    let body = to_bytes(json_response.into_body(), 64 * 1024)
        .await
        .expect("JSON body");
    let generated: GeneratorResponse = serde_json::from_slice(&body).expect("generated name");
    assert_generated_name(&generated.name);

    let text_response = request(Method::GET, "/api/gen.txt").await;
    assert_eq!(text_response.status(), StatusCode::OK);
    assert_eq!(
        text_response.headers()[header::CONTENT_TYPE],
        "text/plain; charset=utf-8"
    );
    let body = to_bytes(text_response.into_body(), 64 * 1024)
        .await
        .expect("text body");
    assert_generated_name(std::str::from_utf8(&body).expect("UTF-8 name"));
}

#[tokio::test(flavor = "current_thread")]
async fn generate_endpoints_support_head_and_reject_post() {
    for path in ["/api/gen", "/api/gen.txt"] {
        let response = request(Method::HEAD, path).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("HEAD body")
            .is_empty());

        let response = request(Method::POST, path).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        let allowed: Vec<_> = response.headers()[header::ALLOW]
            .to_str()
            .expect("Allow header")
            .split(',')
            .map(str::trim)
            .collect();
        assert!(allowed.contains(&"GET"));
        assert!(allowed.contains(&"HEAD"));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_api_route_returns_not_found() {
    let response = request(Method::GET, "/api/does-not-exist").await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
