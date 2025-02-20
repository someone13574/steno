use gpui::prelude::*;
use gpui::{div, px, transparent_black, App, Entity, MouseButton, MouseDownEvent, Rgba, Window};

use crate::theme::ActiveTheme;

type MouseDownListener = dyn Fn(&MouseDownEvent, &mut Window, &mut App);

pub struct Button {
    hovered: bool,
    svg: Option<&'static str>,
    mouse_down_listener: Option<Box<MouseDownListener>>,
    theme_fn: fn(&mut App) -> ButtonTheme,
}

impl Button {
    pub fn builder() -> Self {
        Button {
            hovered: false,
            svg: None,
            mouse_down_listener: None,
            theme_fn: |_cx| ButtonTheme::default(),
        }
    }

    pub fn svg_icon(mut self, icon_path: &'static str) -> Self {
        self.svg = Some(icon_path);
        self
    }

    pub fn theme(mut self, f: fn(&mut App) -> ButtonTheme) -> Self {
        self.theme_fn = f;
        self
    }

    pub fn on_mouse_down(
        mut self,
        listener: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.mouse_down_listener = Some(Box::new(listener));
        self
    }

    pub fn build(self, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| self)
    }
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ButtonTheme {
            background,
            background_hover,
            icon,
        } = (self.theme_fn)(cx);

        div()
            .id("button")
            .size_6()
            .p(px(4.0))
            .rounded_full()
            .bg(background.unwrap_or(transparent_black().into()))
            .when_some(self.svg, |div, svg| {
                div.child(
                    gpui::svg()
                        .path(svg)
                        .text_color(icon.unwrap_or(cx.theme().base.foreground))
                        .size_full(),
                )
            })
            .when(self.hovered, |div| {
                div.bg(background_hover.unwrap_or(cx.theme().base.hover_background))
            })
            .on_hover(cx.listener(|this, hovered, _window, cx| {
                this.hovered = *hovered;
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event, window, cx| {
                    if let Some(listener) = &this.mouse_down_listener {
                        listener(event, window, cx)
                    }
                }),
            )
    }
}

#[derive(Default)]
pub struct ButtonTheme {
    pub background: Option<Rgba>,
    pub background_hover: Option<Rgba>,
    pub icon: Option<Rgba>,
}
