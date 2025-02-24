use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    ease_in_out, fill, point, px, relative, size, AnyElement, App, AvailableSpace, Bounds,
    ContentMask, Element, ElementId, GlobalElementId, Hsla, LayoutId, Path, PathBuilder, Pixels,
    Point, SharedString, Size, Style, Window,
};

use crate::theme::ActiveTheme;

const LABEL_AVAILABLE_SPACE: Size<AvailableSpace> = Size {
    width: AvailableSpace::MinContent,
    height: AvailableSpace::MinContent,
};

pub struct LineChart {
    pub x_axis_label: Option<SharedString>,
    pub grid_lines_spacing: Pixels,
    pub animation_progress: f32,
    pub points: Vec<Point<f32>>,
}

impl IntoElement for LineChart {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct RequestLayout {
    x_axis_label: Option<AnyElement>,
}

pub struct Prepaint {
    content_bounds: Bounds<Pixels>,
    path: Option<Path<Pixels>>,
}

impl Element for LineChart {
    type PrepaintState = Prepaint;
    type RequestLayoutState = RequestLayout;

    fn id(&self) -> Option<ElementId> {
        Some("chart-element".into())
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            size: size(relative(1.0).into(), relative(1.0).into()),
            ..Default::default()
        };

        (
            window.request_layout(style, [], cx),
            RequestLayout {
                x_axis_label: self
                    .x_axis_label
                    .as_ref()
                    .map(|label| label.clone().into_any_element()),
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Axes
        let vertical_offset = if let Some(x_axis_label) = request_layout.x_axis_label.as_mut() {
            let size = x_axis_label.layout_as_root(LABEL_AVAILABLE_SPACE, window, cx);
            let label_origin = point(
                bounds.center().x - size.width / 2.0,
                bounds.bottom() - size.height,
            );
            x_axis_label.prepaint_at(label_origin, window, cx);

            size.height
        } else {
            px(0.0)
        };

        let content_bounds = Bounds {
            origin: bounds.origin + point(px(0.0), vertical_offset),
            size: size(bounds.size.width, bounds.size.height - vertical_offset * 2),
        };

        // Create path
        let (mut data_max, mut data_min) = self.points.iter().fold(
            (point(-10000.0, -10000.0), point(10000.0, 10000.0)),
            |(max, min), point| (point.max(&max), point.min(&min)),
        );
        let data_range = data_max - data_min;
        data_max.y += data_range.y * 0.1;
        data_min.y -= data_range.y * 0.1;
        let data_range = data_max - data_min;

        let scaled_points: Vec<Point<Pixels>> = self
            .points
            .iter()
            .map(|point| {
                Point {
                    x: px(point.x - data_min.x) / data_range.x * content_bounds.size.width,
                    y: px(point.y - data_min.y) / data_range.y * -content_bounds.size.height,
                } + content_bounds.bottom_left()
            })
            .collect();
        let tangents = tangents(&scaled_points);

        let mut path = PathBuilder::stroke(px(2.0));
        path.move_to(scaled_points[0]);
        for idx in 0..(scaled_points.len() - 1) {
            let point_a = scaled_points[idx];
            let point_b = scaled_points[idx + 1];
            let tangent_a = tangents[idx];
            let tangent_b = tangents[idx + 1];

            // Get smoothing factor
            let segment = point_b - point_a;
            let length = segment.x.abs().min(px(segment.magnitude() as f32)); // Use min of x difference to prevent going backwards
            let smoothing = length / 2.0;

            // Control points (clamp to prevent line from leaving content bounds)
            let control_a = (point_a + tangent_a * smoothing)
                .clamp(&content_bounds.origin, &content_bounds.bottom_right());
            let control_b = (point_b - tangent_b * smoothing)
                .clamp(&content_bounds.origin, &content_bounds.bottom_right());

            path.cubic_bezier_to(point_b, control_a, control_b);
        }

        Prepaint {
            content_bounds,
            path: Some(path.build().unwrap()),
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let grid_lines_finish = 0.75;
        let main_line_delay = 0.25;

        // Horizontal grid lines
        let num_grid_lines =
            (prepaint.content_bounds.size.height / self.grid_lines_spacing + 0.5) as u32;
        let num_grid_lines_float = window.with_element_state(id.unwrap(), |state, window| {
            let (mut state, last_frame) = state.unwrap_or((num_grid_lines as f32, None));
            let last_frame = last_frame.unwrap_or(Instant::now());
            let elapsed = last_frame.elapsed().as_secs_f32();

            let last_frame = if (num_grid_lines as f32 - state).abs() > 0.01 {
                let mix = (elapsed * 5.0).clamp(0.0, 1.0);
                state = state * (1.0 - mix) + num_grid_lines as f32 * mix;
                window.request_animation_frame();
                Some(Instant::now())
            } else {
                state = num_grid_lines as f32;
                None
            };

            (state, (state, last_frame))
        });
        let grid_line_spacing = prepaint.content_bounds.size.height / num_grid_lines_float;
        let num_grid_lines = (num_grid_lines_float.ceil() + 0.5) as u32;
        let num_grid_lines_diff = num_grid_lines_float - num_grid_lines as f32;

        for y in
            (0..num_grid_lines).map(|idx| px(idx as f32 + num_grid_lines_diff) * grid_line_spacing)
        {
            // Handle out of bounds from resize animation
            let opacity = ease_in_out((y / grid_line_spacing + 1.0).clamp(0.0, 1.0));
            let y = y.max(px(0.0));

            // Determine progress
            let start_time =
                (1.0 - y / prepaint.content_bounds.size.height) * 0.5 * grid_lines_finish;
            let progress = (self.animation_progress - start_time) / (0.5 * grid_lines_finish);
            if progress < 0.0 {
                continue;
            }

            // Paint
            let width = prepaint.content_bounds.size.width * ease_in_out(progress.clamp(0.0, 1.0));
            window.paint_quad(fill(
                gpui::bounds(
                    prepaint.content_bounds.origin + point(px(0.0), y),
                    size(width, px(1.0)),
                ),
                Hsla::from(cx.theme().base.dim_foreground).opacity(opacity),
            ));
        }

        // Path
        window.with_content_mask(
            Some(ContentMask {
                bounds: Bounds {
                    origin: prepaint.content_bounds.origin,
                    size: size(
                        prepaint.content_bounds.size.width
                            * (((self.animation_progress - main_line_delay)
                                / (1.0 - main_line_delay))
                                .clamp(0.0, 1.0)),
                        prepaint.content_bounds.size.height,
                    ),
                },
            }),
            |window| {
                window.paint_path(prepaint.path.take().unwrap(), cx.theme().base.foreground);
            },
        );

        // Axes
        window.paint_quad(fill(
            gpui::bounds(
                prepaint.content_bounds.bottom_left(),
                size(prepaint.content_bounds.size.width, px(2.0)),
            ),
            cx.theme().base.foreground,
        ));
        window.paint_quad(fill(
            gpui::bounds(
                prepaint.content_bounds.origin,
                size(px(2.0), prepaint.content_bounds.size.height),
            ),
            cx.theme().base.foreground,
        ));

        if let Some(x_axis_label) = request_layout.x_axis_label.as_mut() {
            x_axis_label.paint(window, cx);
        }
    }
}

fn tangents(points: &[Point<Pixels>]) -> Vec<Point<Pixels>> {
    if points.len() <= 2 {
        return Vec::new();
    }

    let mut tangents = Vec::with_capacity(points.len());
    for (idx, &point) in points.iter().enumerate() {
        let unnormalized = match idx {
            0 => points[1] - point,
            idx if idx == points.len() - 1 => point - points[idx - 1],
            _ => (points[idx + 1] - points[idx - 1]) * 0.5,
        };

        let factor = px(1.0 / unnormalized.magnitude() as f32);
        tangents.push(unnormalized * factor);
    }

    tangents
}
