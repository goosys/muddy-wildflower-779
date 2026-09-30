use axum::Router;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::CallToolResult,
    schemars, tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::never::NeverSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ServerHandler,
};
use serde::Deserialize;

use crate::views::haikunator::GeneratorResponse;

#[derive(Debug, Default, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct GenParams {}

impl<'de> Deserialize<'de> for GenParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arguments = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        if arguments.is_empty() {
            Ok(Self {})
        } else {
            Err(serde::de::Error::custom("gen does not accept arguments"))
        }
    }
}

#[derive(Clone)]
struct HaikunatorGenerator;

#[tool_router]
impl HaikunatorGenerator {
    #[tool(
        title = "Haikunator Generator",
        description = "Generate a Heroku-like memorable random string. Returns the same JSON object as /api/gen: {\"name\":\"falling-disk-1736\"}.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    #[expect(clippy::unused_self, reason = "rmcp tool handlers require a receiver")]
    fn gen(&self, Parameters(_): Parameters<GenParams>) -> CallToolResult {
        let generated = super::haikunator::haikunate();
        CallToolResult::structured(serde_json::json!(GeneratorResponse::generate(&generated)))
    }
}

#[expect(
    clippy::unused_async_trait_impl,
    reason = "rmcp generates async trait methods for tool discovery"
)]
#[tool_handler(
    name = "haikunator-generator",
    version = "0.1.0",
    instructions = "Haikunator Generator generates Heroku-like memorable random strings. Use gen for the same JSON response as /api/gen. Generated strings are not guaranteed to be unique."
)]
impl ServerHandler for HaikunatorGenerator {}

pub fn axum_router() -> Router {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(16 * 1024)
        .with_allowed_hosts([
            "haikunator-generator.goosysapp.net",
            "haikunator-generator.goosys.workers.dev",
            "localhost:8787",
            "127.0.0.1:8787",
        ])
        .with_allowed_origins([
            "https://haikunator-generator.goosysapp.net:443",
            "https://haikunator-generator.goosys.workers.dev:443",
            "http://localhost:8787",
            "http://127.0.0.1:8787",
            "http://localhost:5153",
            "http://127.0.0.1:5153",
        ]);
    let service = StreamableHttpService::new(
        || Ok(HaikunatorGenerator),
        NeverSessionManager::default().into(),
        config,
    );
    Router::new().route_service("/mcp", service)
}
