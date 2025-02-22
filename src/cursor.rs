use std::f32::consts::PI;
use std::time::Instant;

use gpui::prelude::*;
use gpui::{div, px, Context, Entity, EventEmitter, Hsla, IsZero, Pixels, Point, Window};

use crate::components::continuous_animation::ContinuousAnimationExt;
use crate::text_view::TextView;
use crate::theme::ActiveTheme;

impl EventEmitter<Cursor> for TextView {}

#[derive(Clone, Copy, PartialEq)]
pub struct Cursor {
    pub line_height: Pixels,
    pub target_position: Point<Pixels>,
    pub text_origin: Point<Pixels>,
    pub animate_movement: bool,
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

            Self {
                line_height: px(0.0),
                target_position: Point::default(),
                text_origin: Point::default(),
                animate_movement: true,
            }
        })
    }
}

impl Render for Cursor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let target_position = self.target_position;
        let text_origin = self.text_origin;
        let animate_movement = self.animate_movement;
        let cursor_color = cx.theme().text_view_cursor;

        div()
            .absolute()
            .w(self.line_height / 3.0)
            .h(px(2.0))
            .with_continuous_animation(
                "cursor",
                AnimationState {
                    position: Point::default(),
                    idle_start: Instant::now(),
                },
                move |element, state, delta, _window, _cx| {
                    // Update state
                    let magnitude = (state.position - target_position).magnitude();
                    if animate_movement && magnitude > 0.5 && !state.position.is_zero() {
                        let mix = (delta * 30.0).clamp(0.0, 1.0);
                        state.position = state.position * (1.0 - mix) + target_position * mix;
                        state.idle_start = Instant::now();
                    } else {
                        state.position = target_position;
                    }

                    // Get style
                    let element_position = state.position + text_origin;
                    let cursor_pulse =
                        (state.idle_start.elapsed().as_secs_f32() * PI).cos() * 0.5 + 0.5;

                    (
                        element
                            .left(element_position.x)
                            .top(element_position.y)
                            .bg(Hsla::from(cursor_color).opacity(cursor_pulse)),
                        true,
                    )
                },
            )
    }
}

#[derive(Clone)]
struct AnimationState {
    position: Point<Pixels>,
    idle_start: Instant,
}
