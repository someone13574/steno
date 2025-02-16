use gpui::prelude::*;
use gpui::{
    point, relative, size, AnyElement, App, AvailableSpace, Bounds, ElementId, GlobalElementId,
    LayoutId, Length, Pixels, Style, Window,
};

pub struct Clamp {
    child: AnyElement,
    max_width: Pixels,
    linear_at: Pixels,
    smoothing: f32,
}

pub fn clamp(
    max_width: Pixels,
    linear_at: Pixels,
    smoothing: f32,
    child: impl IntoElement,
) -> Clamp {
    Clamp {
        child: child.into_element().into_any(),
        max_width,
        linear_at,
        smoothing,
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
        let size = size(
            smooth_max(
                self.max_width,
                self.linear_at,
                self.smoothing,
                bounds.size.width,
            ),
            bounds.size.height,
        );
        let offset = (bounds.size - size) / 2.0;

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
    if available < linear_at {
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
