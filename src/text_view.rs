use gpui::prelude::*;
use gpui::{
    anchored, div, point, px, AnchoredPositionMode, App, Bounds, ElementId, Entity, FocusHandle,
    GlobalElementId, KeyDownEvent, LayoutId, Pixels, Point, StyledText, TextRun, Window,
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
}

impl TextView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            Self {
                text: Dictionary::random_text(20, cx),
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
            .text_color(cx.theme().text_view_placeholder_text)
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
        styled_text.prepaint(None, bounds, &mut (), window, cx);

        self.entity.update(cx, |text_view, cx| {
            let line_height = styled_text.layout().line_height();
            let cursor_position = styled_text
                .layout()
                .position_for_index(text_view.utf8_head)
                .unwrap();

            let width = styled_text
                .layout()
                .position_for_index(text_view.utf8_head + 1)
                .map_or(px(0.0), |pos| pos.x)
                - cursor_position.x;
            let width = if width < px(1.0) {
                line_height / 3.0
            } else {
                width
            };

            let current_cursor = text_view.cursor.read(cx);
            let new_cursor = Cursor {
                line_height,
                target_position: cursor_position - bounds.origin
                    + point(
                        (width - line_height / 3.0) / 2.0,
                        line_height
                            - styled_text
                                .layout()
                                .line_layout_for_index(text_view.utf8_head)
                                .unwrap()
                                .descent(),
                    ),
                text_origin: bounds.origin,
            };

            if *current_cursor != new_cursor {
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
