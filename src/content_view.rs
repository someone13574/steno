use gpui::prelude::*;
use gpui::{div, px, App, Entity, FocusHandle, Percentage, Window};

use crate::components::clamp::clamp;
use crate::counter::Counter;
use crate::text_view::TextView;

pub struct ContentView {
    text_view: Entity<TextView>,
    counter: Entity<Counter>,
}

impl ContentView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let text_view = TextView::new(focus_handle.clone(), cx);

            Self {
                text_view: text_view.clone(),
                counter: Counter::new(text_view, cx),
            }
        })
    }
}

impl Render for ContentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div().flex_1())
            .child(self.text_view.clone())
            .child(
                div().flex_1().flex().flex_col().justify_end().child(
                    clamp(px(80.0), px(40.0), self.counter.clone())
                        .vertical()
                        .position(Percentage(1.0))
                        .smoothing(10.0),
                ),
            )
    }
}
