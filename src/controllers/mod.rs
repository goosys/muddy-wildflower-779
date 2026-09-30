pub mod haikunator;
pub mod mcp;

pub fn axum_router() -> axum::Router {
    haikunator::axum_router().merge(mcp::axum_router())
}
