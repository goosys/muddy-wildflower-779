use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
};
use futures_util::StreamExt;
use haikunator_worker::controllers::axum_router;
use serde_json::{json, Value};
use std::time::Duration;
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
    assert_generator_response(&result["structuredContent"]);
    let text: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().expect("text content"))
            .expect("JSON text content");
    assert_eq!(text, result["structuredContent"]);
}

fn assert_generator_response(value: &Value) {
    let generated = value.as_object().expect("JSON object");
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
}

fn assert_continuous_result(message: &Value, count: usize) {
    assert!(message.get("error").is_none(), "{message}");
    let result = &message["result"];
    assert_eq!(result["isError"], false);
    assert_eq!(
        result["structuredContent"]
            .as_object()
            .expect("structured content")
            .keys()
            .collect::<Vec<_>>(),
        ["results"]
    );
    let generated = result["structuredContent"]["results"]
        .as_array()
        .expect("results");
    assert_eq!(generated.len(), count);
    for item in generated {
        assert_generator_response(item);
    }
    let text: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().expect("text content"))
            .expect("JSON text content");
    assert_eq!(text, result["structuredContent"]);
}

#[derive(Default)]
struct SseMessages {
    pending: String,
}

impl SseMessages {
    fn push(&mut self, bytes: &[u8]) -> Vec<Value> {
        self.pending
            .push_str(std::str::from_utf8(bytes).expect("SSE UTF-8"));
        let mut messages = Vec::new();
        loop {
            let delimiter = self
                .pending
                .find("\n\n")
                .map(|offset| (offset, 2))
                .into_iter()
                .chain(self.pending.find("\r\n\r\n").map(|offset| (offset, 4)))
                .min_by_key(|(offset, _)| *offset);
            let Some((offset, length)) = delimiter else {
                break;
            };
            let event: String = self.pending.drain(..offset + length).collect();
            let data = event
                .lines()
                .filter_map(|line| {
                    line.strip_prefix("data:")
                        .map(|data| data.strip_prefix(' ').unwrap_or(data))
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !data.is_empty() {
                messages.push(serde_json::from_str(&data).expect("SSE JSON-RPC message"));
            }
        }
        messages
    }
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
    assert_eq!(tools.len(), 2);
    let gen = tools
        .iter()
        .find(|tool| tool["name"] == "gen")
        .expect("gen tool");
    assert_eq!(gen["title"], "Haikunator Generator");
    assert_eq!(gen["annotations"]["readOnlyHint"], true);
    assert!(gen["description"]
        .as_str()
        .expect("description")
        .contains("Heroku-like memorable random string"));
    assert_eq!(gen["inputSchema"]["type"], "object");
    assert!(gen["inputSchema"]
        .get("properties")
        .is_none_or(|properties| properties == &json!({})));
    assert_eq!(gen["inputSchema"]["additionalProperties"], false);

    let continuous = tools
        .iter()
        .find(|tool| tool["name"] == "gen_continuous")
        .expect("continuous tool");
    assert_eq!(
        continuous["title"],
        "Haikunator Generator Continuous Generation"
    );
    assert_eq!(continuous["annotations"]["readOnlyHint"], true);
    let schema = &continuous["inputSchema"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["count"]["type"], "integer");
    assert_eq!(schema["properties"]["count"]["default"], 5);
    assert_eq!(schema["properties"]["count"]["minimum"], 1);
    assert_eq!(schema["properties"]["count"]["maximum"], 20);
    assert_eq!(schema["properties"]["interval_ms"]["type"], "integer");
    assert_eq!(schema["properties"]["interval_ms"]["default"], 500);
    assert_eq!(schema["properties"]["interval_ms"]["minimum"], 100);
    assert_eq!(schema["properties"]["interval_ms"]["maximum"], 2000);

    let (_, generated) = rpc("tools/call", json!({"name": "gen", "arguments": {}})).await;
    assert_generated_string(&generated);
}

#[tokio::test(flavor = "current_thread")]
async fn continuous_generation_returns_json_without_a_progress_token() {
    for (arguments, count) in [
        (json!({}), 5),
        (json!({"count": 1, "interval_ms": 100}), 1),
        (json!({"count": 20, "interval_ms": 100}), 20),
    ] {
        let (headers, generated) = rpc(
            "tools/call",
            json!({"name": "gen_continuous", "arguments": arguments}),
        )
        .await;
        assert!(!headers.contains_key("mcp-session-id"));
        assert_continuous_result(&generated, count);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn continuous_generation_streams_progress_without_blocking_existing_routes() {
    let response = request(
        Method::POST,
        &json!({
            "jsonrpc": "2.0", "id": 42, "method": "tools/call",
            "params": {"name": "gen_continuous", "arguments": {"count": 4, "interval_ms": 500}, "_meta": {"progressToken": "host-progress"}}
        }).to_string(),
        None,
        "localhost:8787",
    ).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers()[header::CONTENT_TYPE]
        .to_str()
        .expect("content type")
        .starts_with("text/event-stream"));
    assert!(!response.headers().contains_key("mcp-session-id"));
    let mut stream = response.into_body().into_data_stream();
    let mut parser = SseMessages::default();
    let mut messages = Vec::new();
    loop {
        let chunk = tokio::time::timeout(Duration::from_secs(3), stream.next())
            .await
            .expect("first progress timeout")
            .expect("progress chunk")
            .expect("body chunk");
        messages.extend(parser.push(&chunk));
        assert!(
            !messages.iter().any(|message| message["id"] == 42),
            "final result must follow incremental progress"
        );
        if messages
            .iter()
            .any(|message| message["params"]["progress"] == 1.0)
        {
            break;
        }
    }

    tokio::time::timeout(Duration::from_millis(700), async {
        let (generated, api) = tokio::join!(
            rpc("tools/call", json!({"name": "gen", "arguments": {}})),
            axum_router().oneshot(
                Request::builder()
                    .uri("/api/gen")
                    .body(Body::empty())
                    .expect("API request")
            )
        );
        assert_generated_string(&generated.1);
        let response = api.expect("API response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("API body");
        assert_generator_response(&serde_json::from_slice(&body).expect("API JSON"));
    })
    .await
    .expect("existing routes should finish while continuous generation is pending");

    while let Some(chunk) = tokio::time::timeout(Duration::from_secs(3), stream.next())
        .await
        .expect("stream completion timeout")
    {
        messages.extend(parser.push(&chunk.expect("body chunk")));
    }
    assert!(parser.pending.is_empty(), "unfinished SSE event");
    assert_progress_sequence(&messages);
}

fn assert_progress_sequence(messages: &[Value]) {
    let progress: Vec<_> = messages
        .iter()
        .filter(|message| message["method"] == "notifications/progress")
        .collect();
    assert_eq!(
        progress
            .iter()
            .filter(|message| message["params"]["progress"] == 0.0)
            .count(),
        1
    );
    let generated: Vec<_> = progress
        .iter()
        .filter(|message| message["params"]["progress"] != 0.0)
        .collect();
    assert_eq!(generated.len(), 4);
    let mut items = Vec::new();
    for (progress, notification) in (1_u8..=4).zip(&generated) {
        assert_eq!(notification["params"]["progressToken"], "host-progress");
        assert_eq!(notification["params"]["progress"], f64::from(progress));
        assert_eq!(notification["params"]["total"], 4.0);
        let item: Value = serde_json::from_str(
            notification["params"]["message"]
                .as_str()
                .expect("progress message"),
        )
        .expect("generator JSON");
        assert_generator_response(&item);
        items.push(item);
    }
    let terminal: Vec<_> = messages
        .iter()
        .filter(|message| message["id"] == 42)
        .collect();
    assert_eq!(terminal.len(), 1);
    assert_continuous_result(terminal[0], 4);
    assert_eq!(
        terminal[0]["result"]["structuredContent"]["results"],
        json!(items)
    );
    assert_eq!(messages.last(), terminal.first().copied());
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_continuous_stream_keeps_the_service_healthy() {
    let response = request(
        Method::POST,
        &json!({
            "jsonrpc": "2.0", "id": 43, "method": "tools/call",
            "params": {"name": "gen_continuous", "arguments": {"count": 20, "interval_ms": 100}, "_meta": {"progressToken": 43}}
        }).to_string(),
        None,
        "localhost:8787",
    ).await;
    let mut stream = response.into_body().into_data_stream();
    let mut parser = SseMessages::default();
    loop {
        let chunk = tokio::time::timeout(Duration::from_secs(3), stream.next())
            .await
            .expect("cancel progress timeout")
            .expect("progress chunk")
            .expect("body chunk");
        if parser
            .push(&chunk)
            .iter()
            .any(|message| message["params"]["progress"] == 1.0)
        {
            break;
        }
    }
    drop(stream);
    let (_, generated) = rpc("tools/call", json!({"name": "gen"})).await;
    assert_generated_string(&generated);
    let (_, generated) = rpc(
        "tools/call",
        json!({"name": "gen_continuous", "arguments": {"count": 1, "interval_ms": 100}}),
    )
    .await;
    assert_continuous_result(&generated, 1);
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
    for arguments in [
        json!({"count": 0}),
        json!({"count": 21}),
        json!({"count": -1}),
        json!({"count": 256}),
        json!({"count": 1.5}),
        json!({"count": null}),
        json!({"interval_ms": 99}),
        json!({"interval_ms": 2001}),
        json!({"interval_ms": -1}),
        json!({"interval_ms": 100.5}),
        json!({"interval_ms": null}),
        json!({"unknown": true}),
    ] {
        let (_, message) = rpc(
            "tools/call",
            json!({"name": "gen_continuous", "arguments": arguments}),
        )
        .await;
        assert!(
            message["error"]["code"] == -32602 || message["result"]["isError"] == true,
            "gen_continuous must reject invalid arguments: {message}"
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
