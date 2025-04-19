use std::time::Instant;

use gpui::prelude::*;
use gpui::{AnyElement, App, Bounds, ElementId, GlobalElementId, LayoutId, Pixels, Window};

use crate::theme::ActiveTheme;

type ContinuousAnimationFn<E, S> = Box<dyn Fn(E, &mut S, f32, &mut Window, &mut App) -> (E, bool)>;

pub trait ContinuousAnimationExt {
    fn with_continuous_animation<S>(
        self,
        id: impl Into<ElementId>,
        initial_state: S,
        animator: impl Fn(Self, &mut S, f32, &mut Window, &mut App) -> (Self, bool) + 'static,
    ) -> ContinuousAnimation<Self, S>
    where
        Self: Sized,
    {
        ContinuousAnimation::<Self, S> {
            id: id.into(),
            element: Some(self),
            initial_state,
            animator: Box::new(animator),
        }
    }
}

impl<E> ContinuousAnimationExt for E {}

pub struct ContinuousAnimation<E, S> {
    id: ElementId,
    element: Option<E>,
    initial_state: S,
    animator: ContinuousAnimationFn<E, S>,
}

impl<E: IntoElement + 'static, S: Clone + 'static> IntoElement for ContinuousAnimation<E, S> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: IntoElement + 'static, S: Clone + 'static> Element for ContinuousAnimation<E, S> {
    type PrepaintState = ();
    type RequestLayoutState = AnyElement;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_element_state(id.unwrap(), |state: Option<(Instant, S)>, window| {
            let (last_frame, mut state) =
                state.unwrap_or((Instant::now(), self.initial_state.clone()));
            let delta = last_frame.elapsed().as_secs_f32() * cx.theme().base.animation_speed;

            let (element, animate) =
                (self.animator)(self.element.take().unwrap(), &mut state, delta, window, cx);
            let mut element = element.into_any_element();

            if animate {
                window.request_animation_frame();
            }

            (
                (element.request_layout(window, cx), element),
                (Instant::now(), state),
            )
        })
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}
