use gpui::prelude::*;
use gpui::{
    bounds, ease_in_out, fill, point, px, relative, size, AnyElement, App, AvailableSpace, Bounds,
    Element, ElementId, GlobalElementId, Hsla, LayoutId, Pixels, SharedString, Size, Style, Window,
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

impl Element for LineChart {
    type PrepaintState = Bounds<Pixels>;
    type RequestLayoutState = RequestLayout;

    fn id(&self) -> Option<ElementId> {
        None
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

        Bounds {
            origin: bounds.origin,
            size: size(bounds.size.width, bounds.size.height - vertical_offset),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        content_bounds: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let line_time = 0.5;

        // Horizontal grid lines
        let num_grid_lines = (content_bounds.size.height / self.grid_lines_spacing + 0.5) as u32;
        for y in (0..num_grid_lines).map(|idx| {
            px(idx as f32) * content_bounds.size.height / (num_grid_lines as f32).max(1.0)
        }) {
            let start_time = (1.0 - y / content_bounds.size.height) * (1.0 - line_time);
            let progress = (self.animation_progress - start_time) / line_time;
            if progress < 0.0 {
                continue;
            }

            let width = content_bounds.size.width * ease_in_out(progress.clamp(0.0, 1.0));
            window.paint_quad(fill(
                gpui::bounds(
                    content_bounds.origin + point(px(0.0), y),
                    size(width, px(1.0)),
                ),
                cx.theme().base.dim_foreground,
            ));
        }

        // Axes
        window.paint_quad(fill(
            gpui::bounds(
                content_bounds.bottom_left(),
                size(content_bounds.size.width, px(2.0)),
            ),
            cx.theme().base.foreground,
        ));
        window.paint_quad(fill(
            gpui::bounds(
                content_bounds.origin,
                size(px(2.0), content_bounds.size.height),
            ),
            cx.theme().base.foreground,
        ));

        if let Some(x_axis_label) = request_layout.x_axis_label.as_mut() {
            x_axis_label.paint(window, cx);
        }
    }
}
