use crate::repo::Repo;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub const ITEM_HEIGHT: f32 = 75.;

#[derive(Debug, Clone)]
pub struct RepoItem {
    data: Repo,
}

impl RepoItem {
    pub fn new(data: Repo) -> Self {
        Self { data }
    }
}

impl Render for RepoItem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .child(self.data.language.clone())
            .child(
                v_flex()
                    .child(div().child(self.data.name.clone()).text_size(px(16.)))
                    .child(
                        div()
                            .child(self.data.path.clone())
                            .text_size(px(14.))
                            .text_color(cx.theme().muted_foreground),
                    )
                    .flex_grow(),
            )
            .child(
                div()
                    .child(format!("{}", self.data.count))
                    .text_size(px(14.))
                    .text_color(cx.theme().muted_foreground),
            )
            .pt_2()
            .pb_2()
            .pl_4()
            .pr_4()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().list_hover))
    }
}
