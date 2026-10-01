use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
};
use futures_util::StreamExt;
use haikunator_worker::controllers::{
    axum_router, axum_router_with_config,
    mcp::{AccessConfig, AccessPolicy},
};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;

fn access_config() -> AccessConfig {
    AccessConfig::new(
        "https://generator.example.com",
        "https://sample-worker.example-account.workers.dev",
    )
    .expect("example deployment configuration")
}

async fn request(method: Method, body: &str, origin: Option<&str>, host: &str) -> Response {
    scoped_request(AccessPolicy::Development, None, method, body, origin, host).await
}

async fn scoped_request(
    policy: AccessPolicy,
    request_authority: Option<&str>,
    method: Method,
    body: &str,
    origin: Option<&str>,
    host: &str,
) -> Response {
    configured_request(
        &access_config(),
        policy,
        request_authority,
        method,
        body,
        origin,
        host,
    )
    .await
}

async fn configured_request(
    config: &AccessConfig,
    policy: AccessPolicy,
    request_authority: Option<&str>,
    method: Method,
    body: &str,
    origin: Option<&str>,
    host: &str,
) -> Response {
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
    axum_router_with_config(policy, request_authority, config)
        .oneshot(builder.body(Body::from(body.to_owned())).expect("request"))
        .await
        .expect("response")
}

async fn public_authority_request(uri: &str, hosts: &[&str], origin: Option<&str>) -> Response {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-11-25");
    for host in hosts {
        builder = builder.header(header::HOST, *host);
    }
    if let Some(origin) = origin {
        builder = builder.header(header::ORIGIN, origin);
    }
    let request = builder
        .body(Body::from(ping_body()))
        .expect("authority request");
    let authority = request
        .uri()
        .authority()
        .map(axum::http::uri::Authority::as_str)
        .or_else(|| request.headers().get(header::HOST)?.to_str().ok());
    axum_router_with_config(AccessPolicy::Public, authority, &access_config())
        .oneshot(request)
        .await
        .expect("authority response")
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
}

fn ping_body() -> String {
    json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}).to_string()
}

#[test]
fn access_configuration_requires_https_origin_urls() {
    for public_url in [
        "https://generator.example.com",
        "https://generator.example.com:443",
    ] {
        assert!(
            AccessConfig::new(
                public_url,
                "https://sample-worker.example-account.workers.dev"
            )
            .is_ok(),
            "valid public origin {public_url}"
        );
    }
    for public_url in [
        "http://generator.example.com",
        "generator.example.com",
        "https://user@generator.example.com",
        "https://user:password@generator.example.com",
        "https://generator.example.com/path",
        "https://generator.example.com?query=value",
        "https://generator.example.com#fragment",
        "https://generator.example.com:80",
        "https://generator.example.com:8443",
        "https://generator.example.com:",
        "https://",
    ] {
        assert!(
            AccessConfig::new(
                public_url,
                "https://sample-worker.example-account.workers.dev"
            )
            .is_err(),
            "invalid public origin {public_url}"
        );
    }
}

#[test]
fn access_configuration_requires_a_workers_dev_base_origin() {
    for workers_dev_url in [
        "https://sample-worker.example-account.workers.dev",
        "https://sample-worker.example-account.workers.dev:443",
    ] {
        assert!(
            AccessConfig::new("https://generator.example.com", workers_dev_url).is_ok(),
            "valid workers.dev origin {workers_dev_url}"
        );
    }
    for workers_dev_url in [
        "http://sample-worker.example-account.workers.dev",
        "https://sample-worker.example-account.workers.dev:8787",
        "https://user@sample-worker.example-account.workers.dev",
        "https://sample-worker.example-account.workers.dev/path",
        "https://sample-worker.example-account.workers.dev?query=value",
        "https://sample-worker.example-account.workers.dev#fragment",
        "https://sample-worker.workers.dev",
        "https://branch.sample-worker.example-account.workers.dev",
        "https://sample-worker.example-account.workers.dev.evil.example",
        "https://sample_worker.example-account.workers.dev",
        "https://-sample-worker.example-account.workers.dev",
        "https://sample-worker-.example-account.workers.dev",
        "https://sample-worker.example_account.workers.dev",
        "https://sample-worker.example-account.example.com",
    ] {
        assert!(
            AccessConfig::new("https://generator.example.com", workers_dev_url).is_err(),
            "invalid workers.dev origin {workers_dev_url}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn preview_prefix_limit_follows_the_configured_worker_name_length() {
    let body = ping_body();
    for (worker_name, maximum_prefix_length) in [("sample-worker", 49), ("tiny", 58)] {
        let config = AccessConfig::new(
            "https://generator.example.com",
            &format!("https://{worker_name}.example-account.workers.dev"),
        )
        .expect("worker configuration");
        for (prefix_length, status) in [
            (maximum_prefix_length, StatusCode::OK),
            (maximum_prefix_length + 1, StatusCode::FORBIDDEN),
        ] {
            let authority = format!(
                "{}-{worker_name}.example-account.workers.dev",
                "a".repeat(prefix_length)
            );
            let origin = format!("https://{authority}");
            assert_eq!(
                configured_request(
                    &config,
                    AccessPolicy::Public,
                    Some(&authority),
                    Method::POST,
                    &body,
                    Some(&origin),
                    &authority,
                )
                .await
                .status(),
                status,
                "worker {worker_name} with prefix length {prefix_length}"
            );
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn public_transport_allows_production_hosts_and_native_clients() {
    let body = ping_body();
    for host in [
        "generator.example.com",
        "sample-worker.example-account.workers.dev",
    ] {
        for origin in [None, Some(format!("https://{host}"))] {
            assert_eq!(
                scoped_request(
                    AccessPolicy::Public,
                    Some(host),
                    Method::POST,
                    &body,
                    origin.as_deref(),
                    host,
                )
                .await
                .status(),
                StatusCode::OK,
                "production host {host}, origin {origin:?}"
            );
        }
    }
    let response = axum_router()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mcp")
                .header(header::HOST, "localhost:8787")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .body(Body::from(body))
                .expect("default policy request"),
        )
        .await
        .expect("default policy response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "current_thread")]
async fn public_transport_rejects_development_hosts_and_origins() {
    let body = ping_body();
    let public_host = "generator.example.com";
    for host in ["localhost:8787", "127.0.0.1:8787"] {
        assert_eq!(
            scoped_request(
                AccessPolicy::Public,
                Some(host),
                Method::POST,
                &body,
                None,
                host,
            )
            .await
            .status(),
            StatusCode::FORBIDDEN,
            "development host {host}"
        );
    }
    for origin in [
        "http://localhost:8787",
        "http://127.0.0.1:8787",
        "http://localhost:5153",
        "http://127.0.0.1:5153",
        "https://untrusted.example",
        "null",
    ] {
        assert_eq!(
            scoped_request(
                AccessPolicy::Public,
                Some(public_host),
                Method::POST,
                &body,
                Some(origin),
                public_host,
            )
            .await
            .status(),
            StatusCode::FORBIDDEN,
            "development or untrusted origin {origin}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn development_transport_stays_local() {
    let body = ping_body();
    for host in ["localhost:8787", "127.0.0.1:8787"] {
        for origin in [
            None,
            Some("http://localhost:8787"),
            Some("http://127.0.0.1:8787"),
            Some("http://localhost:5153"),
            Some("http://127.0.0.1:5153"),
        ] {
            assert_eq!(
                request(Method::POST, &body, origin, host).await.status(),
                StatusCode::OK,
                "local host {host}, origin {origin:?}"
            );
        }
    }
    for host in [
        "generator.example.com",
        "sample-worker.example-account.workers.dev",
        "feature-sample-worker.example-account.workers.dev",
    ] {
        assert_eq!(
            scoped_request(
                AccessPolicy::Development,
                Some(host),
                Method::POST,
                &body,
                None,
                host,
            )
            .await
            .status(),
            StatusCode::FORBIDDEN,
            "non-local development host {host}"
        );
    }
    for origin in [
        "https://generator.example.com",
        "https://sample-worker.example-account.workers.dev",
        "https://feature-sample-worker.example-account.workers.dev",
    ] {
        assert_eq!(
            request(Method::POST, &body, Some(origin), "localhost:8787")
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "non-local development origin {origin}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn public_transport_automatically_allows_the_current_worker_preview() {
    let body = ping_body();
    let maximum_alias = format!(
        "{}-sample-worker.example-account.workers.dev",
        "a".repeat(49)
    );
    for authority in [
        "7ca1de02-sample-worker.example-account.workers.dev",
        "feature-mcp-sample-worker.example-account.workers.dev",
        "FEATURE-MCP-SAMPLE-WORKER.EXAMPLE-ACCOUNT.WORKERS.DEV",
        "feature-mcp-sample-worker.example-account.workers.dev:443",
        maximum_alias.as_str(),
    ] {
        let origin = format!("https://{authority}");
        for origin in [None, Some(origin.as_str())] {
            assert_eq!(
                scoped_request(
                    AccessPolicy::Public,
                    Some(authority),
                    Method::POST,
                    &body,
                    origin,
                    authority,
                )
                .await
                .status(),
                StatusCode::OK,
                "preview authority {authority}, origin {origin:?}"
            );
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn preview_origins_are_scoped_to_the_requested_preview() {
    let body = ping_body();
    let preview = "feature-mcp-sample-worker.example-account.workers.dev";
    for (authority, origin, status) in [
        (preview, "https://generator.example.com", StatusCode::OK),
        (
            preview,
            "https://sample-worker.example-account.workers.dev",
            StatusCode::OK,
        ),
        (
            preview,
            "https://other-sample-worker.example-account.workers.dev",
            StatusCode::FORBIDDEN,
        ),
        (
            "generator.example.com",
            "https://feature-mcp-sample-worker.example-account.workers.dev",
            StatusCode::FORBIDDEN,
        ),
        (
            preview,
            "http://feature-mcp-sample-worker.example-account.workers.dev",
            StatusCode::FORBIDDEN,
        ),
        (preview, "null", StatusCode::FORBIDDEN),
    ] {
        assert_eq!(
            scoped_request(
                AccessPolicy::Public,
                Some(authority),
                Method::POST,
                &body,
                Some(origin),
                authority,
            )
            .await
            .status(),
            status,
            "authority {authority}, origin {origin}"
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn public_transport_rejects_preview_lookalikes_and_invalid_authorities() {
    let body = ping_body();
    let overlong_alias = format!(
        "{}-sample-worker.example-account.workers.dev",
        "a".repeat(50)
    );
    for authority in [
        "feature-sample-worker.other-account.workers.dev",
        "feature-other-worker.example-account.workers.dev",
        "feature-sample-worker.example-account.workers.dev.evil.example",
        "feature.branch-sample-worker.example-account.workers.dev",
        "-sample-worker.example-account.workers.dev",
        "-feature-sample-worker.example-account.workers.dev",
        "feature--sample-worker.example-account.workers.dev",
        "feature_1-sample-worker.example-account.workers.dev",
        "feature-sample-worker.example-account.workers.dev:80",
        "feature-sample-worker.example-account.workers.dev:8787",
        "user@feature-sample-worker.example-account.workers.dev",
        overlong_alias.as_str(),
    ] {
        let response = scoped_request(
            AccessPolicy::Public,
            Some(authority),
            Method::POST,
            &body,
            None,
            authority,
        )
        .await;
        assert!(
            matches!(
                response.status(),
                StatusCode::FORBIDDEN | StatusCode::BAD_REQUEST
            ),
            "invalid preview authority {authority}: {}",
            response.status()
        );
    }
    assert_eq!(
        scoped_request(
            AccessPolicy::Public,
            None,
            Method::POST,
            &body,
            None,
            "feature-sample-worker.example-account.workers.dev",
        )
        .await
        .status(),
        StatusCode::FORBIDDEN,
        "a Host header alone must not expand a router without a request authority"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn preview_host_must_match_the_absolute_request_authority() {
    let preview = "feature-sample-worker.example-account.workers.dev";
    let preview_uri = format!("https://{preview}/mcp");
    assert_eq!(
        public_authority_request("/mcp", &[preview], None)
            .await
            .status(),
        StatusCode::OK,
        "a relative request URI uses its Host as the preview authority"
    );
    for host in [
        "localhost:8787",
        "generator.example.com",
        "sample-worker.example-account.workers.dev",
        "other-sample-worker.example-account.workers.dev",
        "feature-sample-worker.example-account.workers.dev:80",
        "feature-sample-worker.example-account.workers.dev:8787",
        "user@feature-sample-worker.example-account.workers.dev",
        "",
    ] {
        assert_eq!(
            public_authority_request(&preview_uri, &[host], None)
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "preview URI authority must not permit a mismatched Host {host}"
        );
    }
    for hosts in [
        &[preview][..],
        &["feature-sample-worker.example-account.workers.dev:443"][..],
        &["FEATURE-SAMPLE-WORKER.EXAMPLE-ACCOUNT.WORKERS.DEV"][..],
        &[][..],
    ] {
        assert_eq!(
            public_authority_request(&preview_uri, hosts, Some(&format!("https://{preview}")))
                .await
                .status(),
            StatusCode::OK,
            "matching preview Host or absolute URI without Host"
        );
    }
    for hosts in [
        &[preview, preview][..],
        &[preview, "other-sample-worker.example-account.workers.dev"][..],
        &["other-sample-worker.example-account.workers.dev", preview][..],
    ] {
        assert_eq!(
            public_authority_request(&preview_uri, hosts, None)
                .await
                .status(),
            StatusCode::FORBIDDEN,
            "preview requests must reject multiple Host headers: {hosts:?}"
        );
    }
    assert_eq!(
        public_authority_request("https://untrusted.example/mcp", &[preview], None)
            .await
            .status(),
        StatusCode::FORBIDDEN,
        "a preview Host cannot override an untrusted absolute URI authority"
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
