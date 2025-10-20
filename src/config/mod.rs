use std::{fs, path::PathBuf, sync::LazyLock};

use anyhow::Ok;
use gpui::*;
use serde::{Deserialize, Serialize};

mod editor_config;
mod repo_config;
mod ui_config;

pub use editor_config::*;
pub use repo_config::*;
pub use ui_config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo_config: GitLauncherConfig,
    pub ui_config: GitLauncherUIConfig,
    pub editor_config: GitLauncherEditorConfig,
}

pub(crate) static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = std::env::home_dir().unwrap();
    home.join(".git-launcher")
});

pub(crate) static SETTING_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_PATH.join("config.toml"));

pub(crate) static REPO_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_PATH.join("repo.json"));

// init config with default or config file
pub fn init(cx: &mut App) -> Result<(), anyhow::Error> {
    fs::create_dir_all(CONFIG_PATH.clone())?;

    let mut config = Config {
        repo_config: GitLauncherConfig::default(),
        ui_config: GitLauncherUIConfig::default(),
        editor_config: GitLauncherEditorConfig::default(),
    };

    if SETTING_PATH.exists() {
        let setting = toml::from_str(&fs::read_to_string(SETTING_PATH.clone())?)?;
        config = setting;
    } else {
        fs::write(SETTING_PATH.clone(), toml::to_string(&config)?)?;
    }

    if !REPO_PATH.exists() {
        fs::write(REPO_PATH.clone(), "")?;
    }

    cx.set_global(config);

    Ok(())
}

impl Global for Config {}
