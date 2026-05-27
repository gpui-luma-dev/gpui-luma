//! Radio button property mappings (tweakcn / shadcn):
//!
//! | State     | Ring token | Dot token              |
//! |-----------|------------|------------------------|
//! | Unselected| outline    | `foreground`           |
//! | Selected  | `{style}`  | `{style}-foreground`   |
//! | Disabled  | `muted`    | `muted-foreground`     |

use crate::controls::radio_button::RadioButtonAppearance;
use crate::theme::{ControlSize, InteractionLayer, InteractionState};

use super::focus::focus_adorner;
use super::resolve::{
    resolve_action_foreground, resolve_action_layer, resolve_color, resolve_label_color, resolve_outline_layer,
};
use super::RadixButtonStyle;
use super::catalog::CssTokenMap;
use super::mode::RadixModeTokens;

pub(crate) fn radio_button_appearance(
    mode: &RadixModeTokens,
    style: RadixButtonStyle,
    selected: bool,
    state: InteractionState,
) -> RadioButtonAppearance {
    radio_button_appearance_from_catalog(&mode.catalog, &mode.metrics, &mode.typography, style, selected, state)
        .unwrap_or_else(|err| panic!("radio properties: {err}"))
}

pub(crate) fn radio_button_appearance_from_catalog(
    catalog: &CssTokenMap,
    metrics: &crate::theme::MetricTokens,
    typography: &crate::theme::LumaTypography,
    style: RadixButtonStyle,
    selected: bool,
    state: InteractionState,
) -> anyhow::Result<RadioButtonAppearance> {
    let size = ControlSize::Md;
    let layer = state.layer();

    let indicator_background = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted")?,
        _ => resolve_outline_layer(catalog, layer)?,
    };

    let selected_color = match layer {
        InteractionLayer::Disabled => resolve_color(catalog, "muted-foreground")?,
        _ => resolve_action_layer(catalog, style, layer)?,
    };

    let indicator_border = if selected && !state.disabled {
        selected_color
    } else {
        resolve_color(catalog, "border")?
    };

    Ok(RadioButtonAppearance {
        control_background: None,
        control_border: None,
        indicator_background,
        indicator_border,
        dot_color: dot_color(catalog, style, selected, state.disabled)?,
        label_color: resolve_label_color(catalog, state.disabled)?,
        adorner: focus_adorner(catalog, metrics, state.focused)?,
        label_typography: typography.text.label,
        label_font_family: typography.font.sans.family.clone().into(),
        control_radius: metrics.radius(size),
        control_padding_x: 0.0,
        control_padding_y: 0.0,
        indicator_size: metrics.control_height(size) * 0.5,
        dot_size: metrics.control_height(size) * 0.24,
        gap: metrics.gap(size),
        height: metrics.control_height(size),
    })
}

fn dot_color(
    catalog: &CssTokenMap,
    style: RadixButtonStyle,
    selected: bool,
    disabled: bool,
) -> anyhow::Result<gpui::Hsla> {
    if disabled {
        return resolve_color(catalog, "muted-foreground");
    }
    if selected {
        return resolve_action_foreground(catalog, style);
    }
    resolve_color(catalog, "foreground")
}
