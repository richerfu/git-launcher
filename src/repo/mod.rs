mod language;
mod search_repo;

use std::sync::{Arc, RwLock};

use gpui::{App, Global};
pub use language::*;
pub use search_repo::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoState {
    pub repos: Arc<RwLock<Vec<Repo>>>,
}

impl Global for RepoState {}

/// init repo state
pub fn init(cx: &mut App) -> Result<(), anyhow::Error> {
    cx.set_global(RepoState {
        repos: Arc::new(RwLock::new(Vec::new())),
    });
    Ok(())
}
