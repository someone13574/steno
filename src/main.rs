use assets::Assets;
use components::clamp::clamp;
use content_view::ContentView;
use dictionary::Dictionary;
use gpui::prelude::*;
use gpui::{div, px, App, Application, Entity, FocusHandle, Window};
use theme::{ActiveTheme, BaseTheme, Theme};
use window::StenoWindow;

mod assets;
pub mod components;
mod content_view;
mod counter;
mod cursor;
mod dictionary;
mod text_view;
mod theme;
mod titlebar;
mod window;

pub const APP_ID: &str = "com.github.someone13574.steno";

pub struct MainView {
    content_view: Entity<ContentView>,
}

impl MainView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                content_view: ContentView::new(focus_handle, cx),
            }
        })
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .justify_center()
            .items_center()
            .p(cx.theme().csd_corner_radius)
            .child(clamp(px(1600.0), px(300.0), self.content_view.clone()))
    }
}

fn main() {
    Application::new().with_assets(Assets).run({
        |cx| {
            cx.set_global(Theme::from(BaseTheme::default_dark()));
            Dictionary::new("en", 250, true).set_global(cx);

            StenoWindow::new(cx, |focus_handle, window, cx| {
                Theme::default_light().set_light(window, cx);
                Theme::default_dark().set_dark(window, cx);

                MainView::new(focus_handle, cx)
            })
            .unwrap();
        }
    });
}
