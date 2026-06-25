use gpui::BoxShadow;
use gpui_luma::theme::{LumaElevation, ThemeMode};

pub(crate) fn thumb_shadow(theme_mode: ThemeMode) -> Vec<BoxShadow> {
    match theme_mode {
        ThemeMode::Light => LumaElevation::light().thumb.to_box_shadows(),
        ThemeMode::Dark => LumaElevation::dark().thumb.to_box_shadows(),
    }
}
