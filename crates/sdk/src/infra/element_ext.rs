//! Shared GPUI element helpers used across controls and layouts.

use gpui::{App, Bounds, Corners, ParentElement, Pixels, Refineable as _, StyleRefinement, Styled, Window, canvas};

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
