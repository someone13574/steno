use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    ease_in_out, fill, point, px, relative, size, AnyElement, App, AvailableSpace, Bounds,
    ContentMask, Corners, Edges, Element, ElementId, GlobalElementId, Hsla, LayoutId, PaintQuad,
    Path, PathBuilder, Pixels, Point, Size, Style, TextStyleRefinement, Window,
};

use crate::theme::ActiveTheme;

const LABEL_AVAILABLE_SPACE: Size<AvailableSpace> = Size {
    width: AvailableSpace::MinContent,
    height: AvailableSpace::MinContent,
};

pub struct LineChart {
    pub target_grid_lines_spacing: Pixels,
    pub scale_rounding: f32,
    pub animation_progress: f32,
    pub points: Vec<Point<f32>>,
}

impl IntoElement for LineChart {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct Prepaint {
    content_bounds: Bounds<Pixels>,

    num_animation_grid_lines: u32,
    grid_line_spacing: Pixels,
    fractional_grid_line: f32,

    y_axis_labels: Vec<AnyElement>,
    points: Vec<Point<Pixels>>,
    path: Option<Path<Pixels>>,
}

impl Element for LineChart {
    type PrepaintState = Prepaint;
    type RequestLayoutState = ();

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

        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let mut content_bounds = bounds;

        // Determine scale
        let data_range = self
            .points
            .iter()
            .fold(Point::default(), |max, point| point.max(&max));

        let target_num_grid_lines =
            (content_bounds.size.height / self.target_grid_lines_spacing + 0.5) as u32;
        let scale_rounding = self.scale_rounding;
        let (num_grid_lines_float, scale, units_per_grid_line) =
            window.with_element_state(id.unwrap(), |state, window| {
                // Get state
                let now = Instant::now();
                let (mut num_grid_lines_float, scale, last_frame) =
                    state.unwrap_or((target_num_grid_lines as f32, None, None));
                let elapsed = last_frame.unwrap_or(now).elapsed().as_secs_f32();

                // Calculate grid lines
                let current_frame =
                    if (num_grid_lines_float - target_num_grid_lines as f32).abs() > 0.01 {
                        window.request_animation_frame();

                        let mix = (elapsed * 5.0).clamp(0.0, 1.0);
                        num_grid_lines_float =
                            num_grid_lines_float * (1.0 - mix) + target_num_grid_lines as f32 * mix;
                        Some(now)
                    } else {
                        num_grid_lines_float = target_num_grid_lines as f32;
                        None
                    };

                // Calculate scale
                let units_per_grid_line =
                    (data_range.y / scale_rounding / target_num_grid_lines as f32).ceil()
                        * scale_rounding;

                let target_scale = units_per_grid_line * target_num_grid_lines as f32;
                let mut scale = scale.unwrap_or(target_scale);

                let current_frame = if (target_scale - scale).abs() > 0.01 {
                    window.request_animation_frame();

                    let mix = (elapsed * 5.0).clamp(0.0, 1.0);
                    scale = scale * (1.0 - mix) + target_scale * mix;
                    Some(now)
                } else {
                    scale = target_scale;
                    current_frame
                };

                (
                    (num_grid_lines_float, scale, units_per_grid_line),
                    (num_grid_lines_float, Some(scale), current_frame),
                )
            });

        let grid_line_spacing = content_bounds.size.height / num_grid_lines_float;
        let num_animation_grid_lines = (num_grid_lines_float.ceil() + 0.5) as u32;
        let fractional_grid_line = num_grid_lines_float - num_animation_grid_lines as f32;

        // Y-axis labels
        let mut max_label_width = px(40.0);
        let mut y_axis_labels = Vec::with_capacity(num_animation_grid_lines as usize + 1);
        for (idx, y) in (0..=num_animation_grid_lines).map(|idx| {
            (
                idx,
                px(idx as f32 + fractional_grid_line) * grid_line_spacing,
            )
        }) {
            // Handle out of bounds from resize animation
            let opacity = ease_in_out((y / grid_line_spacing + 1.0).clamp(0.0, 1.0));
            let y = y.max(px(0.0));

            // Paint text
            window.with_text_style(
                Some(TextStyleRefinement {
                    color: Some(Hsla::from(cx.theme().base.foreground).opacity(opacity)),
                    ..Default::default()
                }),
                |window| {
                    let mut label = format!(
                        "{}",
                        (num_animation_grid_lines - idx) as f64 * units_per_grid_line as f64
                    )
                    .into_any_element();
                    let label_size = label.layout_as_root(LABEL_AVAILABLE_SPACE, window, cx);
                    max_label_width = max_label_width.max(label_size.width);

                    label.prepaint_at(
                        content_bounds.origin + point(px(0.0), y - label_size.height / 2.0),
                        window,
                        cx,
                    );
                    y_axis_labels.push(label);
                },
            );
        }

        content_bounds.size.width -= max_label_width * 2.0;
        content_bounds.origin.x += max_label_width;

        // Create path
        let scaled_points: Vec<Point<Pixels>> = self
            .points
            .iter()
            .map(|point| {
                Point {
                    x: px(point.x) / data_range.x * content_bounds.size.width,
                    y: px(point.y) / scale * -content_bounds.size.height,
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

            // Control points
            let segment = point_b - point_a;
            let dot_a = segment.x * tangent_a.x + segment.y * tangent_a.y;
            let dot_b = segment.x * tangent_b.x + segment.y * tangent_b.y;

            let smoothing = 1.0 / 3.0;
            let control_a = point_a + tangent_a * dot_a * smoothing;
            let control_b = point_b - tangent_b * dot_b * smoothing;

            path.cubic_bezier_to(point_b, control_a, control_b);
        }

        Prepaint {
            content_bounds,
            num_animation_grid_lines,
            grid_line_spacing,
            fractional_grid_line,
            points: scaled_points,
            path: Some(path.build().unwrap()),
            y_axis_labels,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let grid_lines_finish = 0.75;
        let main_line_delay = 0.25;

        // Grid lines
        for y in (0..prepaint.num_animation_grid_lines)
            .map(|idx| px(idx as f32 + prepaint.fractional_grid_line) * prepaint.grid_line_spacing)
        {
            // Handle out of bounds from resize animation
            let opacity = ease_in_out((y / prepaint.grid_line_spacing + 1.0).clamp(0.0, 1.0));
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
        let path_progress =
            ((self.animation_progress - main_line_delay) / (1.0 - main_line_delay)).clamp(0.0, 1.0);
        window.with_content_mask(
            Some(ContentMask {
                bounds: Bounds {
                    origin: prepaint.content_bounds.origin,
                    size: size(
                        prepaint.content_bounds.size.width * path_progress,
                        prepaint.content_bounds.size.height,
                    ),
                },
            }),
            |window| {
                window.paint_path(prepaint.path.take().unwrap(), cx.theme().base.foreground);
            },
        );

        for point in &prepaint.points {
            if point.x
                > prepaint.content_bounds.size.width * path_progress
                    + prepaint.content_bounds.origin.x
            {
                break;
            }

            window.paint_quad(PaintQuad {
                bounds: Bounds::centered_at(*point, size(px(8.0), px(8.0))),
                corner_radii: Corners {
                    top_left: px(4.0),
                    top_right: px(4.0),
                    bottom_right: px(4.0),
                    bottom_left: px(4.0),
                },
                background: cx.theme().base.foreground.into(),
                border_widths: Edges::default(),
                border_color: gpui::transparent_black(),
            });
        }

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

        for label in &mut prepaint.y_axis_labels {
            label.paint(window, cx);
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
            _ if (points[idx + 1].y > points[idx].y) == (points[idx - 1].y > points[idx].y) => {
                Point {
                    x: px(1.0),
                    y: px(0.0),
                }
            }
            _ => (points[idx + 1] - points[idx - 1]) * 0.5,
        };

        let factor = px(1.0 / unnormalized.magnitude() as f32);
        tangents.push(unnormalized * factor);
    }

    tangents
}
