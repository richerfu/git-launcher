use gpui::*;
use gpui_component::{input::{InputState, TextInput}};

pub struct SearchInput {
    state: InputState,
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
        }
    }
}

impl RenderOnce for SearchInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        TextInput::new(InputState::new(window))
            .render()
    }
}