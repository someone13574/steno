use assets::Assets;
use components::clamp::clamp;
use gpui::prelude::*;
use gpui::{div, px, App, Application, Entity, FocusHandle, Window};
use text_view::TextView;
use theme::{BaseTheme, Theme};
use window::StenoWindow;

mod assets;
mod components;
mod cursor;
mod text_view;
mod theme;
mod titlebar;
mod window;

pub const APP_ID: &str = "com.github.someone13574.steno";

pub struct MainView {
    text_view: Entity<TextView>,
}

impl MainView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                text_view: TextView::new(focus_handle, cx),
            }
        })
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .justify_center()
            .items_center()
            .child(clamp(px(400.0), px(400.0), 2.0, self.text_view.clone()))
    }
}

fn main() {
    Application::new().with_assets(Assets).run({
        |cx| {
            cx.set_global(Theme::from(BaseTheme::default_dark()));

            StenoWindow::new(cx, |focus_handle, _window, cx| {
                MainView::new(focus_handle, cx)
            })
            .unwrap();
        }
    });
}
