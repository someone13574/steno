use gpui::{px, rgb, rgba, App, Context, Global, Pixels, Rgba, Window, WindowAppearance};

#[derive(Clone, Copy)]
pub struct BaseTheme {
    pub animation_speed: f32,

    pub background: Rgba,
    pub border: Rgba,
    pub dim_foreground: Rgba,
    pub foreground: Rgba,
    pub hover_background: Rgba,

    pub border_width: Pixels,
    pub gap: Pixels,
    pub radius_large: Pixels,
    pub shadow_size_large: Pixels,

    pub font_family: &'static str,
}

impl BaseTheme {
    pub fn default_light() -> Self {
        Self {
            animation_speed: 1.0,
            background: rgb(0xffffff),
            border: rgb(0xd0d0d0),
            dim_foreground: rgba(0x000000a0),
            foreground: rgb(0x505050),
            hover_background: rgb(0xe0e0e0),
            border_width: px(1.0),
            gap: px(8.0),
            radius_large: px(16.0),
            shadow_size_large: px(32.0),
            font_family: "Sans",
        }
    }

    pub fn default_dark() -> Self {
        Self {
            animation_speed: 1.0,
            background: rgb(0x202020),
            border: rgb(0x303030),
            dim_foreground: rgba(0xffffff20),
            foreground: rgb(0xe0e0e0),
            hover_background: rgb(0x303030),
            border_width: px(1.0),
            gap: px(8.0),
            radius_large: px(16.0),
            shadow_size_large: px(32.0),
            font_family: "Sans",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub base: BaseTheme,
    pub counter_idle_message: &'static str,
    pub counter_idle_text: Rgba,
    pub counter_font_family: &'static str,
    pub counter_text: Rgba,
    pub csd_button_background: Rgba,
    pub csd_button_background_hovered: Rgba,
    pub csd_button_foreground: Rgba,
    pub csd_button_gap: Pixels,
    pub csd_corner_radius: Pixels,
    pub csd_font_family: &'static str,
    pub csd_foreground: Rgba,
    pub csd_shadow_size: Pixels,
    pub csd_shadow_focused: Rgba,
    pub csd_shadow_unfocused: Rgba,
    pub csd_titlebar_background: Rgba,
    pub csd_titlebar_border: Rgba,
    pub csd_titlebar_border_width: Pixels,
    pub csd_titlebar_padding_x: Pixels,
    pub csd_titlebar_padding_y: Pixels,
    pub csd_window_border: Rgba,
    pub csd_window_border_width: Pixels,
    pub text_view_correct_text: Rgba,
    pub text_view_cursor: Rgba,
    pub text_view_incorrect_text: Rgba,
    pub text_view_placeholder_text: Rgba,
    pub window_background: Rgba,
}

impl Theme {
    pub fn default_light() -> Self {
        Self {
            ..Self::from(BaseTheme::default_light())
        }
    }

    pub fn default_dark() -> Self {
        Self {
            ..Self::from(BaseTheme::default_dark())
        }
    }

    pub fn set_light(self, window: &mut Window, cx: &mut App) {
        match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => {
                cx.set_global(self);
            }
            _ => {}
        }
        window
            .observe_window_appearance(move |window, cx| {
                match window.appearance() {
                    WindowAppearance::Light | WindowAppearance::VibrantLight => {
                        cx.set_global(self);
                        window.refresh();
                    }
                    _ => {}
                }
            })
            .detach();
    }

    pub fn set_dark(self, window: &mut Window, cx: &mut App) {
        match window.appearance() {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => {
                cx.set_global(self);
            }
            _ => {}
        }
        window
            .observe_window_appearance(move |window, cx| {
                match window.appearance() {
                    WindowAppearance::Dark | WindowAppearance::VibrantDark => {
                        cx.set_global(self);
                        window.refresh();
                    }
                    _ => {}
                }
            })
            .detach();
    }
}

impl From<BaseTheme> for Theme {
    fn from(base: BaseTheme) -> Self {
        Self {
            base,
            counter_idle_message: "Type to start...",
            counter_idle_text: base.dim_foreground,
            counter_font_family: base.font_family,
            counter_text: base.foreground,
            csd_button_background: base.background,
            csd_button_background_hovered: base.hover_background,
            csd_button_foreground: base.foreground,
            csd_button_gap: base.gap,
            csd_corner_radius: base.radius_large,
            csd_font_family: base.font_family,
            csd_foreground: base.foreground,
            csd_shadow_size: base.shadow_size_large,
            csd_shadow_focused: rgba(0x00000080),
            csd_shadow_unfocused: rgba(0x00000040),
            csd_titlebar_background: base.background,
            csd_titlebar_border: base.border,
            csd_titlebar_border_width: base.border_width,
            csd_titlebar_padding_x: base.radius_large / 2.0,
            csd_titlebar_padding_y: base.radius_large / 2.0,
            csd_window_border: base.border,
            csd_window_border_width: base.border_width,
            text_view_correct_text: base.foreground,
            text_view_cursor: base.foreground,
            text_view_incorrect_text: rgb(0xe23636),
            text_view_placeholder_text: base.dim_foreground,
            window_background: base.background,
        }
    }
}

impl Global for Theme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global()
    }
}

impl<T> ActiveTheme for Context<'_, T> {
    fn theme(&self) -> &Theme {
        self.global()
    }
}
