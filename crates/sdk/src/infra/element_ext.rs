//! Shared GPUI element helpers used across controls and layouts.

use gpui::{App, Bounds, Corners, ParentElement, Pixels, Refineable as _, StyleRefinement, Styled, Window, canvas};

pub trait ElementExt: ParentElement + Sized {
    fn on_prepaint<F>(self, f: F) -> Self
    where
        F: FnOnce(Bounds<Pixels>, &mut Window, &mut App) + 'static,
    {
        self.child(
            canvas(move |bounds, window, cx| f(bounds, window, cx), |_, _, _, _| {})
                .absolute()
                .inset_0()
                .size_full(),
        )
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

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use gpui::{Context, IntoElement, Render, TestAppContext, div, prelude::*, px};

    struct BoundsProbe(Arc<Mutex<Option<Bounds<Pixels>>>>);
    impl Render for BoundsProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let measured = self.0.clone();
            div()
                .relative()
                .w(px(80.0))
                .h(px(40.0))
                .debug_selector(|| "bounds-owner".into())
                .child(div().w(px(20.0)).h(px(10.0)))
                .on_prepaint(move |bounds, _, _| {
                    *measured.lock().unwrap() = Some(bounds);
                })
        }
    }

    #[test]
    fn prepaint_measures_owner_instead_of_static_position_after_content() {
        let mut app = TestAppContext::single();
        let measured = Arc::new(Mutex::new(None));
        let (_, cx) = app.add_window_view(|_, _| BoundsProbe(measured.clone()));
        cx.run_until_parked();
        assert_eq!(*measured.lock().unwrap(), cx.debug_bounds("bounds-owner"));
    }
}
