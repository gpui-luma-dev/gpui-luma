//! ComboBox now supports layered template customization:
//! - control template (`ComboBoxTemplate`)
//! - panel template (`ComboBoxPanelTemplate`)
//! - item template (`ComboBoxItemTemplate`)
//!
//! For new integrations, prefer `panel_template(...)` + `with_item_template(...)`
//! over the transitional `items_template(...)` hook.

mod behavior;
mod control;
mod item_template;
mod items_template;
mod model;
mod panel_template;
mod template;
mod text_selection;

pub use behavior::SelectionItem;
pub use control::{ComboBox, ComboBoxControl, ComboBoxEvent};
pub use model::{ComboBoxBuilder, ComboBoxModel, TypingPolicy, new};
pub use item_template::{
    ComboBoxItemRenderModel, ComboBoxItemTemplate, ComboBoxItemTemplateModifier, make_combobox_item_template,
};
pub use items_template::{
    ComboBoxItemsTemplateModifier, DefaultComboBoxItemsTemplate, combobox_items_template_with_modifier,
};
pub use panel_template::{
    ComboBoxPanelRenderModel, ComboBoxPanelTemplate, ComboBoxPanelTemplateModifier, default_combobox_panel_template,
};
pub use template::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, ComboBoxRenderModel,
    ComboBoxTemplate, ComboBoxTemplateHandlers, ComboBoxTemplateModifier, default_combobox_items_template,
    default_combobox_template,
};

mod callback;
pub use callback::ComboBoxEventExt;

pub use model::ComboBoxMotion;

crate::theme::stylesheet::geometry::geometry_family!(
    ComboBoxStylesheet,
    ComboBoxGeometryStylesheet,
    ComboBoxGeometry,
    ResolvedComboBoxGeometry,
    "combobox",
    [affordance_inset, affordance_width, affordance_icon_size, affordance_gap, trigger_extra_gap]
);

impl ComboBoxStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ComboBoxGeometryStylesheet {
                affordance_inset: Some(10.0),
                affordance_width: Some(18.0),
                affordance_icon_size: Some(12.0),
                affordance_gap: Some(2.0),
                trigger_extra_gap: Some(8.0),
            },
            ..Default::default()
        }
    }
}
