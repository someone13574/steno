use std::time::Duration;

use gpui::prelude::*;
use gpui::{div, point, px, Animation, AnimationExt, App, Entity, FocusHandle, Percentage, Window};

use crate::components::clamp::clamp;
use crate::components::line_chart::LineChart;
use crate::counter::Counter;
use crate::text_view::TextView;

pub struct ContentView {
    text_view: Entity<TextView>,
    counter: Entity<Counter>,
    wpm_measurements: Option<Vec<f32>>,
}

impl ContentView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let text_view = TextView::new(focus_handle.clone(), cx);
            let counter = Counter::new(text_view.clone(), cx);

            cx.subscribe(&counter, |this: &mut Self, _counter, event, cx| {
                this.wpm_measurements = Some(event.wpm_measurements.clone());
                cx.notify();
            })
            .detach();

            Self {
                text_view,
                counter,
                wpm_measurements: None,
            }
        })
    }
}

impl Render for ContentView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .text_color(gpui::white())
            .font_family("Sans")
            .when(self.wpm_measurements.is_none(), |element| {
                element
                    .child(div().flex_1())
                    .child(self.text_view.clone())
                    .child(
                        div().flex_1().flex().flex_col().justify_end().child(
                            clamp(px(80.0), px(40.0), self.counter.clone())
                                .vertical()
                                .position(Percentage(1.0))
                                .smoothing(10.0),
                        ),
                    )
            })
            .when_some(
                self.wpm_measurements.as_ref(),
                |element, wpm_measurements| {
                    element.child(
                        clamp(
                            px(500.0),
                            px(300.0),
                            LineChart {
                                target_grid_lines_spacing: px(64.0),
                                scale_rounding: 5.0,
                                animation_progress: 1.0,
                                points: {
                                    wpm_measurements
                                        .iter()
                                        .enumerate()
                                        .map(|(idx, &wpm)| point(idx as f32, wpm))
                                        .collect::<Vec<_>>()
                                },
                            }
                            .with_animation(
                                "chart",
                                Animation::new(Duration::from_millis(1500)),
                                |mut element, progress| {
                                    element.animation_progress = progress;
                                    element
                                },
                            ),
                        )
                        .vertical(),
                    )
                },
            )
    }
}
