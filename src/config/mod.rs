use anyhow::Ok;
use gpui::*;

mod repo_config;
mod ui_config;

pub use repo_config::*;
pub use ui_config::*;

#[derive(Debug, Clone)]
pub struct Config {
    pub repo_config: GitLauncherConfig,
    pub ui_config: GitLauncherUIConfig,
}

// init config with default or config file
pub fn init(cx: &mut App) -> Result<(), anyhow::Error> {
    let config = Config {
        repo_config: GitLauncherConfig::default(),
        ui_config: GitLauncherUIConfig::default(),
    };

    cx.set_global(config);

    Ok(())
}

impl Global for Config {}
