//! Shared ControlSize → font-size mapping from `button.metrics`.

use gpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{embedded_stylesheet, resolve_button_metrics_rule};

/// Scales typography from `button.metrics.*.font_size` (12 / 14 / 16).
pub fn apply_button_metrics_typography(typography: &mut LumaTextStyle, mode: &ShadcnModeTokens, size: ControlSize) {
    if let Some(rule) = embedded_stylesheet().button.metrics_for_size(size) {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }
}
