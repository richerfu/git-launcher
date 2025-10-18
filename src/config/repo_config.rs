#[derive(Debug, Clone)]
pub struct GitLauncherConfig {
    pub base_dir: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub max_depth: usize,
    pub max_concurrent_tasks: usize,
}

impl Default for GitLauncherConfig {
    fn default() -> Self {
        // let home_dir = std::env::home_dir().unwrap();
        Self {
            // base_dir: vec![home_dir.to_string_lossy().to_string()],
            base_dir: vec!["/Users/ranger/Desktop/project".to_string()],
            ignore_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "build".to_string(),
                "dist".to_string(),
            ],
            max_depth: 10,
            max_concurrent_tasks: 20,
        }
    }
}
