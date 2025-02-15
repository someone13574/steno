use assets::Assets;
use gpui::prelude::*;
use gpui::{div, white, App, Application, Entity, Window};
use theme::{BaseTheme, Theme};
use window::TapperWindow;

mod assets;
mod components;
mod theme;
mod titlebar;
mod window;

pub const APP_ID: &str = "com.github.someone13574.tapper";

pub struct MainView {}

impl MainView {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| Self {})
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .text_2xl()
            .text_color(white())
            .font_family("Sans")
            .child("Hello World")
    }
}

fn main() {
    Application::new().with_assets(Assets).run({
        |cx| {
            cx.set_global(Theme::from(BaseTheme::default_dark()));

            TapperWindow::new(cx, |_window, cx| MainView::new(cx)).unwrap();
        }
    });
}
