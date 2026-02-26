use std::time::Duration;

use gpui::prelude::*;
use gpui::{div, App, Entity, EventEmitter, Window};
use web_time::Instant;

use crate::text_view::TextView;
use crate::theme::ActiveTheme;

const WPM_CHARS_PER_WORD: f32 = 5.0;
const NUM_SAMPLES: u32 = 10;

pub struct Counter {
    start_time: Option<Instant>,
    duration: u64,

    text_view: Entity<TextView>,
}

impl Counter {
    pub fn new(text_view: Entity<TextView>, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            cx.subscribe(
                &text_view,
                |counter: &mut Self, _text_view, _event: &StartCounterEvent, cx| {
                    counter.start_timer(cx);
                },
            )
            .detach();

            Self {
                start_time: None,
                duration: 30,
                text_view,
            }
        })
    }

    /// Starts the counter if not already started
    pub fn start_timer(&mut self, cx: &mut Context<Self>) {
        if self.start_time.is_some() {
            return;
        }

        let start_time = Instant::now();
        let sample_interval = Duration::from_secs(self.duration) / NUM_SAMPLES;
        self.start_time = Some(start_time);

        cx.spawn(async move |counter, cx| {
            let mut last_typed_count = 0;
            let mut wpm_measurements = Vec::with_capacity(NUM_SAMPLES as usize + 1);

            let tick_interval = sample_interval.min(Duration::from_millis(100));
            let mut last_sample = Instant::now();
            cx.background_executor().timer(tick_interval).await;

            loop {
                cx.background_executor().timer(tick_interval).await;
                let active = counter
                    .update(cx, |counter, cx| {
                        if last_sample.elapsed() >= sample_interval {
                            let current_typed_count = counter.text_view.read(cx).typed_chars;
                            wpm_measurements.push(
                                (current_typed_count - last_typed_count) as f32
                                    / WPM_CHARS_PER_WORD
                                    * (60.0 / sample_interval.as_secs_f32()),
                            );
                            last_typed_count = current_typed_count;
                            last_sample = last_sample + sample_interval;

                            if wpm_measurements.len() == NUM_SAMPLES as usize {
                                cx.emit(CounterFinishedEvent {
                                    wpm_measurements: wpm_measurements.clone(),
                                });
                                return false;
                            }
                        }

                        cx.notify();
                        counter.start_time.is_some()
                    })
                    .unwrap();
                if !active {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let remaining = self
            .start_time
            .map(|start_time| self.duration.saturating_sub(start_time.elapsed().as_secs()));

        div()
            .flex()
            .size_full()
            .justify_center()
            .text_color(cx.theme().counter_text)
            .when(remaining.is_none(), |element| {
                element.text_color(cx.theme().counter_idle_text)
            })
            .text_lg()
            .font_family(cx.theme().counter_font_family)
            .child(if let Some(remaining) = remaining {
                format!("{remaining}")
            } else {
                cx.theme().counter_idle_message.to_string()
            })
    }
}

pub struct StartCounterEvent;

impl EventEmitter<StartCounterEvent> for TextView {}

pub struct CounterFinishedEvent {
    pub wpm_measurements: Vec<f32>,
}

impl EventEmitter<CounterFinishedEvent> for Counter {}
