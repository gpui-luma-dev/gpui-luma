use gpui::{BorrowAppContext, Context, Global, Subscription};

/// App-global epoch used to invalidate persistent control entities after a theme change.
#[derive(Default)]
pub struct LumaThemeRevision(u64);

impl Global for LumaThemeRevision {}

pub trait LumaThemeSyncExt: BorrowAppContext {
    fn bump_luma_theme_revision(&mut self) {
        self.update_default_global(|revision: &mut LumaThemeRevision, _| {
            revision.0 = revision.0.wrapping_add(1);
        });
    }
}

impl<C> LumaThemeSyncExt for C where C: BorrowAppContext {}

pub fn observe_theme_revision<T: 'static>(
    cx: &mut Context<T>,
    f: impl FnMut(&mut T, &mut Context<T>) + 'static,
) -> Subscription {
    cx.observe_global::<LumaThemeRevision>(f)
}
