use gpui::prelude::*;
use gpui::{div, App, AppContext, Entity, Render, Window};

use crate::components::button::{Button, ButtonTheme};
use crate::theme::ActiveTheme;
use crate::window::StenoWindow;

pub struct Titlebar<V: Render> {
    window: Entity<StenoWindow<V>>,
    buttons: Entity<TitlebarButtons>,
}

impl<V: Render> Titlebar<V> {
    pub fn new(window: Entity<StenoWindow<V>>, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                window,
                buttons: TitlebarButtons::new(cx),
            }
        })
    }
}

impl<V: Render> Render for Titlebar<V> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let parent = self.window.clone();

        div()
            .id("titlebar")
            .flex()
            .w_full()
            .items_center()
            .justify_between()
            .bg(cx.theme().csd_titlebar_background)
            .border_b(cx.theme().csd_titlebar_border_width)
            .border_color(cx.theme().csd_titlebar_border)
            .px(cx.theme().csd_titlebar_padding_x)
            .py(cx.theme().csd_titlebar_padding_y)
            .rounded_t(cx.theme().csd_corner_radius)
            .font_family(cx.theme().csd_font_family)
            .text_color(cx.theme().csd_foreground)
            .child(gpui::div())
            .child("Steno")
            .child(
                gpui::div()
                    .w_0()
                    .flex()
                    .justify_end()
                    .child(self.buttons.clone()),
            )
            .on_mouse_move(move |event, window, cx| {
                if event.dragging() {
                    parent.update(cx, |window, cx| {
                        window.active_csd_event = true;
                        cx.notify();
                    });

                    window.start_window_move();
                }
            })
            .on_click(|event, window, _cx| {
                if event.standard_click() && event.click_count() >= 2 {
                    window.zoom_window();
                }
            })
    }
}

struct TitlebarButtons {
    pub minimize: Entity<Button>,
    pub maximize: Entity<Button>,
    pub close: Entity<Button>,
}

impl TitlebarButtons {
    pub fn new(cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let theme_fn = |cx: &mut App| {
                ButtonTheme {
                    background: Some(cx.theme().csd_button_background),
                    background_hover: Some(cx.theme().csd_button_background_hovered),
                    icon: Some(cx.theme().csd_button_foreground),
                }
            };

            Self {
                minimize: Button::builder()
                    .svg_icon("minimize.svg")
                    .on_mouse_down(|_event, window, _cx| {
                        window.minimize_window();
                    })
                    .theme(theme_fn)
                    .build(cx),
                maximize: Button::builder()
                    .svg_icon("maximize.svg")
                    .on_mouse_down(|_event, window, _cx| {
                        window.zoom_window();
                    })
                    .theme(theme_fn)
                    .build(cx),
                close: Button::builder()
                    .svg_icon("close.svg")
                    .on_mouse_down(|_event, window, _cx| {
                        window.remove_window();
                    })
                    .theme(theme_fn)
                    .build(cx),
            }
        })
    }
}

impl Render for TitlebarButtons {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .gap(cx.theme().csd_button_gap)
            .child(self.minimize.clone())
            .child(self.maximize.clone())
            .child(self.close.clone())
    }
}
