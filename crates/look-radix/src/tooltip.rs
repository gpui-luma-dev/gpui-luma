//! Theme-aware tooltip defaults, bound to the same shared/forked look as the control.
use std::sync::Arc;
use gpui_luma::controls::tooltip::{TooltipLook, TooltipTheme};
use crate::{Look, SemanticRole};

struct RadixTooltipTheme(Look);

impl TooltipTheme for RadixTooltipTheme {
    fn resolve(&self) -> TooltipLook {
        let metrics = self.0.metrics();
        let mut resolved = TooltipLook::compact(
            self.0.resolve_role(SemanticRole::Foreground).hsla(),
            self.0.resolve_role(SemanticRole::Background).hsla(),
            metrics.radius.md,
        );
        let geometry = self.0.common_stylesheet().tooltip.resolve_geometry(
            "",
            gpui_luma::theme::stylesheet::TooltipGeometry {
                padding: resolved.padding,
                padding_y: resolved.padding_y,
                max_width: resolved.max_width,
                font_size: resolved.text_size,
                line_height: resolved.line_height,
            },
        );
        resolved.padding = geometry.padding.value_px;
        resolved.padding_y = geometry.padding_y.value_px;
        resolved.max_width = geometry.max_width.value_px;
        resolved.text_size = geometry.font_size.value_px;
        resolved.line_height = geometry.line_height.value_px;
        resolved
    }
}

/// Resolve tooltip presentation from this look on every render, including mode/palette changes.
pub fn tooltip_theme(look: &Look) -> Arc<dyn TooltipTheme> {
    Arc::new(RadixTooltipTheme(look.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ThemeMode;

    #[test]
    fn tooltip_tracks_bound_fork_and_mode_changes() {
        let ambient = Look::built_in();
        let fork = ambient.fork();
        let theme = tooltip_theme(&fork);
        let light = theme.resolve();
        fork.set_mode(ThemeMode::Dark);
        let dark = theme.resolve();
        assert_ne!(light.background, dark.background);
        assert_eq!(ambient.mode(), ThemeMode::Light);
        assert_eq!(dark.background, fork.resolve_role(SemanticRole::Foreground).hsla());
        assert_eq!(dark.foreground, fork.resolve_role(SemanticRole::Background).hsla());
    }
}
