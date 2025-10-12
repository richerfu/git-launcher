#[derive(Debug, Clone)]
pub struct GitLauncherConfig {
    base_dir: Vec<String>,
    ignore_dirs: Vec<String>,
}

impl Default for GitLauncherConfig {
    fn default() -> Self {
        let home_dir = std::env::home_dir().unwrap();
        Self {
            base_dir: vec![home_dir.to_string_lossy().to_string()],
            ignore_dirs: vec![],
        }
    }
}
