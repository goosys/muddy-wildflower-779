use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneratorResponse {
    pub name: String,
}

impl GeneratorResponse {
    #[must_use]
    pub fn generate(generated: &str) -> Self {
        Self {
            name: generated.to_owned(),
        }
    }
}
