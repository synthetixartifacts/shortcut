use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub preferred_language: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            preferred_language: "en".to_string(),
        }
    }
}
