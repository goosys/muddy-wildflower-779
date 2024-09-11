use axum::debug_handler;
use loco_rs::prelude::*;

use crate::views::haikunator::GeneratorResponse;

#[debug_handler]
async fn generate() -> Result<Response> {
    let generated = haikunator::Haikunator::default().haikunate();
    format::json(GeneratorResponse::generate(&generated))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/haikunator")
        .add("/generate", get(generate))
}
