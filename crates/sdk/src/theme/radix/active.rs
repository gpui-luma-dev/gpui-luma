use std::sync::{Arc, RwLock};

use super::RadixTheme;

static ACTIVE_RADIX_THEME: RwLock<Option<Arc<RadixTheme>>> = RwLock::new(None);

/// Register the app's Radix theme so SDK default templates resolve the current CSS catalog.
///
/// Call once at app startup, before spawning stock controls.
pub fn set_active_radix_theme(theme: Arc<RadixTheme>) {
    if let Ok(mut active) = ACTIVE_RADIX_THEME.write() {
        *active = Some(theme);
    }
}

pub(crate) fn active_radix_theme() -> Option<Arc<RadixTheme>> {
    ACTIVE_RADIX_THEME.read().ok().and_then(|active| active.as_ref().cloned())
}

#[cfg(test)]
pub(crate) fn clear_active_radix_theme_for_tests() {
    if let Ok(mut active) = ACTIVE_RADIX_THEME.write() {
        *active = None;
    }
}
