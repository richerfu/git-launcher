#[derive(Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub path: String,
    pub language: String,
    pub count: u32,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            name: String::from("test"),
            path: String::from("test"),
            language: String::from("test"),
            count: 1,
        }
    }
}
