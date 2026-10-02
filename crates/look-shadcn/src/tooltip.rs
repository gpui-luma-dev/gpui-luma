//! Tooltip presentation bound to the control's mutable Shadcn look.
use std::sync::Arc;
use gpui_luma::controls::tooltip::{TooltipLook, TooltipTheme};
use crate::{ShadcnLook, ShadcnToken, ShadcnRadius};
struct ShadcnTooltipTheme(ShadcnLook);
impl TooltipTheme for ShadcnTooltipTheme {
    fn resolve(&self) -> TooltipLook {
        TooltipLook {
            background: self.0.color(ShadcnToken::Foreground),
            foreground: self.0.color(ShadcnToken::Background),
            padding: 8.0,
            padding_y: 4.0,
            radius: self.0.radius(ShadcnRadius::Md),
            max_width: 260.0,
            text_size: 12.0,
            line_height: 16.0,
            shadow: Vec::new(),
        }
    }
}
/// Resolve tooltip tokens on each presentation, including mode changes.
pub fn tooltip_theme(look: &ShadcnLook) -> Arc<dyn TooltipTheme> {
    Arc::new(ShadcnTooltipTheme(look.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ThemeMode;
    #[test]
    fn tooltip_tracks_bound_look_mode() {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("bundled CSS");
        let theme = tooltip_theme(&look);
        let light = theme.resolve();
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve();
        assert_ne!(light.background, dark.background);
        assert_eq!(dark.background, look.color(ShadcnToken::Foreground));
        assert_eq!(dark.foreground, look.color(ShadcnToken::Background));
    }
}
