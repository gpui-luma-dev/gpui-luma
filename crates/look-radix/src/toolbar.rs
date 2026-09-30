//! Radix chrome for the SDK toolbar; hosted controls keep their own Radix looks.

use std::sync::Arc;

use luma::controls::toolbar::{ToolbarLook, ToolbarTemplate, ToolbarTheme, ToolbarVariant, toolbar_template_with_theme};
use luma::theme::ControlSize;

use crate::{Look, ScaleFamily};

struct RadixToolbarTheme {
    look: Look,
}

impl ToolbarTheme for RadixToolbarTheme {
    fn resolve(&self, enabled: bool, size: ControlSize, variant: ToolbarVariant) -> ToolbarLook {
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        let transparent = gpui::hsla(0.0, 0.0, 0.0, 0.0);
        let metrics = self.look.metrics();
        let control = metrics.for_size(size);
        let (background, border) = match variant {
            ToolbarVariant::Outline => (gray(if enabled { 1 } else { 2 }), gray(6)),
            ToolbarVariant::Ghost => (transparent, transparent),
        };
        ToolbarLook {
            background,
            border,
            separator: gray(6),
            radius: metrics.radius.md,
            padding_x: 4.0,
            padding_y: 3.0,
            gap: 6.0,
            separator_height: control.height * 0.65,
        }
    }
}

/// Live toolbar shell colors, following mode and custom palette changes.
pub fn toolbar_theme(look: &Look) -> Arc<dyn ToolbarTheme> {
    Arc::new(RadixToolbarTheme { look: look.clone() })
}

/// Theme the SDK toolbar while retaining its hosted controls and keyboard navigation.
pub fn toolbar_template(look: &Look) -> Arc<dyn ToolbarTemplate> {
    toolbar_template_with_theme(toolbar_theme(look))
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::theme::ThemeMode;

    #[test]
    fn toolbar_tracks_mode_and_preserves_ghost_transparency() {
        let look = Look::built_in();
        let theme = toolbar_theme(&look);
        let light = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
        assert_ne!(light.background, dark.background);
        assert_eq!(dark.border, look.resolve_step(ScaleFamily::Gray, 6).hsla());
        let ghost = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Ghost);
        assert_eq!(ghost.background.a, 0.0);
        assert_eq!(ghost.border.a, 0.0);
    }
}
