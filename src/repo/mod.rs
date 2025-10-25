mod language;
mod search_repo;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    sync::{Arc, RwLock},
};

use gpui::{App, Global};
pub use language::*;
pub use search_repo::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub count: u32,
}

impl Hash for Repo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoState {
    pub repos: Arc<RwLock<HashSet<Repo>>>,
}

impl Global for RepoState {}

/// init repo state
pub fn init(cx: &mut App) -> Result<(), anyhow::Error> {
    cx.set_global(RepoState {
        repos: Arc::new(RwLock::new(HashSet::new())),
    });
    Ok(())
}
