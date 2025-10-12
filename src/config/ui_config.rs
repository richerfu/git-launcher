#[derive(Debug, Clone, Copy)]
pub struct GitLauncherUIConfig {
    width: f32,
    height: f32,
    max_scroll_height: f32,
}

impl Default for GitLauncherUIConfig {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 40.0,
            max_scroll_height: 600.0,
        }
    }
}