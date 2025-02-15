use gpui::{px, rgb, rgba, App, Context, Global, Pixels, Rgba};

pub struct BaseTheme {
    pub background: Rgba,
    pub border: Rgba,

    pub border_width: Pixels,
    pub radius_large: Pixels,
    pub shadow_size_large: Pixels,
}

impl BaseTheme {
    pub fn default_dark() -> Self {
        Self {
            background: rgb(0x202020),
            border: rgb(0x404040),
            border_width: px(1.0),
            radius_large: px(16.0),
            shadow_size_large: px(8.0),
        }
    }
}

pub struct Theme {
    pub csd_corner_radius: Pixels,
    pub csd_shadow_size: Pixels,
    pub csd_shadow_focused: Rgba,
    pub csd_shadow_unfocused: Rgba,
    pub csd_window_border: Rgba,
    pub csd_window_border_width: Pixels,
    pub window_background: Rgba,
}

impl From<BaseTheme> for Theme {
    fn from(base: BaseTheme) -> Self {
        Self {
            csd_corner_radius: base.radius_large,
            csd_shadow_size: base.shadow_size_large,
            csd_shadow_focused: rgba(0x00000080),
            csd_shadow_unfocused: rgba(0x00000040),
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
