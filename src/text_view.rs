use std::ops::Range;

use gpui::prelude::*;
use gpui::{
    div, rgba, App, Entity, FocusHandle, HighlightStyle, KeyDownEvent, StyledText, TextStyle,
    Window,
};

pub struct TextView {
    text: String,
    char_head: usize,
    utf8_head: usize,
    over_inserted_stack: Vec<usize>,
    runs: Vec<(bool, Range<usize>)>,
    focus_handle: FocusHandle,
}

impl TextView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|_cx| {
            Self {
                text: "The quick brown fox jumped over the lazy dog".into(),
                char_head: 0,
                utf8_head: 0,
                over_inserted_stack: vec![0],
                runs: Vec::new(),
                focus_handle,
            }
        })
    }

    fn add_run(&mut self, correct: bool, utf8_len: usize, char_len: usize) {
        if let Some((last_run_correct, last_run)) = self.runs.last_mut() {
            if *last_run_correct == correct {
                last_run.end += utf8_len;
            } else {
                self.runs
                    .push((correct, self.utf8_head..self.utf8_head + utf8_len));
            }
        } else {
            self.runs
                .push((correct, self.utf8_head..self.utf8_head + utf8_len));
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
        let delete = if let Some((_, last_run)) = self.runs.last_mut() {
            last_run.end -= unwind_len;
            Range::is_empty(last_run)
        } else {
            false
        };
        if delete {
            self.runs.pop();
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
            .child(StyledText::new(self.text.clone()).with_highlights(
                &TextStyle {
                    font_family: "Sans".into(),
                    color: rgba(0xffffff20).into(),
                    ..window.text_style()
                },
                self.runs.iter().map(|(correct, run)| {
                    (
                        run.clone(),
                        HighlightStyle {
                            color: Some(if *correct {
                                rgba(0xffffffff).into()
                            } else {
                                rgba(0xff0000ff).into()
                            }),
                            background_color: None,
                            ..Default::default()
                        },
                    )
                }),
            ))
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
