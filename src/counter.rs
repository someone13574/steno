use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{div, App, Entity, EventEmitter, Window};
use smol::stream::StreamExt;
use smol::Timer;

use crate::text_view::TextView;
use crate::theme::ActiveTheme;

const WPM_CHARS_PER_WORD: f32 = 5.0;

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
                duration: 15,
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
        self.start_time = Some(start_time);

        let num_samples = self.duration as usize + 1;
        cx.spawn(async move |counter, mut cx| {
            let mut last_typed_count = 0;
            let mut wpm_measurements = Vec::with_capacity(num_samples);

            let mut timer = Timer::interval_at(start_time, Duration::from_secs(1));
            timer.next().await;
            while timer.next().await.is_some() {
                let active = counter
                    .update(&mut cx, |counter, cx| {
                        let current_typed_count = counter.text_view.read(cx).typed_chars;
                        wpm_measurements.push(
                            (current_typed_count - last_typed_count) as f32 / WPM_CHARS_PER_WORD
                                * 60.0,
                        );
                        last_typed_count = current_typed_count;

                        if counter
                            .start_time
                            .is_some_and(|start| start.elapsed().as_secs() > counter.duration)
                        {
                            cx.emit(CounterFinishedEvent {
                                wpm_measurements: wpm_measurements.clone(),
                            });
                            return false;
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
