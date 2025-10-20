use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLauncherEditorConfig {
    pub editor: String,
}

impl Default for GitLauncherEditorConfig {
    fn default() -> Self {
        Self {
            editor: "/Applications/Visual Studio Code.app".to_string(),
        }
    }
}
