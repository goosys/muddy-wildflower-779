use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneratorResponse {
    pub name: String,
}

impl GeneratorResponse {
    #[must_use]
    pub fn generate(generated: &String) -> Self {
        Self {
            name: generated.to_string(),
        }
    }
}
