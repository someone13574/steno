use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    anchored, div, point, px, relative, size, AnchoredPositionMode, App, Bounds, ContentMask,
    ElementId, Entity, FocusHandle, GlobalElementId, KeyDownEvent, LayoutId, Pixels, Point, Style,
    StyledText, TextLayout, TextRun, Window,
};

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
    target_scroll: Point<Pixels>,
}

impl TextView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                text: Dictionary::random_text(100, cx),
                char_head: 0,
                utf8_head: 0,
                over_inserted_stack: vec![0],
                run_lens: Vec::new(),
                focus_handle,
                cursor: Cursor::new(cx),
                target_scroll: Point::default(),
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
            .text_color(cx.theme().text_view_placeholder_text)
            .child(TextViewElement {
                entity: cx.entity(),
                line_clamp: 3,
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
    line_clamp: usize,
}

impl IntoElement for TextViewElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextViewElement {
    type PrepaintState = Point<Pixels>;
    type RequestLayoutState = StyledText;

    fn id(&self) -> Option<ElementId> {
        Some("text-view".into())
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let text_style = window.text_style();
        let text_view = self.entity.read(cx);

        // Create style
        let style = Style {
            max_size: size(
                relative(1.0).into(),
                (window.line_height() * self.line_clamp).into(),
            ),
            ..Default::default()
        };

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
        let styled_text_layout = styled_text.request_layout(None, window, cx).0;

        (
            window.request_layout(style, [styled_text_layout], cx),
            styled_text,
        )
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        styled_text: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let target_scroll = self.entity.read(cx).target_scroll;
        let scroll_offset = window.with_element_state(id.unwrap(), |state, window| {
            let (scroll_offset, last_frame) = state.unwrap_or((Point::default(), Instant::now()));

            let magnitude = (scroll_offset - target_scroll).magnitude();
            let scroll_offset = if magnitude > 0.1 {
                window.request_animation_frame();

                let delta = (last_frame.elapsed().as_secs_f64() * 20.0).clamp(0.0, 1.0) as f32;
                scroll_offset * (1.0 - delta) + target_scroll * delta
            } else {
                target_scroll
            };

            (scroll_offset, (scroll_offset, Instant::now()))
        });

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            styled_text.prepaint(None, bounds + scroll_offset, &mut (), window, cx);
        });

        self.entity.update(cx, |text_view, cx| {
            let line_height = styled_text.layout().line_height();
            let current_cursor = text_view.cursor.read(cx);
            let (glyph_position, cursor_position) =
                cursor_pos(text_view.utf8_head, styled_text.layout(), line_height / 3.0);

            text_view.target_scroll =
                point(px(0.0), -(glyph_position.y - line_height).max(px(0.0)));

            let new_cursor = Cursor {
                line_height,
                target_position: cursor_position,
                text_origin: bounds.origin + scroll_offset,
            };

            if *current_cursor != new_cursor {
                cx.emit(new_cursor);
            }
        });

        scroll_offset
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        styled_text: &mut Self::RequestLayoutState,
        scroll_offset: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            styled_text.paint(None, bounds + *scroll_offset, &mut (), &mut (), window, cx);
        });
    }
}

fn cursor_pos(
    glyph_idx: usize,
    layout: &TextLayout,
    cursor_width: Pixels,
) -> (Point<Pixels>, Point<Pixels>) {
    let line_height = layout.line_height();
    let layout = layout.line_layout_for_index(glyph_idx).unwrap();

    // Get glyph position and width
    let glyph_width = layout.unwrapped_layout.x_for_index(glyph_idx + 1)
        - layout.unwrapped_layout.x_for_index(glyph_idx);
    let run_offsets = layout
        .runs()
        .iter()
        .scan(0, |acc, run| {
            let offset = *acc;
            *acc += run.glyphs.len();
            Some(offset)
        })
        .collect::<Vec<_>>();

    let glyph_position = layout.position_for_index(glyph_idx, line_height).unwrap();
    let glyph_position = if layout
        .wrap_boundaries()
        .iter()
        .map(|wrap_boundary| run_offsets[wrap_boundary.run_ix] + wrap_boundary.glyph_ix)
        .any(|wrap_idx| wrap_idx == glyph_idx)
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
