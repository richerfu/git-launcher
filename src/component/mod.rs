use crate::component::repo_list::ITEM_HEIGHT;
use crate::repo::Repo;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::divider::Divider;
use gpui_component::scroll::{Scrollbar, ScrollbarAxis};
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt, h_flex,
    input::{InputEvent, InputState, TextInput},
    v_flex,
};

mod repo_list;

const MAX_ITEM_COUNT: usize = 6;

pub struct GitLauncher {
    input: Entity<InputState>,
    result: Vec<Repo>,
    search: String,
    _sub: Vec<Subscription>,
}

impl EventEmitter<InputEvent> for GitLauncher {}

impl Focusable for GitLauncher {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

impl GitLauncher {
    pub fn view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        cx.new(|cx| Self::new(input, window, cx))
    }

    fn new(input: Entity<InputState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let window_handle = window.window_handle().clone();

        let _sub = vec![cx.subscribe(
            &input,
            move |this, _, event: &InputEvent, ctx| match event {
                InputEvent::Change => {
                    let text = this.input.read(ctx).value();
                    let mut height = if text.len() > 0 {
                        ITEM_HEIGHT * (this.result.len() + 1) as f32 + 60.
                    } else {
                        60.
                    };

                    if height > ITEM_HEIGHT * (MAX_ITEM_COUNT as f32) {
                        height = ITEM_HEIGHT * (MAX_ITEM_COUNT as f32);
                    }

                    let _ = window_handle.update(ctx, |_, window: &mut Window, _| {
                        window.resize(size(px(600.), px(height)));
                    });
                    this.search = text.to_string().clone();
                    this.result.push(Repo {
                        name: format!("Git Launcher item #{}", this.result.len() + 1),
                        ..Default::default()
                    });
                }
                InputEvent::Blur => {
                    ctx.hide();
                }
                _ => {}
            },
        )];

        Self {
            input,
            _sub,
            result: vec![],
            search: String::new(),
        }
    }
}

impl Render for GitLauncher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .v_flex()
            .child(
                TextInput::new(&self.input)
                    .bordered(false)
                    .w_full()
                    .large()
                    .text_size(px(18.))
                    .mt_2()
                    .mb_1()
                    .prefix(
                        Icon::new(IconName::Search)
                            .size_5()
                            .text_color(cx.theme().secondary_foreground),
                    )
                    .when(self.search.len() > 0, |this| {
                        this.suffix(
                            Icon::new(IconName::CircleX)
                                .size_4()
                                .text_color(cx.theme().muted_foreground)
                                .cursor_pointer(),
                        )
                    }),
            )
            .when(self.search.len() > 0, |this| {
                this.child(Divider::horizontal())
                    .child(
                        v_flex()
                            .children(
                                self.result
                                    .iter()
                                    .map(|repo| cx.new(|_| repo_list::RepoItem::new(repo.clone()))),
                            )
                            .mt_1()
                            .pb_1()
                            .when(self.result.len() > MAX_ITEM_COUNT, |this| {
                                this.pb(px(ITEM_HEIGHT))
                            })
                            .scrollable(Axis::Vertical),
                    )
                    .max_h(px(ITEM_HEIGHT * (MAX_ITEM_COUNT as f32)))
            })
    }
}
