use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GitLauncherUIConfig {
    pub width: f32,
    pub height: f32,
}

impl Default for GitLauncherUIConfig {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 60.0,
        }
    }
}
