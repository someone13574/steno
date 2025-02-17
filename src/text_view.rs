use std::f32::consts::PI;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    anchored, div, fill, px, rgba, size, AnchoredPositionMode, App, Bounds, Edges, ElementId,
    Entity, EventEmitter, FocusHandle, GlobalElementId, IsZero, KeyDownEvent, LayoutId, PaintQuad,
    Pixels, Point, Position, Rgba, Style, StyledText, TextRun, Window,
};

pub struct TextView {
    text: String,
    char_head: usize,
    utf8_head: usize,
    over_inserted_stack: Vec<usize>,
    run_lens: Vec<(bool, usize)>,
    focus_handle: FocusHandle,
    cursor: Entity<Cursor>,
}

impl TextView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                text: "The quick brown fox jumped over the lazy dog".into(),
                char_head: 0,
                utf8_head: 0,
                over_inserted_stack: vec![0],
                run_lens: Vec::new(),
                focus_handle,
                cursor: Cursor::new(cx),
            }
        })
    }

    fn add_run(&mut self, correct: bool, utf8_len: usize, char_len: usize) {
        if let Some((last_run_correct, last_run)) = self.run_lens.last_mut() {
            if *last_run_correct == correct {
                *last_run += utf8_len;
            } else {
                self.run_lens.push((correct, utf8_len));
            }
        } else {
            self.run_lens.push((correct, utf8_len));
        }

        self.utf8_head += utf8_len;
        self.char_head += char_len;
    }

    fn add_whitespace(&mut self, whitespace: &str) {
        let end_of_word: usize = self
            .text
            .char_indices()
            .take_while(|(idx, char)| *idx < self.char_head || !char.is_whitespace())
            .map(|(_, char)| char.len_utf8())
            .sum();

        // Add run for invalid chars
        if end_of_word > self.utf8_head {
            self.add_run(
                false,
                end_of_word - self.utf8_head,
                self.text[self.utf8_head..end_of_word].chars().count(),
            );
        }

        // Replace whitespace with written whitespace
        let replace_len: usize = self
            .text
            .chars()
            .skip(self.char_head)
            .take_while(|char| char.is_whitespace())
            .map(char::len_utf8)
            .sum();
        self.text
            .replace_range(end_of_word..end_of_word + replace_len, whitespace);

        // Add run for whitespace
        self.add_run(
            replace_len != 0,
            whitespace.len(),
            whitespace.chars().count(),
        );

        // Advance over inserted stack
        self.over_inserted_stack.push(0);
    }

    fn backspace(&mut self) {
        if self.char_head == 0 {
            return;
        }

        let unwind_len = self
            .text
            .chars()
            .nth(self.char_head - 1)
            .unwrap()
            .len_utf8();

        // Remove text
        let over_inserted = self.over_inserted_stack.last_mut().unwrap();
        if *over_inserted != 0 {
            self.text
                .replace_range(self.utf8_head - unwind_len..self.utf8_head, "");
            *over_inserted -= unwind_len;
        }

        // Remove runs
        let delete = if let Some((_, last_run_len)) = self.run_lens.last_mut() {
            *last_run_len -= unwind_len;
            *last_run_len == 0
        } else {
            false
        };
        if delete {
            self.run_lens.pop();
        }

        self.utf8_head -= unwind_len;
        self.char_head -= 1;
    }
}

impl Render for TextView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .track_focus(&self.focus_handle)
            .text_2xl()
            .font_family("Sans")
            .text_color(rgba(0xffffff20))
            .child(TextViewElement {
                entity: cx.entity(),
            })
            .when(
                self.focus_handle.is_focused(window) && window.is_window_active(),
                |element| {
                    element.child(
                        anchored()
                            .position_mode(AnchoredPositionMode::Window)
                            .position(Point::default())
                            .child(self.cursor.clone()),
                    )
                },
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                match (
                    this.text.chars().nth(this.char_head),
                    event.keystroke.key.as_str(),
                    event.keystroke.key_char.as_deref(),
                ) {
                    (_, "backspace", _) => {
                        if !this
                            .text
                            .chars()
                            .nth(this.char_head.saturating_sub(1))
                            .is_some_and(|char| char.is_whitespace())
                        {
                            this.backspace();
                        }

                        let mut removed_whitespace = false;
                        while this
                            .text
                            .chars()
                            .nth(this.char_head.saturating_sub(1))
                            .is_some_and(|char| char.is_whitespace())
                        {
                            this.backspace();
                            removed_whitespace = true;
                        }
                        if removed_whitespace {
                            this.over_inserted_stack.pop();
                        }
                    }
                    (_, _, Some(whitespace))
                        if whitespace.chars().all(|char| char.is_whitespace()) =>
                    {
                        if this.char_head != 0
                            && this
                                .text
                                .chars()
                                .nth(this.char_head - 1)
                                .is_some_and(|char| !char.is_whitespace())
                        {
                            this.add_whitespace(whitespace);
                        }
                    }
                    (Some(replaced), _, Some(replace_with)) if !replaced.is_whitespace() => {
                        this.add_run(
                            replaced.to_string() == *replace_with,
                            replaced.len_utf8(),
                            replace_with.chars().count(),
                        );
                    }
                    (_, _, Some(to_insert)) => {
                        this.text.insert_str(this.utf8_head, to_insert);
                        this.add_run(false, to_insert.len(), to_insert.chars().count());
                        *this.over_inserted_stack.last_mut().unwrap() += to_insert.len();
                    }
                    _ => {}
                }

                cx.notify();
            }))
    }
}

struct TextViewElement {
    entity: Entity<TextView>,
}

impl IntoElement for TextViewElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextViewElement {
    type PrepaintState = ();
    type RequestLayoutState = StyledText;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let text_style = window.text_style();
        let text_view = self.entity.read(cx);

        let runs = text_view
            .run_lens
            .iter()
            .map(|(correct, run_len)| {
                TextRun {
                    len: *run_len,
                    font: text_style.font(),
                    color: if *correct {
                        rgba(0xffffffff).into()
                    } else {
                        rgba(0xff0000ff).into()
                    },
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }
            })
            .chain([TextRun {
                len: text_view.text.len() - text_view.utf8_head,
                font: text_style.font(),
                color: text_style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }])
            .collect();

        let mut styled_text = StyledText::new(&text_view.text).with_runs(runs);
        (styled_text.request_layout(None, window, cx).0, styled_text)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        styled_text: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        styled_text.prepaint(None, bounds, &mut (), window, cx);

        let text_style = window.text_style();
        let line_height = text_style.line_height_in_pixels(window.rem_size());
        self.entity.update(cx, |text_view, cx| {
            let cursor_position = styled_text
                .layout()
                .position_for_index(text_view.utf8_head)
                .unwrap()
                - bounds.origin;
            let current_cursor = text_view.cursor.read(cx).element;
            let new_cursor = CursorElement {
                line_height,
                target_position: cursor_position,
                text_origin: bounds.origin,
            };

            if current_cursor != new_cursor {
                cx.emit(new_cursor);
            }
        });
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        styled_text: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        styled_text.paint(None, bounds, &mut (), &mut (), window, cx);
    }
}

#[derive(Clone, Copy, Default)]
struct Cursor {
    element: CursorElement,
}

impl Cursor {
    pub fn new(cx: &mut Context<TextView>) -> Entity<Self> {
        let text_view = cx.entity();

        cx.new(|cx| {
            cx.subscribe(&text_view, |cursor: &mut Self, _, new_cursor, cx| {
                cursor.element = *new_cursor;

                cx.notify();
            })
            .detach();

            Self::default()
        })
    }
}

impl Render for Cursor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.element
    }
}

impl EventEmitter<CursorElement> for TextView {}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct CursorElement {
    line_height: Pixels,
    target_position: Point<Pixels>,
    text_origin: Point<Pixels>,
}

struct CursorState {
    position: Point<Pixels>,
    last_frame: Instant,
    idle_time: Instant,
}

impl IntoElement for CursorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for CursorElement {
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
                size: size(px(2.0).into(), self.line_height.into()),
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
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let pulse = (idle_time.as_secs_f32() * PI).cos() * 0.5 + 0.5;

        fill(
            bounds,
            Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: pulse,
            },
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
