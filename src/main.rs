use crate::{
    component::GitLauncher,
    repo::{GitProjectFinder, LanguageAnalyzer, Repo},
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::*;
use gpui_component::Root;
use std::{path::Path, sync::mpsc};
use std::{sync::LazyLock, time::Duration};
use std::{sync::RwLock, thread::spawn};
use tokio::runtime::Runtime;

mod assets;
mod component;
mod config;
mod repo;
mod system;

actions!(git_launcher, [Quit, ShowWindow]);

pub(crate) static GLOBAL_APP_STATE: LazyLock<RwLock<AppState>> =
    LazyLock::new(|| RwLock::new(AppState::new()));

pub(crate) static GLOBAL_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

struct AppState {
    hot_key_manager: GlobalHotKeyManager,
    window_handle: Option<WindowHandle<Root>>,
    repo_finder: GitProjectFinder,
    repos: Vec<Repo>,
}

impl AppState {
    fn new() -> Self {
        let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");

        let repo_finder = GitProjectFinder::builder()
            .ignore_dirs(["node_modules", "target", ".git", "build", "dist"])
            .ignore_dir("vendor")
            .ignore_dir("cache")
            .ignore_dir("tmp")
            .max_depth(6)
            .max_concurrent_tasks(15)
            .build();

        Self {
            hot_key_manager: manager,
            window_handle: None,
            repo_finder: repo_finder,
            repos: Vec::new(),
        }
    }

    pub async fn fresh_repos(
        &mut self,
        root_path: impl AsRef<Path>,
    ) -> anyhow::Result<(), anyhow::Error> {
        let ret = self.repo_finder.find_git_projects(root_path).await?;
        for repo in ret {
            let mut repo = Repo {
                name: repo.folder_name.clone(),
                path: repo.full_path.to_string_lossy().to_string().clone(),
                language: String::from("unknown"),
                count: 0,
            };
            // let stats = LanguageAnalyzer::new(repo.path.as_str()).language().await?;
            // repo.language = stats.0;

            self.repos.push(repo);
        }
        Ok(())
    }

    fn register_global_hotkeys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyP);
        self.hot_key_manager.register(hotkey)?;
        Ok(())
    }

    fn set_window_handle(&mut self, handle: WindowHandle<Root>) {
        self.window_handle = Some(handle);
    }

    fn show_window(&self, app: &mut App) {
        if let Some(handle) = self.window_handle.as_ref() {
            handle
                .update(app, |root, window, cx| {
                    let git_launcher = root.view().clone().downcast::<GitLauncher>().unwrap();
                    cx.activate(true);
                    window.activate_window();
                    cx.focus_view(&git_launcher, window);
                })
                .ok();
        }
    }
}

fn main() {
    let app = Application::new().with_assets(assets::Assets);

    {
        let mut app_state = GLOBAL_APP_STATE.write().unwrap();
        app_state
            .register_global_hotkeys()
            .expect("Failed to register global hotkeys");
    }

    app.run(move |cx| {
        let (tx, rx) = mpsc::channel::<()>();

        spawn(move || {
            loop {
                match GlobalHotKeyEvent::receiver().try_recv() {
                    Ok(_) => {
                        if let Err(_) = tx.send(()) {
                            break;
                        }
                    }
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
        });

        gpui_component::init(cx);
        let config = config::init(cx).expect("failed to init config");

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.on_action(move |_: &ShowWindow, cx: &mut App| {
            let app_state = GLOBAL_APP_STATE.read().unwrap();
            app_state.show_window(cx);
        });

        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        cx.activate(true);

        cx.spawn(async move |_| {
            GLOBAL_RUNTIME.block_on(async move {
                let mut app_state = GLOBAL_APP_STATE.write().unwrap();
                app_state
                    .fresh_repos(Path::new("/Users/ranger/Desktop/project"))
                    .await
                    .expect("failed to fresh repos");
            });
        })
        .detach();

        cx.spawn(async move |cx| {
            loop {
                match rx.try_recv() {
                    Ok(_) => {
                        cx.update(|cx| {
                            cx.dispatch_action(&ShowWindow);
                        })
                        .ok();
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }

                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
            }
        })
        .detach();

        cx.spawn(async move |cx| {
            let (_, window_bounds) = cx.update(|cx| {
                let mut window_size = size(px(config.ui_config.width), px(config.ui_config.height));

                if let Some(display) = cx.primary_display() {
                    let display_size = display.bounds().size;
                    let display_origin = display.bounds().origin;

                    window_size.width = window_size.width.min(display_size.width * 0.85);
                    window_size.height = window_size.height.min(display_size.height * 0.85);

                    let center_x =
                        display_origin.x + (display_size.width - window_size.width) / 2.0;
                    let center_y =
                        display_origin.y + (display_size.height - window_size.height) / 2.0;

                    let centered_origin = point(center_x, center_y);
                    let centered_bounds = Bounds::new(centered_origin, window_size);

                    (window_size, centered_bounds)
                } else {
                    let default_bounds = Bounds::centered(None, window_size, cx);
                    (window_size, default_bounds)
                }
            })?;

            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                window_min_size: Some(gpui::Size {
                    width: px(config.ui_config.width),
                    height: px(config.ui_config.height),
                }),
                titlebar: None,
                kind: WindowKind::Normal,
                #[cfg(target_os = "linux")]
                window_background: gpui::WindowBackgroundAppearance::Transparent,
                #[cfg(target_os = "linux")]
                window_decorations: Some(gpui::WindowDecorations::Client),
                ..Default::default()
            };

            let window = cx
                .open_window(options, |window, cx| {
                    let view = component::GitLauncher::view(window, cx);
                    cx.new(|cx| Root::new(view.into(), window, cx))
                })
                .expect("failed to open window");

            let mut app_state = GLOBAL_APP_STATE.write().unwrap();
            app_state.set_window_handle(window.clone());

            window
                .update(cx, |_, window, cx| {
                    window.set_window_title("Git Launcher");
                    window.activate_window();
                })
                .expect("failed to update window");

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
