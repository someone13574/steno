use gpui::prelude::*;
use gpui::{
    div, fill, px, rgba, size, App, Bounds, ElementId, Entity, FocusHandle, GlobalElementId,
    KeyDownEvent, LayoutId, PaintQuad, Pixels, StyledText, TextRun, Window,
};

pub struct TextView {
    text: String,
    char_head: usize,
    utf8_head: usize,
    over_inserted_stack: Vec<usize>,
    run_lens: Vec<(bool, usize)>,
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
                run_lens: Vec::new(),
                focus_handle,
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
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
                text: self.text.clone(),
                runs: self.run_lens.clone(),
                head: self.utf8_head,
            })
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
    text: String,
    runs: Vec<(bool, usize)>,
    head: usize,
}

impl IntoElement for TextViewElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextViewElement {
    type PrepaintState = Option<PaintQuad>;
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

        let runs = self
            .runs
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
                len: self.text.len() - self.head,
                font: text_style.font(),
                color: text_style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }])
            .collect();

        let mut styled_text = StyledText::new(&self.text).with_runs(runs);
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

        let cursor_position = styled_text.layout().position_for_index(self.head).unwrap();
        let cursor_height = window.text_style().line_height_in_pixels(window.rem_size());
        let cursor_bounds = Bounds::new(cursor_position, size(px(2.0), cursor_height));

        Some(fill(cursor_bounds, rgba(0xffffffff)))
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        styled_text: &mut Self::RequestLayoutState,
        cursor: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        styled_text.paint(None, bounds, &mut (), &mut (), window, cx);
        window.paint_quad(cursor.take().unwrap());
    }
}
