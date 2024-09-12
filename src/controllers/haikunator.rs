use axum::debug_handler;
use loco_rs::prelude::*;

use crate::views::haikunator::GeneratorResponse;

#[debug_handler]
async fn generate() -> Result<Response> {
    let generated = haikunator::Haikunator::default().haikunate();
    format::json(GeneratorResponse::generate(&generated))
}

async fn generate_txt() -> Result<Response> {
    let generated = haikunator::Haikunator::default().haikunate();
    format::text(&generated)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/gen", get(generate))
        .add("/gen.txt", get(generate_txt))
}
