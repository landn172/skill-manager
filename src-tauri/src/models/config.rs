use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub skillsmp_api_key: Option<String>,
    pub theme: Option<String>,
    pub default_scope: Option<String>,
}

impl AppConfig {
    /// Mask the API key for display (show only last 4 chars)
    pub fn masked_api_key(&self) -> Option<String> {
        self.skillsmp_api_key.as_ref().map(|key| {
            if key.len() > 4 {
                format!("{}...{}", &key[..4], &key[key.len() - 4..])
            } else {
                "****".to_string()
            }
        })
    }
}
