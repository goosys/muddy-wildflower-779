use axum::{routing::get, Json, Router};

use crate::views::haikunator::GeneratorResponse;

async fn generate() -> Json<GeneratorResponse> {
    let generated = haikunator::Haikunator::default().haikunate();
    Json(GeneratorResponse::generate(&generated))
}

async fn generate_txt() -> String {
    haikunator::Haikunator::default().haikunate()
}

pub fn axum_router() -> Router {
    Router::new()
        .route("/api/gen", get(generate))
        .route("/api/gen.txt", get(generate_txt))
}
