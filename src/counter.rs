use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{div, App, Entity, EventEmitter, Window};
use smol::stream::StreamExt;
use smol::Timer;

use crate::text_view::TextView;

pub struct Counter {
    start_time: Option<Instant>,
    duration: u64,
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

        cx.spawn(async move |counter, mut cx| {
            let mut timer = Timer::interval_at(start_time, Duration::from_secs(1));
            while timer.next().await.is_some() {
                let active = counter
                    .update(&mut cx, |counter, cx| {
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let elapsed = self
            .start_time
            .map_or(0, |start_time| start_time.elapsed().as_secs());

        div()
            .flex()
            .size_full()
            .justify_center()
            .text_color(gpui::white())
            .text_lg()
            .font_family("Sans")
            .child(format!("{}", self.duration.saturating_sub(elapsed)))
    }
}

pub struct StartCounterEvent;

impl EventEmitter<StartCounterEvent> for TextView {}
