use std::time::Duration;

use gpui::prelude::*;
use gpui::{div, point, px, Animation, AnimationExt, App, Entity, FocusHandle, Window};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::components::clamp::clamp;
use crate::components::line_chart::LineChart;
use crate::counter::Counter;
use crate::text_view::TextView;

pub struct ContentView {
    _text_view: Entity<TextView>,
    _counter: Entity<Counter>,
    seed: u64,
}

impl ContentView {
    pub fn new(focus_handle: FocusHandle, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| {
            let text_view = TextView::new(focus_handle.clone(), cx);

            Self {
                _text_view: text_view.clone(),
                _counter: Counter::new(text_view, cx),
                seed: rand::rng().random(),
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
            .child(
                clamp(
                    px(500.0),
                    px(300.0),
                    LineChart {
                        target_grid_lines_spacing: px(32.0),
                        scale_rounding: 2.0,
                        animation_progress: 1.0,
                        points: {
                            let n = 10;

                            let mut rng = StdRng::seed_from_u64(self.seed);
                            let mut points = Vec::with_capacity(n);
                            let mut y = 50.0;
                            for x in 0..n {
                                y += (rng.random::<f32>() - 0.45) * 50.0;
                                y = y.max(0.0);
                                points.push(point(x as f32, y));
                            }

                            points
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
        // .child(div().flex_1())
        // .child(self.text_view.clone())
        // .child(
        //     div().flex_1().flex().flex_col().justify_end().child(
        //         clamp(px(80.0), px(40.0), self.counter.clone())
        //             .vertical()
        //             .position(Percentage(1.0))
        //             .smoothing(10.0),
        //     ),
        // )
    }
}
