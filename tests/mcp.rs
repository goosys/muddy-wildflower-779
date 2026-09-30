use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
};
use haikunator_worker::controllers::axum_router;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn request(method: Method, body: &str, origin: Option<&str>, host: &str) -> Response {
    let mut builder = Request::builder()
        .method(method)
        .uri("/mcp")
        .header(header::HOST, host)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-11-25");
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    axum_router()
        .oneshot(builder.body(Body::from(body.to_owned())).expect("request"))
        .await
        .expect("response")
}

async fn rpc(method: &str, params: Value) -> (HeaderMap, Value) {
    let response = request(
        Method::POST,
        &json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string(),
        None,
        "localhost:8787",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers()[header::CONTENT_TYPE]
        .to_str()
        .expect("content type")
        .starts_with("application/json"));
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let message: Value = serde_json::from_slice(&body).expect("JSON-RPC response");
    assert_eq!(message["jsonrpc"], "2.0");
    assert_eq!(message["id"], 1);
    (headers, message)
}

fn assert_generated_string(message: &Value) {
    let result = &message["result"];
    assert_eq!(result["isError"], false);
    let generated = result["structuredContent"]
        .as_object()
        .expect("JSON object");
    assert_eq!(generated.keys().collect::<Vec<_>>(), ["name"]);
    let parts: Vec<_> = generated["name"]
        .as_str()
        .expect("string")
        .split('-')
        .collect();
    assert_eq!(parts.len(), 3);
    assert!(!parts[0].is_empty() && !parts[1].is_empty());
    assert_eq!(parts[2].len(), 4);
    assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
    let text: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().expect("text content"))
            .expect("JSON text content");
    assert_eq!(text, result["structuredContent"]);
}

#[tokio::test(flavor = "current_thread")]
async fn stateless_lifecycle_lists_and_calls_haikunator_generator() {
    let (headers, initialized) = rpc(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "integration-test", "version": "1.0.0"}
        }),
    )
    .await;
    assert!(!headers.contains_key("mcp-session-id"));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-11-25");
    assert_eq!(
        initialized["result"]["serverInfo"]["name"],
        "haikunator-generator"
    );
    assert!(initialized["result"]["capabilities"]["tools"].is_object());

    let notification = request(
        Method::POST,
        &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string(),
        None,
        "localhost:8787",
    )
    .await;
    assert_eq!(notification.status(), StatusCode::ACCEPTED);
    assert!(to_bytes(notification.into_body(), 64 * 1024)
        .await
        .expect("notification body")
        .is_empty());

    let (_, listed) = rpc("tools/list", json!({})).await;
    let tools = listed["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "gen");
    assert_eq!(tools[0]["title"], "Haikunator Generator");
    assert_eq!(tools[0]["annotations"]["readOnlyHint"], true);
    assert!(tools[0]["description"]
        .as_str()
        .expect("description")
        .contains("Heroku-like memorable random string"));
    assert_eq!(tools[0]["inputSchema"]["type"], "object");
    assert!(tools[0]["inputSchema"]
        .get("properties")
        .is_none_or(|properties| properties == &json!({})));
    assert_eq!(tools[0]["inputSchema"]["additionalProperties"], false);

    let (_, generated) = rpc("tools/call", json!({"name": "gen", "arguments": {}})).await;
    assert_generated_string(&generated);
}

#[tokio::test(flavor = "current_thread")]
async fn generation_matches_existing_json_api_interface() {
    let response = axum_router()
        .oneshot(
            Request::builder()
                .uri("/api/gen")
                .body(Body::empty())
                .expect("request"),
        )
        .await;
    let body = to_bytes(response.expect("response").into_body(), 64 * 1024)
        .await
        .expect("body");
    let api: Value = serde_json::from_slice(&body).expect("API JSON response");
    for params in [
        json!({"name": "gen", "arguments": {}}),
        json!({"name": "gen"}),
    ] {
        let (_, generated) = rpc("tools/call", params).await;
        assert_generated_string(&generated);
        assert_eq!(
            generated["result"]["structuredContent"]
                .as_object()
                .expect("MCP response")
                .keys()
                .collect::<Vec<_>>(),
            api.as_object()
                .expect("API response")
                .keys()
                .collect::<Vec<_>>()
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_arguments_and_unknown_tools_return_errors() {
    for arguments in [
        json!({"count": 1}),
        json!({"delimiter": "-"}),
        json!({"unknown": true}),
    ] {
        let (_, message) = rpc("tools/call", json!({"name": "gen", "arguments": arguments})).await;
        assert!(
            message["error"]["code"] == -32602 || message["result"]["isError"] == true,
            "gen must reject unsupported arguments: {message}"
        );
    }
    let (_, message) = rpc("tools/call", json!({"name": "unknown", "arguments": {}})).await;
    assert_eq!(message["error"]["code"], -32602);
    let (_, message) = rpc("unknown/method", json!({})).await;
    assert_eq!(message["error"]["code"], -32601);
}

#[tokio::test(flavor = "current_thread")]
async fn transport_rejects_untrusted_origins_hosts_and_unsupported_methods() {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}).to_string();
    for method in [Method::GET, Method::DELETE] {
        assert_eq!(
            request(method, "", None, "localhost:8787").await.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    for origin in ["https://untrusted.example", "null", "http://localhost:9999"] {
        assert_eq!(
            request(Method::POST, &body, Some(origin), "localhost:8787")
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(
        request(Method::POST, &body, None, "untrusted.example")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            Method::POST,
            &body,
            Some("https://haikunator-generator.goosysapp.net"),
            "haikunator-generator.goosysapp.net"
        )
        .await
        .status(),
        StatusCode::OK
    );
}

#[tokio::test(flavor = "current_thread")]
async fn transport_limits_request_size_and_rejects_invalid_json() {
    let oversized = json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": {"padding": "x".repeat(17 * 1024)}}).to_string();
    assert_eq!(
        request(Method::POST, &oversized, None, "localhost:8787")
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(
        request(Method::POST, "{", None, "localhost:8787")
            .await
            .status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}
