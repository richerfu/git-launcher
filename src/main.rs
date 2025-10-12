use crate::component::GitLauncher;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use gpui::*;
use gpui_component::Root;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::spawn;
use std::time::Duration;

mod assets;
mod component;
mod config;
mod repo;

actions!(git_launcher, [Quit, ShowWindow]);

#[derive(Clone)]
struct AppState {
    hot_key_manager: Arc<GlobalHotKeyManager>,
    window_handle: Arc<Mutex<Option<WindowHandle<Root>>>>,
}

impl AppState {
    fn new() -> Self {
        let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
        Self {
            hot_key_manager: Arc::new(manager),
            window_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn register_global_hotkeys(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let hotkey = HotKey::new(Some(Modifiers::ALT), Code::KeyP);
        self.hot_key_manager.register(hotkey)?;
        Ok(())
    }

    fn set_window_handle(&self, handle: WindowHandle<Root>) {
        *self.window_handle.lock().unwrap() = Some(handle);
    }

    fn show_window(&self, app: &mut App) {
        if let Some(handle) = self.window_handle.lock().unwrap().as_ref() {
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

    let mut app_state = AppState::new();
    app_state
        .register_global_hotkeys()
        .expect("Failed to register global hotkeys");

    app.run(move |cx| {
        let (tx, rx) = mpsc::channel::<()>();

        spawn(move || {
            loop {
                match GlobalHotKeyEvent::receiver().try_recv() {
                    Ok(event) => {
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

        let app_state_clone = app_state.clone();
        cx.on_action(move |_: &ShowWindow, cx: &mut App| {
            app_state_clone.show_window(cx);
        });

        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        cx.activate(true);

        let app_state_for_hotkey = app_state.clone();
        cx.spawn(async move |mut cx| {
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

        let app_state_for_window = app_state.clone();
        cx.spawn(async move |cx| {
            let (window_size, window_bounds) = cx.update(|cx| {
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

            app_state_for_window.set_window_handle(window.clone());

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
