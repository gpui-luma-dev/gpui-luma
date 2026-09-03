use gpui::hsla;
use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::ShadcnButtonStyle;
use luma_look_shadcn::ShadcnTextFieldStyle;

use super::box_model::BoxModelLayerColors;

pub fn choice_variant_style(variant_id: &str) -> ShadcnButtonStyle {
    match variant_id {
        "secondary" => ShadcnButtonStyle::Secondary,
        "content-only" => ShadcnButtonStyle::ContentOnly,
        _ => ShadcnButtonStyle::Primary,
    }
}

pub fn button_variant_style(variant_id: &str) -> ShadcnButtonStyle {
    match variant_id {
        "secondary" => ShadcnButtonStyle::Secondary,
        "outline" => ShadcnButtonStyle::Outline,
        "ghost" => ShadcnButtonStyle::Ghost,
        "content-only" => ShadcnButtonStyle::ContentOnly,
        _ => ShadcnButtonStyle::Primary,
    }
}

pub fn choice_indicator_only(part_id: &str) -> bool {
    part_id == "indicator-only"
}

pub fn interaction_state(state_id: &str) -> InteractionState {
    match state_id {
        "hover" => InteractionState { hovered: true, ..InteractionState::default() },
        "focused" => InteractionState { focused: true, ..InteractionState::default() },
        "pressed" => InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        "disabled" => InteractionState { disabled: true, ..InteractionState::default() },
        _ => InteractionState::default(),
    }
}

pub fn control_size(size_id: &str) -> ControlSize {
    match size_id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
    }
}

pub fn neutral_box_model_colors(mode: ThemeMode) -> BoxModelLayerColors {
    match mode {
        ThemeMode::Dark => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.16, 1.0),
            border: hsla(0.0, 0.0, 0.24, 1.0),
            padding: hsla(0.0, 0.0, 0.32, 1.0),
            content: hsla(0.0, 0.0, 0.40, 1.0),
            stroke: hsla(0.0, 0.0, 0.88, 0.92),
            highlight: hsla(0.0, 0.0, 0.96, 1.0),
        },
        ThemeMode::Light => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.94, 1.0),
            border: hsla(0.0, 0.0, 0.86, 1.0),
            padding: hsla(0.0, 0.0, 0.78, 1.0),
            content: hsla(0.0, 0.0, 0.70, 1.0),
            stroke: hsla(0.0, 0.0, 0.18, 0.88),
            highlight: hsla(0.0, 0.0, 0.08, 1.0),
        },
    }
}

pub fn neutral_box_model_label_color(mode: ThemeMode) -> gpui::Hsla {
    match mode {
        ThemeMode::Dark => hsla(0.0, 0.0, 0.94, 0.96),
        ThemeMode::Light => hsla(0.0, 0.0, 0.14, 0.96),
    }
}

pub fn value_flag(value_id: &str, true_id: &str) -> bool {
    value_id == true_id
}

pub fn inspector_elevation_applies(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn choice_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    variant_id != "content-only" && inspector_elevation_applies(state_id)
}

pub fn button_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    variant_id != "content-only" && inspector_elevation_applies(state_id)
}

pub fn textfield_variant_style(variant_id: &str) -> ShadcnTextFieldStyle {
    match variant_id {
        "primary" => ShadcnTextFieldStyle::Primary,
        "surface" => ShadcnTextFieldStyle::Surface,
        _ => ShadcnTextFieldStyle::Outline,
    }
}

pub fn textfield_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn textfield_state(state_id: &str) -> luma::controls::textfield::TextFieldState {
    match state_id {
        "hover" => luma::controls::textfield::TextFieldState {
            hovered: true,
            ..luma::controls::textfield::TextFieldState::default()
        },
        "focus" => luma::controls::textfield::TextFieldState {
            focused: true,
            focus_visible: true,
            ..luma::controls::textfield::TextFieldState::default()
        },
        "invalid" => luma::controls::textfield::TextFieldState {
            invalid: true,
            ..luma::controls::textfield::TextFieldState::default()
        },
        _ => luma::controls::textfield::TextFieldState::default(),
    }
}

pub fn textarea_state(state_id: &str) -> luma::controls::textarea::TextAreaState {
    match state_id {
        "hover" => luma::controls::textarea::TextAreaState {
            hovered: true,
            ..luma::controls::textarea::TextAreaState::default()
        },
        "focus" => luma::controls::textarea::TextAreaState {
            focused: true,
            focus_visible: true,
            ..luma::controls::textarea::TextAreaState::default()
        },
        "invalid" => luma::controls::textarea::TextAreaState {
            invalid: true,
            ..luma::controls::textarea::TextAreaState::default()
        },
        _ => luma::controls::textarea::TextAreaState::default(),
    }
}

pub fn textfield_elevation_applies(variant_id: &str, state_id: &str) -> bool {
    !variant_id.is_empty() && textfield_enabled(state_id)
}

pub fn badge_variant(variant_id: &str) -> luma_look_shadcn::BadgeVariant {
    match variant_id {
        "secondary" => luma_look_shadcn::BadgeVariant::Secondary,
        "outline" => luma_look_shadcn::BadgeVariant::Outline,
        "ghost" => luma_look_shadcn::BadgeVariant::Ghost,
        _ => luma_look_shadcn::BadgeVariant::Default,
    }
}

pub fn progress_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn stepper_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn scrollbar_style(variant_id: &str) -> luma::controls::scrollbar::ScrollbarStyle {
    match variant_id {
        "soft" => luma::controls::scrollbar::ScrollbarStyle::Soft,
        _ => luma::controls::scrollbar::ScrollbarStyle::Ghost,
    }
}

pub fn pager_style(variant_id: &str) -> luma::controls::pager::PagerStyle {
    match variant_id {
        "minimal" => luma::controls::pager::PagerStyle::Minimal,
        "minimal-edge" => luma::controls::pager::PagerStyle::MinimalEdge,
        _ => luma::controls::pager::PagerStyle::Numeric,
    }
}

pub fn pager_shell_enabled(state_id: &str) -> bool {
    state_id != "disabled"
}

pub fn pager_button_role(variant_id: &str, value_id: &str) -> luma::controls::button_family::ButtonFamilyRole {
    match variant_id {
        "page" => {
            luma::controls::button_family::ButtonFamilyRole::Toggle { selected: value_flag(value_id, "selected") }
        }
        _ => luma::controls::button_family::ButtonFamilyRole::Icon,
    }
}

pub fn popup_menu_trigger_style(variant_id: &str) -> luma::controls::popup_menu::PopupMenuTriggerStyle {
    match variant_id {
        "primary" => luma::controls::popup_menu::PopupMenuTriggerStyle::Primary,
        "secondary" => luma::controls::popup_menu::PopupMenuTriggerStyle::Secondary,
        "ghost" => luma::controls::popup_menu::PopupMenuTriggerStyle::Ghost,
        _ => luma::controls::popup_menu::PopupMenuTriggerStyle::Outline,
    }
}

pub fn toolbar_variant(variant_id: &str) -> luma::controls::toolbar::ToolbarVariant {
    match variant_id {
        "ghost" => luma::controls::toolbar::ToolbarVariant::Ghost,
        _ => luma::controls::toolbar::ToolbarVariant::Outline,
    }
}

pub fn listbox_list_enabled(state_id: &str) -> bool {
    state_id == "enabled" || state_id == "focused"
}

pub fn listbox_list_focused(state_id: &str) -> bool {
    state_id == "focused"
}

pub fn listbox_row_state(state_id: &str) -> InteractionState {
    match state_id {
        "keyboard-active" => InteractionState { focused: true, ..InteractionState::default() },
        other => interaction_state(other),
    }
}

pub fn list_view_row_selected(value_id: &str) -> bool {
    value_id == "selected"
}

pub fn accordion_content_expanded(state_id: &str) -> bool {
    state_id == "expanded"
}

pub fn tabs_navigation_active(variant_id: &str) -> bool {
    variant_id == "active"
}

pub fn sidebar_item_selected(value_id: &str) -> bool {
    value_id == "selected"
}

pub fn resize_handle_size(size_id: &str) -> luma::controls::resizable_panels::ResizeHandleSize {
    match size_id {
        "sm" => luma::controls::resizable_panels::ResizeHandleSize::Sm,
        "lg" => luma::controls::resizable_panels::ResizeHandleSize::Lg,
        _ => luma::controls::resizable_panels::ResizeHandleSize::Md,
    }
}

#[cfg(test)]
mod tests {
    use super::{button_elevation_applies, textfield_elevation_applies};

    #[test]
    fn button_elevation_inspection_is_not_limited_to_outline() {
        assert!(button_elevation_applies("primary", "default"));
        assert!(button_elevation_applies("secondary", "default"));
        assert!(button_elevation_applies("outline", "default"));
        assert!(!button_elevation_applies("content-only", "default"));
    }

    #[test]
    fn textfield_elevation_inspection_is_not_limited_to_primary() {
        assert!(textfield_elevation_applies("outline", "default"));
        assert!(textfield_elevation_applies("primary", "default"));
        assert!(textfield_elevation_applies("surface", "default"));
        assert!(!textfield_elevation_applies("outline", "disabled"));
    }
}
