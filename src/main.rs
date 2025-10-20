use crate::{
    component::GitLauncher,
    config::{Config, REPO_PATH},
    repo::{GitProjectFinder, Repo, RepoState},
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::*;
use gpui_component::Root;
use std::{fs, path::Path, sync::mpsc};
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
}

impl AppState {
    fn new() -> Self {
        let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
        Self {
            hot_key_manager: manager,
            window_handle: None,
        }
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
        config::init(cx).expect("failed to init config");
        repo::init(cx).expect("failed to init repo");

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

        cx.on_action(move |_: &ShowWindow, cx: &mut App| {
            let app_state = GLOBAL_APP_STATE.read().unwrap();
            app_state.show_window(cx);
        });

        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        cx.activate(true);

        cx.spawn(async move |cx| {
            let config = cx
                .read_global(|state: &Config, _: &App| state.repo_config.clone())
                .unwrap();

            let content = fs::read_to_string(REPO_PATH.clone()).unwrap();

            if content.is_empty() {
                GLOBAL_RUNTIME.block_on(async move {
                    let repo_finder = GitProjectFinder::builder(config.clone()).build();

                    for dir in config.base_dir.clone() {
                        let repos = repo_finder
                            .find_git_projects(Path::new(&dir.clone()))
                            .await
                            .unwrap();

                        cx.update_global(|state: &mut RepoState, _: &mut App| {
                            let mut repo_state = state.repos.write().unwrap();
                            for repo in repos {
                                repo_state.push(Repo {
                                    name: repo.folder_name.clone(),
                                    path: repo.full_path.to_string_lossy().to_string().clone(),
                                    language: String::from("unknown"),
                                    count: 0,
                                });
                            }
                        })
                        .unwrap();
                    }

                    let repo_state = cx
                        .read_global(|state: &RepoState, _: &App| state.repos.clone())
                        .unwrap();

                    fs::write(
                        REPO_PATH.clone(),
                        serde_json::to_string(&repo_state).unwrap(),
                    )
                    .unwrap();
                });
            } else {
                let repo_state = fs::read_to_string(REPO_PATH.clone()).unwrap();
                let repos: Vec<Repo> = serde_json::from_str(&repo_state).unwrap();
                cx.update_global(|state: &mut RepoState, _: &mut App| {
                    let mut repo_state = state.repos.write().unwrap();
                    for repo in repos {
                        repo_state.push(repo);
                    }
                })
                .unwrap();

                let _ = cx.spawn(async move |cx| {
                    GLOBAL_RUNTIME.block_on(async move {
                        let repo_finder = GitProjectFinder::builder(config.clone()).build();

                        for dir in config.base_dir.clone() {
                            let repos = repo_finder
                                .find_git_projects(Path::new(&dir.clone()))
                                .await
                                .unwrap();

                            cx.update_global(|state: &mut RepoState, _: &mut App| {
                                let mut repo_state = state.repos.write().unwrap();
                                for repo in repos {
                                    repo_state.push(Repo {
                                        name: repo.folder_name.clone(),
                                        path: repo.full_path.to_string_lossy().to_string().clone(),
                                        language: String::from("unknown"),
                                        count: 0,
                                    });
                                }
                            })
                            .unwrap();
                        }
                    })
                });
            }

            Ok::<_, anyhow::Error>(())
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
            let config = cx
                .read_global(|state: &Config, _: &App| state.ui_config)
                .unwrap();
            let (_, window_bounds) = cx.update(|cx| {
                let config = cx.global::<Config>();
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
                    width: px(config.width),
                    height: px(config.height),
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
                .update(cx, |_, window, _| {
                    window.set_window_title("Git Launcher");
                    window.activate_window();
                })
                .expect("failed to update window");

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
