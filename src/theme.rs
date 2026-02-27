use gpui::{px, rgb, rgba, App, Context, Global, Pixels, Rgba, Window, WindowAppearance};

#[derive(Clone, Copy)]
pub struct BaseTheme {
    pub animation_speed: f32,

    pub background: Rgba,
    #[cfg(not(target_family = "wasm"))]
    pub border: Rgba,
    pub dim_foreground: Rgba,
    pub foreground: Rgba,
    pub hover_background: Rgba,

    #[cfg(not(target_family = "wasm"))]
    pub border_width: Pixels,
    #[cfg(not(target_family = "wasm"))]
    pub gap: Pixels,
    pub radius_large: Pixels,
    #[cfg(not(target_family = "wasm"))]
    pub shadow_size_large: Pixels,

    pub font_family: &'static str,
}

impl BaseTheme {
    pub fn default_light() -> Self {
        Self {
            animation_speed: 1.0,
            background: rgb(0xffffff),
            #[cfg(not(target_family = "wasm"))]
            border: rgb(0xd0d0d0),
            dim_foreground: rgba(0x000000a0),
            foreground: rgb(0x202020),
            hover_background: rgb(0xe0e0e0),
            #[cfg(not(target_family = "wasm"))]
            border_width: px(1.0),
            #[cfg(not(target_family = "wasm"))]
            gap: px(8.0),
            radius_large: px(16.0),
            #[cfg(not(target_family = "wasm"))]
            shadow_size_large: px(32.0),
            font_family: "Sans",
        }
    }

    pub fn default_dark() -> Self {
        Self {
            animation_speed: 1.0,
            background: rgb(0x202020),
            #[cfg(not(target_family = "wasm"))]
            border: rgb(0x303030),
            dim_foreground: rgba(0xffffff20),
            foreground: rgb(0xe0e0e0),
            hover_background: rgb(0x303030),
            #[cfg(not(target_family = "wasm"))]
            border_width: px(1.0),
            #[cfg(not(target_family = "wasm"))]
            gap: px(8.0),
            radius_large: px(16.0),
            #[cfg(not(target_family = "wasm"))]
            shadow_size_large: px(32.0),
            font_family: "Sans",
        }
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Clone, Copy)]
pub struct CsdTheme {
    pub button_background: Rgba,
    pub button_background_hovered: Rgba,
    pub button_foreground: Rgba,
    pub button_gap: Pixels,
    pub corner_radius: Pixels,
    pub font_family: &'static str,
    pub foreground: Rgba,
    pub shadow_size: Pixels,
    pub shadow_focused: Rgba,
    pub shadow_unfocused: Rgba,
    pub titlebar_background: Rgba,
    pub titlebar_border: Rgba,
    pub titlebar_border_width: Pixels,
    pub titlebar_padding_x: Pixels,
    pub titlebar_padding_y: Pixels,
    pub window_border: Rgba,
    pub window_border_width: Pixels,
}

#[cfg(not(target_family = "wasm"))]
impl From<BaseTheme> for CsdTheme {
    fn from(base: BaseTheme) -> Self {
        Self {
            button_background: base.background,
            button_background_hovered: base.hover_background,
            button_foreground: base.foreground,
            button_gap: base.gap,
            corner_radius: base.radius_large,
            font_family: base.font_family,
            foreground: base.foreground,
            shadow_size: base.shadow_size_large,
            shadow_focused: rgba(0x00000080),
            shadow_unfocused: rgba(0x00000040),
            titlebar_background: base.background,
            titlebar_border: base.border,
            titlebar_border_width: base.border_width,
            titlebar_padding_x: base.radius_large / 2.0,
            titlebar_padding_y: base.radius_large / 2.0,
            window_border: base.border,
            window_border_width: base.border_width,
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
    #[cfg(not(target_family = "wasm"))]
    pub csd: CsdTheme,
    pub text_view_correct_text: Rgba,
    pub text_view_cursor: Rgba,
    pub text_view_incorrect_text: Rgba,
    pub text_view_placeholder_text: Rgba,
    pub window_background: Rgba,
}

impl Theme {
    pub fn default_light() -> Self {
        Self {
            text_view_incorrect_text: rgb(0xf44336),
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
            #[cfg(not(target_family = "wasm"))]
            csd: CsdTheme::from(base),
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
