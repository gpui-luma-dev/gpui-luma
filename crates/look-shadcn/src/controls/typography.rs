//! Shared ControlSize → font-size mapping from common geometry with legacy metric fallback.

use gpui_luma::theme::{ControlSize, LumaTextStyle};

use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{resolve_button_metrics_rule};

/// Preserve the supplied typography ratio while applying configured button sizes.
pub fn apply_button_metrics_typography(typography: &mut LumaTextStyle, mode: &ShadcnModeTokens, size: ControlSize) {
    if let Some(rule) = mode.stylesheet().button.metrics_for_size(size) {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = typography.size;
        typography.size = metrics.font_size;
        if base_size > 0.0 {
            typography.line_height = metrics.font_size * (typography.line_height / base_size);
        }
    }
    apply_common_button_typography(typography, mode.stylesheet(), size);
}

pub(crate) fn apply_common_button_typography(
    typography: &mut LumaTextStyle,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
) {
    let (font, line, _, _) = crate::tables::typography::common_typography_overrides(stylesheet, size, false);
    if let Some(font) = font {
        let ratio = if typography.size > 0.0 {
            typography.line_height / typography.size
        } else {
            1.0
        };
        typography.size = font;
        typography.line_height = font * ratio;
    }
    if let Some(line) = line {
        typography.line_height = line;
    }
}
