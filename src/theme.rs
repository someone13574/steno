use gpui::{px, rgb, rgba, App, Context, Global, Pixels, Rgba};

pub struct BaseTheme {
    pub background: Rgba,
    pub border: Rgba,
    pub foreground: Rgba,

    pub border_width: Pixels,
    pub gap: Pixels,
    pub radius_large: Pixels,
    pub shadow_size_large: Pixels,

    pub font_family: &'static str,
}

impl BaseTheme {
    pub fn default_dark() -> Self {
        Self {
            background: rgb(0x202020),
            border: rgb(0x404040),
            foreground: rgb(0xe0e0e0),
            border_width: px(1.0),
            gap: px(8.0),
            radius_large: px(16.0),
            shadow_size_large: px(8.0),
            font_family: "Sans",
        }
    }
}

pub struct Theme {
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
    pub window_background: Rgba,
}

impl From<BaseTheme> for Theme {
    fn from(base: BaseTheme) -> Self {
        Self {
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
