use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ProgressNotificationParam},
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::never::NeverSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ErrorData, RoleServer, ServerHandler,
};
use serde::Deserialize;
use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle};

pub use crate::config::AccessConfig;
use crate::views::haikunator::GeneratorResponse;

pub const MAX_REQUEST_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub enum AccessPolicy {
    #[default]
    Public,
    Development,
}

impl AccessPolicy {
    /// Local access is enabled only by an explicit development binding.
    #[must_use]
    pub fn from_environment(value: Option<&str>) -> Self {
        if value == Some("development") {
            Self::Development
        } else {
            Self::Public
        }
    }
}

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

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct GenContinuousParams {
    /// Number of strings to generate, from 1 to 20. Defaults to 5.
    #[schemars(range(min = 1, max = 20))]
    count: u8,
    /// Time between generated strings in milliseconds, from 100 to 2000. Defaults to 500.
    #[schemars(range(min = 100, max = 2000))]
    interval_ms: u64,
}

impl Default for GenContinuousParams {
    fn default() -> Self {
        Self {
            count: 5,
            interval_ms: 500,
        }
    }
}

struct GenerationTask(JoinHandle<()>);

impl Drop for GenerationTask {
    fn drop(&mut self) {
        self.0.abort();
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

    #[tool(
        title = "Haikunator Generator Continuous Generation",
        description = "Generate a finite sequence of Heroku-like memorable random strings at fixed intervals. Each result has the same JSON shape as /api/gen: {\"name\":\"falling-disk-1736\"}. Supply a progress token to receive each result as a progress notification; the final result contains a results array.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn gen_continuous(
        &self,
        Parameters(params): Parameters<GenContinuousParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        if !(1..=20).contains(&params.count) || !(100..=2000).contains(&params.interval_ms) {
            return Err(ErrorData::invalid_params(
                "count must be between 1 and 20; interval_ms must be between 100 and 2000",
                None,
            ));
        }

        let progress_token = context.meta.get_progress_token();
        if let Some(token) = &progress_token {
            context
                .peer
                .notify_progress(
                    ProgressNotificationParam::new(token.clone(), 0.0)
                        .with_total(f64::from(params.count))
                        .with_message("Starting continuous generation".to_owned()),
                )
                .await
                .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        }

        let (sender, mut receiver) = mpsc::channel(1);
        let cancellation = context.ct.clone();
        let producer = tokio::spawn(async move {
            for _ in 0..params.count {
                tokio::select! {
                    () = cancellation.cancelled() => return,
                    () = tokio::time::sleep(Duration::from_millis(params.interval_ms)) => {},
                }
                let generated = super::haikunator::haikunate();
                let response = GeneratorResponse::generate(&generated);
                tokio::select! {
                    () = cancellation.cancelled() => return,
                    sent = sender.send(response) => {
                        if sent.is_err() {
                            return;
                        }
                    },
                }
            }
        });
        // Abort the producer even if the handler is dropped before it can observe cancellation.
        let _producer = GenerationTask(producer);
        let mut results = Vec::with_capacity(usize::from(params.count));
        for completed in 1..=params.count {
            let generated = tokio::select! {
                () = context.ct.cancelled() => {
                    return Err(ErrorData::internal_error("Continuous generation cancelled", None));
                },
                item = receiver.recv() => item,
            }
            .ok_or_else(|| ErrorData::internal_error("Continuous generation ended early", None))?;
            if let Some(token) = &progress_token {
                context
                    .peer
                    .notify_progress(
                        ProgressNotificationParam::new(token.clone(), f64::from(completed))
                            .with_total(f64::from(params.count))
                            .with_message(serde_json::json!(&generated).to_string()),
                    )
                    .await
                    .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
            }
            results.push(generated);
        }
        Ok(CallToolResult::structured(
            serde_json::json!({ "results": results }),
        ))
    }
}

#[expect(
    clippy::unused_async_trait_impl,
    reason = "rmcp generates async trait methods for tool discovery"
)]
#[tool_handler(
    name = "haikunator-generator",
    version = "0.1.0",
    instructions = "Haikunator Generator generates Heroku-like memorable random strings. Use gen for the same JSON response as /api/gen, or gen_continuous for a finite sequence with progress. Generated strings are not guaranteed to be unique."
)]
impl ServerHandler for HaikunatorGenerator {}

pub fn axum_router() -> Router {
    axum_router_with_access(AccessPolicy::Public, None)
}

pub fn axum_router_with_access(policy: AccessPolicy, request_authority: Option<&str>) -> Router {
    axum_router_with_config(policy, request_authority, &crate::config::PUBLIC_ACCESS)
}

pub fn axum_router_with_config(
    policy: AccessPolicy,
    request_authority: Option<&str>,
    access: &AccessConfig,
) -> Router {
    let preview = match policy {
        AccessPolicy::Public => {
            request_authority.and_then(|authority| access.preview_host(authority))
        }
        AccessPolicy::Development => None,
    };
    let (hosts, origins) = match policy {
        AccessPolicy::Public => {
            let mut hosts = access.public_hosts.to_vec();
            let mut origins = access
                .public_hosts
                .iter()
                .map(|host| format!("https://{host}:443"))
                .collect::<Vec<_>>();
            if let Some(host) = &preview {
                // Trust only this Worker's preview namespace, and bind the
                // inferred preview Origin to the current request's authority.
                // A different Host header must not inherit its permissions.
                hosts = vec![host.clone()];
                origins.push(format!("https://{host}:443"));
            }
            (hosts, origins)
        }
        AccessPolicy::Development => (
            ["localhost:8787", "127.0.0.1:8787"]
                .map(str::to_owned)
                .to_vec(),
            [
                "http://localhost:8787",
                "http://127.0.0.1:8787",
                "http://localhost:5153",
                "http://127.0.0.1:5153",
            ]
            .map(str::to_owned)
            .to_vec(),
        ),
    };
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(MAX_REQUEST_BYTES)
        .with_allowed_hosts(hosts)
        .with_allowed_origins(origins);
    let service = StreamableHttpService::new(
        || Ok(HaikunatorGenerator),
        NeverSessionManager::default().into(),
        config,
    );
    let router = Router::new().route_service("/mcp", service);
    if let Some(preview) = preview {
        let access = access.clone();
        // rmcp treats an allowed hostname without a port as allowing any port.
        // Keep a preview's actual Host bound to the validated request authority.
        router.layer(middleware::from_fn(move |request: Request, next: Next| {
            let mut hosts = request.headers().get_all(header::HOST).iter();
            let valid = hosts.next().is_none_or(|value| {
                value
                    .to_str()
                    .ok()
                    .and_then(|authority| access.preview_host(authority))
                    .is_some_and(|host| host == preview)
            }) && hosts.next().is_none();
            async move {
                if valid {
                    next.run(request).await
                } else {
                    (StatusCode::FORBIDDEN, "Forbidden: Invalid preview Host").into_response()
                }
            }
        }))
    } else {
        router
    }
}
