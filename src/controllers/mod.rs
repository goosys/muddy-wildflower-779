pub mod haikunator;
pub mod mcp;

pub fn axum_router() -> axum::Router {
    haikunator::axum_router().merge(mcp::axum_router())
}

pub fn axum_router_with_access(
    policy: mcp::AccessPolicy,
    request_authority: Option<&str>,
) -> axum::Router {
    haikunator::axum_router().merge(mcp::axum_router_with_access(policy, request_authority))
}

pub fn axum_router_with_config(
    policy: mcp::AccessPolicy,
    request_authority: Option<&str>,
    access: &mcp::AccessConfig,
) -> axum::Router {
    haikunator::axum_router().merge(mcp::axum_router_with_config(
        policy,
        request_authority,
        access,
    ))
}
