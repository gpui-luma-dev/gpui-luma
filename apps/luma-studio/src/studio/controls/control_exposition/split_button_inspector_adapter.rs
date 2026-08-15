use std::sync::Arc;

use super::inspector::SharedInspectorResolver;
use super::inspector::ControlInspectorSpec;
use super::popup_menu_inspector_adapter::PopupMenuInspectorAdapter;
use super::inspector::specs::{CHOICE_SIZES, SPLIT_BUTTON_PARTS, POPUP_MENU_STATES};

pub static SPLIT_BUTTON_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "Split Button",
    id_prefix: "split-button-theme-inspector",
    parts: &SPLIT_BUTTON_PARTS,
    variants: &[],
    states: &POPUP_MENU_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "trigger",
    default_variant_id: "outline",
    default_size_id: "md",
    default_value_id: "",
};

pub fn split_button_inspector_resolver() -> SharedInspectorResolver {
    Arc::new(PopupMenuInspectorAdapter)
}
