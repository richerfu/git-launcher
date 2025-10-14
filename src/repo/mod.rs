mod language;
mod search_repo;

pub use language::*;
pub use search_repo::*;

#[derive(Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub count: u32,
}
