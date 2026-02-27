use gpui::prelude::*;
use gpui::{
    div, point, px, size, transparent_black, App, BoxShadow, CursorStyle, Decorations, Div, Entity,
    FocusHandle, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels, Point, ResizeEdge, Result,
    Size, Tiling, TitlebarOptions, Window, WindowDecorations, WindowHandle, WindowOptions,
};

use crate::theme::{ActiveTheme, Theme};
use crate::titlebar::Titlebar;
use crate::APP_ID;

pub const CSD_RESIZE_EDGE_SIZE: Pixels = px(16.0);

pub struct StenoWindow<V: Render> {
    main_view: Entity<V>,
    titlebar: Entity<Titlebar<V>>,
    focus_handle: FocusHandle,
    cursor_style: CursorStyle,
    pub active_csd_event: bool,
}

impl<V: Render> StenoWindow<V> {
    pub fn new(
        cx: &mut App,
        build_root_view: impl FnOnce(FocusHandle, &mut Window, &mut Context<Self>) -> Entity<V>,
    ) -> Result<WindowHandle<Self>> {
        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Steno".into()),
                ..Default::default()
            }),
            app_id: Some(APP_ID.to_string()),
            window_min_size: Some(size(px(300.0), px(275.0))),
            window_decorations: Some(if cfg!(target_os = "linux") {
                WindowDecorations::Client
            } else {
                WindowDecorations::Server
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |window, cx| {
            cx.new(|cx| {
                let focus_handle = cx.focus_handle();

                Self {
                    main_view: build_root_view(focus_handle.clone(), window, cx),
                    titlebar: Titlebar::new(cx.entity(), cx),
                    focus_handle,
                    cursor_style: CursorStyle::default(),
                    active_csd_event: false,
                }
            })
        })
    }
}

impl<V: Render> Render for StenoWindow<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let decorations = window.window_decorations();
        let client_inset = cx.theme().csd.shadow_size.max(CSD_RESIZE_EDGE_SIZE);

        if let Decorations::Client { .. } = decorations {
            window.set_client_inset(client_inset);
        }

        div()
            .track_focus(&self.focus_handle)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, window, cx| {
                    this.focus_handle.focus(window, cx);
                }),
            )
            .size_full()
            .bg(transparent_black())
            .map(|element| {
                match decorations {
                    Decorations::Server => {
                        element
                            .bg(cx.theme().window_background)
                            .overflow_hidden()
                            .child(self.main_view.clone())
                    }
                    Decorations::Client { tiling } => {
                        element
                            .when(!tiling.top, |div| div.pt(client_inset))
                            .when(!tiling.bottom, |div| div.pb(client_inset))
                            .when(!tiling.left, |div| div.pl(client_inset))
                            .when(!tiling.right, |div| div.pr(client_inset))
                            .cursor(self.cursor_style)
                            .on_mouse_move(cx.listener(
                                move |this, event: &MouseMoveEvent, window, cx| {
                                    let new_cursor = if let Some(edge) = get_resize_edge(
                                        event.position,
                                        window.bounds().size,
                                        tiling,
                                        cx.theme().csd.shadow_size,
                                    ) {
                                        resize_edge_cursor(edge)
                                    } else {
                                        CursorStyle::default()
                                    };

                                    if this.cursor_style != new_cursor || this.active_csd_event {
                                        this.cursor_style = new_cursor;
                                        this.active_csd_event = false;
                                        cx.notify();
                                    }
                                },
                            ))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                    let new_cursor = if let Some(edge) = get_resize_edge(
                                        event.position,
                                        window.bounds().size,
                                        tiling,
                                        cx.theme().csd.shadow_size,
                                    ) {
                                        window.start_window_resize(edge);
                                        this.active_csd_event = true;

                                        resize_edge_cursor(edge)
                                    } else {
                                        CursorStyle::default()
                                    };

                                    if this.cursor_style != new_cursor || this.active_csd_event {
                                        this.cursor_style = new_cursor;
                                        cx.notify();
                                    }
                                }),
                            )
                            .child(csd_div(
                                tiling,
                                window.is_window_active() || self.active_csd_event,
                                cx.theme(),
                                self.titlebar.clone(),
                                self.main_view.clone(),
                            ))
                    }
                }
            })
    }
}

fn get_resize_edge(
    position: Point<Pixels>,
    outer_window_size: Size<Pixels>,
    tiling: Tiling,
    shadow_size: Pixels,
) -> Option<ResizeEdge> {
    let resize_edge_outer_inset = (shadow_size - CSD_RESIZE_EDGE_SIZE).max(px(0.0));
    let window_bounds_inset = resize_edge_outer_inset + CSD_RESIZE_EDGE_SIZE;
    let resize_edge_inner_inset = window_bounds_inset + CSD_RESIZE_EDGE_SIZE;

    // Check for window obstruction
    if (tiling.top || position.y > window_bounds_inset)
        && (tiling.bottom || position.y < outer_window_size.height - window_bounds_inset)
        && (tiling.left || position.x > window_bounds_inset)
        && (tiling.right || position.x < outer_window_size.width - window_bounds_inset)
    {
        return None;
    }

    // Get resize edge
    let top = !tiling.top && position.y > resize_edge_outer_inset;
    let bottom = !tiling.bottom && position.y < outer_window_size.height - resize_edge_outer_inset;
    let left = !tiling.left && position.x > resize_edge_outer_inset;
    let right = !tiling.right && position.x < outer_window_size.width - resize_edge_outer_inset;

    let top_inner = !tiling.top && position.y < resize_edge_inner_inset;
    let bottom_inner =
        !tiling.bottom && position.y > outer_window_size.height - resize_edge_inner_inset;
    let left_inner = !tiling.left && position.x < resize_edge_inner_inset;
    let right_inner =
        !tiling.right && position.x > outer_window_size.width - resize_edge_inner_inset;

    match (
        top,
        bottom,
        left,
        right,
        top_inner,
        bottom_inner,
        left_inner,
        right_inner,
    ) {
        (true, _, true, _, true, _, true, _) => Some(ResizeEdge::TopLeft),
        (true, _, _, true, true, _, _, true) => Some(ResizeEdge::TopRight),
        (_, true, true, _, _, true, true, _) => Some(ResizeEdge::BottomLeft),
        (_, true, _, true, _, true, _, true) => Some(ResizeEdge::BottomRight),
        (true, _, _, _, true, _, false, false) => Some(ResizeEdge::Top),
        (_, true, _, _, _, true, false, false) => Some(ResizeEdge::Bottom),
        (_, _, true, _, false, false, true, _) => Some(ResizeEdge::Left),
        (_, _, _, true, false, false, _, true) => Some(ResizeEdge::Right),
        _ => None,
    }
}

fn resize_edge_cursor(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

fn csd_div<T: Render>(
    tiling: Tiling,
    focused: bool,
    theme: &Theme,
    titlebar: Entity<Titlebar<T>>,
    child: impl IntoElement,
) -> Div {
    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(theme.window_background)
        .shadow(vec![BoxShadow {
            color: if focused {
                theme.csd.shadow_focused.into()
            } else {
                theme.csd.shadow_unfocused.into()
            },
            offset: point(px(0.0), px(0.0)),
            blur_radius: theme.csd.shadow_size / 4.0,
            spread_radius: px(0.0),
        }])
        .border_color(theme.csd.window_border)
        .when(!tiling.top, |div| {
            div.border_t(theme.csd.window_border_width)
        })
        .when(!tiling.bottom, |div| {
            div.border_b(theme.csd.window_border_width)
        })
        .when(!tiling.left, |div| {
            div.border_l(theme.csd.window_border_width)
        })
        .when(!tiling.right, |div| {
            div.border_r(theme.csd.window_border_width)
        })
        .when(!tiling.top && !tiling.left, |div| {
            div.rounded_tl(theme.csd.corner_radius)
        })
        .when(!tiling.top && !tiling.right, |div| {
            div.rounded_tr(theme.csd.corner_radius)
        })
        .when(!tiling.bottom && !tiling.left, |div| {
            div.rounded_bl(theme.csd.corner_radius)
        })
        .when(!tiling.bottom && !tiling.right, |div| {
            div.rounded_br(theme.csd.corner_radius)
        })
        .cursor_default()
        .on_mouse_move(|_, _, cx| cx.stop_propagation())
        .child(titlebar)
        .child(div().size_full().overflow_hidden().child(child))
}
