use gpui::FontWeight;
use gpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyRole};
use gpui_luma::controls::pager::{PagerLook, PagerStyle};
use gpui_luma::theme::{ControlSize, InteractionState};

use crate::look::ShadcnLook;

pub fn pager_look(theme: &ShadcnLook, enabled: bool, style: PagerStyle) -> PagerLook {
    let tokens = theme.mode_tokens();
    let palette = &tokens.palette;
    let metrics = &tokens.metrics;
    let typography = theme.typography_scale(crate::ShadcnTextSize::Xs);
    let key = style.stylesheet_key();
    let baseline = gpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().pager.authored_geometry(key);
    let recipe = tokens.stylesheet().common.pager.authored_geometry(key).with_fallback(&baseline);

    let mut look = PagerLook {
        gap_icon_size: recipe.gap_icon_size.unwrap_or_default(),
        border_width: recipe.border_width.unwrap_or_default(),
        page_size_label_gap: recipe.page_size_label_gap.unwrap_or_default(),
        popup_offset_y: recipe.popup_offset_y.unwrap_or_default(),
        popup_gap: recipe.popup_gap.unwrap_or_default(),
        popup_padding: recipe.popup_padding.unwrap_or_default(),
        popup_item_padding_y: recipe.popup_item_padding_y.unwrap_or_default(),
        popup_item_radius_inset: recipe.popup_item_radius_inset.unwrap_or_default(),

        panel_background: if enabled {
            palette.panel_background
        } else {
            palette.disabled_background
        },
        border: palette.border_default,
        body_text: if enabled {
            palette.body_text
        } else {
            palette.disabled_foreground
        },
        muted_text: if enabled {
            palette.app_muted_foreground
        } else {
            palette.disabled_foreground
        },
        selected_background: palette.selected_background,
        selected_foreground: palette.selected_foreground,
        hover_background: palette.accent_background,
        shadow: vec![],
        typography,
        button_size: metrics.control_height(ControlSize::Sm),
        button_min_width: metrics.control_height(ControlSize::Sm),
        control_height: metrics.control_height(ControlSize::Sm),
        radius: metrics.radius(ControlSize::Sm),
        padding_x: metrics.padding_x(ControlSize::Sm),
        padding_y: metrics.padding_y(ControlSize::Sm),
        gap: metrics.gap(ControlSize::Sm),
        group_gap: metrics.gap(ControlSize::Sm),
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
    };
    let geometry = tokens.stylesheet().common.pager.resolve_geometry(
        key,
        gpui_luma::theme::stylesheet::PagerGeometry {
            button_size: look.button_size,
            button_min_width: look.button_min_width,
            control_height: look.control_height,
            padding_x: look.padding_x,
            padding_y: look.padding_y,
            gap: look.gap,
            group_gap: look.group_gap,
            font_size: look.typography.size,
            line_height: look.typography.line_height,
            radius: look.radius,
            disabled_opacity: look.disabled_opacity,
            ..Default::default()
        },
    );
    look.button_size = geometry.button_size.value_px;
    look.button_min_width = geometry.button_min_width.value_px;
    look.control_height = geometry.control_height.value_px;
    look.padding_x = geometry.padding_x.value_px;
    look.padding_y = geometry.padding_y.value_px;
    look.gap = geometry.gap.value_px;
    look.group_gap = geometry.group_gap.value_px;
    look.radius = geometry.radius.value_px;
    look.disabled_opacity = geometry.disabled_opacity.value_px;
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut look.typography,
        &geometry.font_size,
        &geometry.line_height,
    );

    look
}

pub fn pager_button_look(
    theme: &ShadcnLook,
    pager_look: &PagerLook,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyLook {
    let mut look = theme.resolve_outline_button(role, ControlSize::Sm, state);
    look.height = pager_look.button_size;
    look.radius = pager_look.radius;
    look.gap = pager_look.gap;
    look.typography.size = pager_look.typography.size;
    look.typography.line_height = pager_look.typography.line_height;
    look.typography.weight = if matches!(role, ButtonFamilyRole::Toggle { selected: true }) {
        FontWeight::SEMIBOLD
    } else {
        pager_look.typography.weight
    };

    if matches!(role, ButtonFamilyRole::Icon) {
        look.padding_x = 0.0;
        look.padding_y = 0.0;
    } else {
        look.padding_x = pager_look.padding_x;
        look.padding_y = 0.0;
    }

    look
}
