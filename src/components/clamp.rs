use gpui::prelude::*;
use gpui::{
    point, px, relative, size, AnyElement, App, AvailableSpace, Axis, Bounds, ElementId,
    GlobalElementId, LayoutId, Length, Percentage, Pixels, Style, Window,
};

pub struct Clamp {
    child: AnyElement,
    max_size: Pixels,
    linear_at: Pixels,
    smoothing: f32,
    axis: Axis,
    position: Percentage,
}

impl Clamp {
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
        self
    }

    pub fn smoothing(mut self, smoothing: f32) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub fn position(mut self, percentage: Percentage) -> Self {
        self.position = percentage;
        self
    }
}

pub fn clamp(max_size: Pixels, linear_at: Pixels, child: impl IntoElement) -> Clamp {
    Clamp {
        child: child.into_element().into_any(),
        max_size,
        linear_at,
        smoothing: 2.0,
        axis: Axis::Horizontal,
        position: Percentage(0.5),
    }
}

impl IntoElement for Clamp {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Clamp {
    type PrepaintState = ();
    type RequestLayoutState = ();

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
            size: size(
                Length::Definite(relative(1.0)),
                Length::Definite(relative(1.0)),
            ),
            ..Default::default()
        };

        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let clamped_size = smooth_max(
            self.max_size,
            self.linear_at,
            self.smoothing,
            match self.axis {
                Axis::Vertical => bounds.size.height,
                Axis::Horizontal => bounds.size.width,
            },
        );
        let size = match self.axis {
            Axis::Vertical => size(bounds.size.width, clamped_size),
            Axis::Horizontal => size(clamped_size, bounds.size.height),
        };
        let offset = (bounds.size - size) * px(self.position.0);

        self.child.prepaint_as_root(
            bounds.origin + point(offset.width, offset.height),
            size.map(AvailableSpace::Definite),
            window,
            cx,
        );
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

fn smooth_max(max: Pixels, linear_at: Pixels, k: f32, available: Pixels) -> Pixels {
    if available <= linear_at {
        return available;
    } else if available > max * k + max {
        return max;
    }

    // Use inverse bezier to get lerp from available
    let determinant =
        (max - linear_at).pow(2.0) + (available - linear_at) * (linear_at + k * max - max);
    let delta = (available - linear_at) / (max - linear_at + determinant.pow(0.5));
    let inverse_delta = 1.0 - delta;

    // Forward bezier to get width
    let lerp = max * delta + linear_at * inverse_delta;
    lerp * inverse_delta + max * delta
}
