#![cfg_attr(target_family = "wasm", no_main)]

#[cfg(not(target_family = "wasm"))]
use assets::Assets;
use components::clamp::clamp;
use content_view::ContentView;
use dictionary::Dictionary;
use gpui::prelude::*;
#[cfg(target_family = "wasm")]
use gpui::WindowOptions;
use gpui::{div, px, App, Entity, FocusHandle, Window};
use gpui_platform::application;
use theme::{ActiveTheme, BaseTheme, Theme};
#[cfg(not(target_family = "wasm"))]
use window::StenoWindow;

#[cfg(not(target_family = "wasm"))]
mod assets;
pub mod components;
mod content_view;
mod counter;
mod cursor;
mod dictionary;
mod text_view;
mod theme;
#[cfg(not(target_family = "wasm"))]
mod titlebar;
#[cfg(not(target_family = "wasm"))]
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
            .bg(cx.theme().window_background)
            .p(cx.theme().csd_corner_radius)
            .child(clamp(px(1600.0), px(300.0), self.content_view.clone()))
    }
}

fn init_globals(cx: &mut App) {
    cx.set_global(Theme::from(BaseTheme::default_dark()));
    Dictionary::new("en", 250, true).set_global(cx);
}

#[cfg(target_family = "wasm")]
fn run_app_web() {
    application().run(|cx: &mut App| {
        init_globals(cx);

        cx.open_window(WindowOptions::default(), |window, cx| {
            Theme::default_light().set_light(window, cx);
            Theme::default_dark().set_dark(window, cx);

            let focus_handle = cx.focus_handle();
            MainView::new(focus_handle, cx)
        })
        .expect("failed to open web steno window");
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn run_app_native() {
    application().with_assets(Assets).run({
        |cx| {
            init_globals(cx);

            StenoWindow::new(cx, |focus_handle, window, cx| {
                Theme::default_light().set_light(window, cx);
                Theme::default_dark().set_dark(window, cx);

                MainView::new(focus_handle, cx)
            })
            .unwrap();
        }
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_app_native();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_app_web();
}
