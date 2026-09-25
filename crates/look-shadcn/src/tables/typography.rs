//! Resolved runtime text style and its source metadata.
use gpui::SharedString;
use luma::theme::{ControlSize, LumaTextStyle};
use crate::{ShadcnModeTokens, TypographySource};
use crate::stylesheet::{StylesheetConfig, embedded_stylesheet, resolve_button_metrics_rule};

#[derive(Clone, Debug)]
pub struct ControlTypographyTable {
    pub style: LumaTextStyle,
    pub font_family: SharedString,
    pub font_family_source: TypographySource,
    pub font_size_source: TypographySource,
    pub font_weight_source: TypographySource,
    pub line_height_source: TypographySource,
}

/// Label typography for button-sized controls, including Accordion.
pub fn resolve_control_typography(mode: &ShadcnModeTokens, size: ControlSize, toggle: bool) -> ControlTypographyTable {
    resolve_control_typography_with_stylesheet(mode, embedded_stylesheet(), size, toggle)
}

pub(crate) fn resolve_control_typography_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    toggle: bool,
) -> ControlTypographyTable {
    let mut style = mode.typography.text.label;
    let mut font_size_source = TypographySource::Scaffold { path: "LumaTypography.text.label.size".into() };
    let mut line_height_source = TypographySource::Scaffold { path: "LumaTypography.text.label.line_height".into() };
    let rule = if toggle {
        stylesheet.toggle.metrics_for_size(size)
    } else {
        stylesheet.button.metrics_for_size(size)
    };
    if let Some(rule) = rule {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = style.size;
        style.size = metrics.font_size;
        let family = if toggle { "toggle" } else { "button" };
        let size_key = super::metrics::helpers::control_size_key(size);
        font_size_source =
            TypographySource::Constant { label: format!("style.toml [{family}.metrics.{size_key}].font_size") };
        if base_size > 0.0 {
            style.line_height = style.size * (style.line_height / base_size);
            line_height_source = TypographySource::Derived { note: "font_size × label line-height ratio".into() };
        }
    }
    ControlTypographyTable {
        style,
        font_family: mode.typography.font.sans.family.clone().into(),
        font_family_source: if mode.catalog.get("font-sans").is_some() {
            TypographySource::CssVar { token: "font-sans".into() }
        } else {
            TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() }
        },
        font_size_source,
        font_weight_source: TypographySource::Scaffold { path: "LumaTypography.text.label.weight".into() },
        line_height_source,
    }
}
