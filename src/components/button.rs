use gpui::prelude::*;
use gpui::{
    div, px, rgba, transparent_black, white, App, Entity, MouseButton, MouseDownEvent, Window,
};

type MouseDownListener = dyn Fn(&MouseDownEvent, &mut Window, &mut App);

pub struct Button {
    hovered: bool,
    svg: Option<&'static str>,
    mouse_down_listener: Option<Box<MouseDownListener>>,
}

impl Button {
    pub fn builder() -> Self {
        Button {
            hovered: false,
            svg: None,
            mouse_down_listener: None,
        }
    }

    pub fn svg_icon(mut self, icon_path: &'static str) -> Self {
        self.svg = Some(icon_path);
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
        div()
            .id("button")
            .size_6()
            .p(px(4.0))
            .rounded_full()
            .bg(transparent_black())
            .when_some(self.svg, |div, svg| {
                div.child(gpui::svg().path(svg).text_color(white()).size_full())
            })
            .when(self.hovered, |div| div.bg(rgba(0x303030ff)))
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
