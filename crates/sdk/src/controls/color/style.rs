use crate::theme::ThemeTokens;
use gpui::{
    App, AppContext, Bounds, Corners, Hsla, ParentElement, Pixels, Refineable as _, StyleRefinement, Styled, Window,
    canvas,
};
use std::cell::RefCell;

thread_local! {
    static ACTIVE_COLOR_CONTROL_THEME: RefCell<Option<ColorControlTheme>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy, Debug)]
pub struct ColorControlTheme {
    pub border: Hsla,
    pub background: Hsla,
    dark: bool,
}

impl ColorControlTheme {
    pub fn new(border: Hsla, background: Hsla, dark: bool) -> Self {
        Self { border, background, dark }
    }

    pub fn is_dark(&self) -> bool {
        self.dark
    }
}

fn default_color_control_theme() -> ColorControlTheme {
    let tokens = ThemeTokens::default();
    let background = tokens.palette.surface.panel.background;

    ColorControlTheme { border: tokens.palette.border.default, background, dark: background.l < 0.5 }
}

pub fn set_active_color_control_theme(theme: ColorControlTheme) {
    ACTIVE_COLOR_CONTROL_THEME.with(|cell| {
        *cell.borrow_mut() = Some(theme);
    });
}

pub fn active_color_control_theme() -> ColorControlTheme {
    ACTIVE_COLOR_CONTROL_THEME
        .with(|cell| cell.borrow().as_ref().copied())
        .unwrap_or_else(default_color_control_theme)
}

pub trait ActiveTheme {
    fn theme(&self) -> ColorControlTheme;
}

impl<T: AppContext> ActiveTheme for T {
    fn theme(&self) -> ColorControlTheme {
        active_color_control_theme()
    }
}

pub trait ElementExt: ParentElement + Sized {
    fn on_prepaint<F>(self, f: F) -> Self
    where
        F: FnOnce(Bounds<Pixels>, &mut Window, &mut App) + 'static,
    {
        self.child(canvas(move |bounds, window, cx| f(bounds, window, cx), |_, _, _, _| {}).absolute().size_full())
    }
}

impl<T: ParentElement> ElementExt for T {}

pub trait StyledExt: Styled + Sized {
    fn refine_style(mut self, style: &StyleRefinement) -> Self {
        self.style().refine(style);
        self
    }

    fn corner_radii(self, radius: Corners<Pixels>) -> Self {
        self.rounded_tl(radius.top_left)
            .rounded_tr(radius.top_right)
            .rounded_bl(radius.bottom_left)
            .rounded_br(radius.bottom_right)
    }
}

impl<T: Styled> StyledExt for T {}

#[derive(Clone, Default, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    Size(Pixels),
    XSmall,
    Small,
    #[default]
    Medium,
    Large,
}

impl From<Pixels> for Size {
    fn from(value: Pixels) -> Self {
        Size::Size(value)
    }
}

pub trait Sizable: Sized {
    fn with_size(self, size: impl Into<Size>) -> Self;
}
