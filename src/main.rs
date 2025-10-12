use gpui::*;
use gpui_component::Root;

mod assets;
mod component;
mod config;
mod repo;

fn main() {
    let app = Application::new().with_assets(assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);
        let config = config::init(cx).expect("failed to init config");

        cx.activate(true);

        let mut window_size = size(px(config.ui_config.width), px(config.ui_config.height));
        if let Some(display) = cx.primary_display() {
            let display_size = display.bounds().size;
            window_size.width = window_size.width.min(display_size.width * 0.85);
            window_size.height = window_size.height.min(display_size.height * 0.85);
        }
        let window_bounds = Bounds::centered(None, window_size, cx);

        cx.spawn(async move |cx| {
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

            window
                .update(cx, |_, window, _| {
                    window.activate_window();
                    window.set_window_title("Git Launcher");
                })
                .expect("failed to update window");

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
