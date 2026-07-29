use super::inspector::specs::{CHOICE_SIZES, DEFAULT_INTERACTION_STATES};
use super::inspector::ControlInspectorSpec;
use super::textfield_menu_inspector_adapter::TextFieldMenuInspectorAdapter;

pub static COMBOBOX_INSPECTOR_SPEC: ControlInspectorSpec = ControlInspectorSpec {
    control_label: "ComboBox",
    id_prefix: "combobox-theme-inspector",
    parts: &[],
    variants: &[],
    states: &DEFAULT_INTERACTION_STATES,
    sizes: &CHOICE_SIZES,
    value_modes: &[],
    default_part_id: "",
    default_variant_id: "",
    default_size_id: "md",
    default_value_id: "",
};

pub fn combobox_inspector_adapter() -> super::inspector::SharedInspectorResolver {
    TextFieldMenuInspectorAdapter::shared("combobox-theme-inspector-box-model")
}
