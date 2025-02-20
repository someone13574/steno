use std::f32::consts::PI;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    fill, px, size, App, Bounds, Edges, ElementId, Entity, EventEmitter, GlobalElementId, Hsla,
    IsZero, LayoutId, PaintQuad, Pixels, Point, Position, Style, Window,
};

use crate::text_view::TextView;
use crate::theme::ActiveTheme;

impl EventEmitter<Cursor> for TextView {}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Cursor {
    pub line_height: Pixels,
    pub target_position: Point<Pixels>,
    pub text_origin: Point<Pixels>,
}

impl Cursor {
    pub fn new(cx: &mut Context<TextView>) -> Entity<Self> {
        let text_view = cx.entity();

        cx.new(|cx| {
            cx.subscribe(&text_view, |cursor: &mut Self, _, new_cursor, cx| {
                *cursor = *new_cursor;

                cx.notify();
            })
            .detach();

            Self::default()
        })
    }
}

struct CursorState {
    position: Point<Pixels>,
    last_frame: Instant,
    idle_time: Instant,
}

impl IntoElement for Cursor {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Cursor {
    type PrepaintState = PaintQuad;
    type RequestLayoutState = Duration;

    fn id(&self) -> Option<ElementId> {
        Some("cursor".into())
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_element_state(id.unwrap(), |state, window| {
            let mut state = state.unwrap_or(CursorState {
                position: self.target_position,
                last_frame: Instant::now(),
                idle_time: Instant::now(),
            });

            let position = state.position + self.text_origin;
            let style = Style {
                position: Position::Absolute,
                size: size(px(self.line_height.0 / 3.0).into(), px(2.0).into()),
                inset: Edges {
                    left: position.x.into(),
                    top: position.y.into(),
                    ..Default::default()
                },
                ..Default::default()
            };

            let magnitude = (state.position - self.target_position).magnitude();
            if state.position.is_zero() || magnitude > 100.0 {
                state.position = self.target_position;
            } else if (state.position - self.target_position).magnitude() > 0.1 {
                let mix = (state.last_frame.elapsed().as_secs_f64() * 30.0).clamp(0.0, 1.0) as f32;
                state.position = state.position * (1.0 - mix) + self.target_position * mix;
            }

            if magnitude > 0.25 {
                state.idle_time = Instant::now();
            }

            state.last_frame = Instant::now();
            window.request_animation_frame();

            (
                (
                    window.request_layout(style, [], cx),
                    state.idle_time.elapsed(),
                ),
                state,
            )
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        idle_time: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let pulse = (idle_time.as_secs_f32() * PI).cos() * 0.5 + 0.5;

        fill(
            bounds,
            Hsla::from(cx.theme().text_view_cursor).opacity(pulse),
        )
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        window.paint_quad(prepaint.clone());
    }
}

impl Render for Cursor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        *self
    }
}
