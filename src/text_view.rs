use gpui::prelude::*;
use gpui::{
    anchored, div, point, px, AnchoredPositionMode, App, Bounds, ElementId, Entity, FocusHandle,
    GlobalElementId, KeyDownEvent, LayoutId, Pixels, Point, StyledText, TextLayout, TextRun,
    Window,
};

use crate::components::continuous_animation::ContinuousAnimationExt;
use crate::counter::StartCounterEvent;
use crate::cursor::Cursor;
use crate::dictionary::Dictionary;
use crate::theme::ActiveTheme;

pub struct TextView {
    text: String,
    char_head: usize,
    utf8_head: usize,
    over_inserted_stack: Vec<usize>,
    run_lens: Vec<(bool, usize)>,
    focus_handle: FocusHandle,
    cursor: Entity<Cursor>,
    target_scroll: Pixels,
    animate_scroll: bool,
}

impl TextView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                text: Dictionary::random_text(50, cx),
                char_head: 0,
                utf8_head: 0,
                over_inserted_stack: vec![0],
                run_lens: Vec::new(),
                focus_handle,
                cursor: Cursor::new(cx),
                target_scroll: px(0.0),
                animate_scroll: true,
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

    fn fruncate_text(&mut self, utf8_len: usize) {
        let char_len = self.text[..utf8_len].chars().count();

        // Fruncate runs
        let (run_offset, run_len, run_idx) = self
            .run_lens
            .iter()
            .enumerate()
            .scan(0, |offset_acc, (run_idx, (_correct, run_len))| {
                let offset = *offset_acc;
                *offset_acc += run_len;
                Some((offset, *run_len, run_idx))
            })
            .find(|(offset, run_len, _)| (*offset..offset + run_len).contains(&utf8_len))
            .unwrap();
        self.run_lens.drain(0..run_idx);
        self.run_lens[0].1 = run_offset + run_len - utf8_len;

        // Remove text
        self.text.drain(0..utf8_len);

        // Move head
        self.char_head -= char_len;
        self.utf8_head -= utf8_len;
        self.target_scroll = px(0.0);
        self.animate_scroll = false;
    }
}

impl Render for TextView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let target_scroll = self.target_scroll;
        let animate_scroll = self.animate_scroll;
        let window_active = window.is_window_active();
        let entity = cx.entity().downgrade();

        div()
            .track_focus(&self.focus_handle)
            .text_3xl()
            .font_family("Sans")
            .text_color(cx.theme().text_view_placeholder_text)
            .child(div().with_continuous_animation(
                "text-entry-animation",
                px(0.0),
                move |element, current_scroll, delta, window, _cx| {
                    let magnitude = (*current_scroll - target_scroll).abs().0;
                    let animating = if magnitude > 0.5 && window_active && animate_scroll {
                        let mix = (delta * 20.0).clamp(0.0, 1.0);
                        *current_scroll = *current_scroll * (1.0 - mix) + target_scroll * mix;
                        true
                    } else {
                        *current_scroll = target_scroll;
                        false
                    };

                    (
                        element
                            .w_full()
                            .h(window.line_height() * 3)
                            .overflow_hidden()
                            .child(TextViewElement {
                                entity: entity.upgrade().unwrap(),
                                scroll: *current_scroll,
                                scrolling: animating,
                            }),
                        animating,
                    )
                },
            ))
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
                cx.emit(StartCounterEvent);

                match (
                    this.text.chars().nth(this.char_head),
                    event.keystroke.key.as_str(),
                    event.keystroke.key_char.as_deref(),
                ) {
                    (_, "backspace", _) => {
                        let is_whitespace = this
                            .text
                            .chars()
                            .nth(this.char_head.saturating_sub(1))
                            .is_some_and(|char| char.is_whitespace());

                        this.backspace();

                        if is_whitespace {
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
    scroll: Pixels,
    scrolling: bool,
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

        // Create styled text
        let runs = text_view
            .run_lens
            .iter()
            .map(|(correct, run_len)| {
                TextRun {
                    len: *run_len,
                    font: text_style.font(),
                    color: if *correct {
                        cx.theme().text_view_correct_text.into()
                    } else {
                        cx.theme().text_view_incorrect_text.into()
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
        let scrolled_bounds = bounds - point(px(0.0), self.scroll);
        styled_text.prepaint(None, scrolled_bounds, &mut (), window, cx);

        self.entity.update(cx, |text_view, cx| {
            let utf8_len = text_view
                .text
                .chars()
                .nth(text_view.char_head)
                .map_or(1, char::len_utf8);
            let (glyph_position, cursor_position) = cursor_pos(
                text_view.utf8_head,
                utf8_len,
                styled_text.layout(),
                window.line_height() / 3.0,
            );

            // Set scroll target
            let scrolled_lines = scrolled_lines(text_view.target_scroll, window.line_height());
            text_view.target_scroll = (glyph_position.y - window.line_height()).max(px(0.0));

            // Update cursor
            let current_cursor = *text_view.cursor.read(cx);
            let new_cursor = Cursor {
                line_height: window.line_height(),
                target_position: cursor_position,
                text_origin: scrolled_bounds.origin,
                animate_movement: text_view.animate_scroll,
            };

            if current_cursor != new_cursor {
                cx.emit(new_cursor);
            }

            // Remove old text
            if scrolled_lines != 0 && !self.scrolling {
                let layout = styled_text.layout().line_layout_for_index(0).unwrap();
                let wrap_boundary = layout.wrap_boundaries[scrolled_lines - 1];
                text_view.fruncate_text(
                    layout.runs()[wrap_boundary.run_ix].glyphs[wrap_boundary.glyph_ix].index,
                );
            } else {
                text_view.animate_scroll = true;
            }

            // Add new text
            let num_full_lines = styled_text
                .layout()
                .line_layout_for_index(0)
                .unwrap()
                .wrap_boundaries
                .len();
            if num_full_lines - scrolled_lines < 5 {
                text_view
                    .text
                    .push_str(format!(" {}", Dictionary::random_text(16, cx)).as_str());
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
        styled_text.paint(
            None,
            bounds - point(px(0.0), self.scroll),
            &mut (),
            &mut (),
            window,
            cx,
        );
    }
}

fn cursor_pos(
    utf8_idx: usize,
    utf8_len: usize,
    layout: &TextLayout,
    cursor_width: Pixels,
) -> (Point<Pixels>, Point<Pixels>) {
    let line_height = layout.line_height();
    let layout = layout.line_layout_for_index(utf8_idx).unwrap();

    // Get glyph position and width
    let glyph_width = layout.unwrapped_layout.x_for_index(utf8_idx + utf8_len)
        - layout.unwrapped_layout.x_for_index(utf8_idx);

    let glyph_position = layout.position_for_index(utf8_idx, line_height).unwrap();
    let glyph_position = if layout
        .wrap_boundaries()
        .iter()
        .map(|wrap_boundary| {
            layout.runs()[wrap_boundary.run_ix].glyphs[wrap_boundary.glyph_ix].index
        })
        .any(|wrap_idx| wrap_idx == utf8_idx)
    {
        // Go to next line
        point(px(0.0), glyph_position.y + line_height)
    } else {
        glyph_position
    };

    // Calculate cursor x position
    let cursor_center_x = glyph_position.x + glyph_width / 2.0;
    let cursor_x = cursor_center_x - cursor_width / 2.0;

    // Calculate cursor y position
    let cursor_y = glyph_position.y + line_height - layout.descent();

    (glyph_position, point(cursor_x, cursor_y))
}

fn scrolled_lines(y_pos: Pixels, line_height: Pixels) -> usize {
    (y_pos / line_height + 0.5) as usize
}
