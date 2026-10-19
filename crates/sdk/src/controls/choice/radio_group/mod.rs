//! Radio group — LMTP specialization over button-backed options.
//!
//! Files: `model` / `control` / `template`. Look-level colors come from the
//! radio-button / button family themes (no separate `theme.rs` here).

mod control;
mod model;
mod template;

pub use control::RadioGroupControl;
pub use model::{
    RadioGroup, RadioGroupBuilder, RadioGroupEvent, RadioGroupItem, RadioGroupItemLike, RadioGroupRenderModel,
    RadioGroupTemplate, RadioGroupTemplateHandlers, SelectionMode, horizontal, new,
};
pub use template::{
    RadioGroupLayout, default_horizontal_group_container_template, default_radio_group_container_template,
    horizontal_group_container_template, radio_group_button_item_element_template, radio_group_buttons_template,
    radio_group_container_template, render_radio_button_option, render_radio_button_rows,
};

mod callback;
pub use callback::RadioGroupEventExt;

crate::theme::stylesheet::geometry::geometry_family!(
    RadioGroupStylesheet,
    RadioGroupGeometryStylesheet,
    RadioGroupGeometry,
    ResolvedRadioGroupGeometry,
    "radio_group",
    [gap, item_gap],
    unit[disabled_opacity]
);

impl RadioGroupStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: RadioGroupGeometryStylesheet {
                item_gap: Some(8.0),
                disabled_opacity: Some(0.56),
                ..Default::default()
            },
            sizes: [("vertical", 8.0), ("horizontal", 12.0)]
                .into_iter()
                .map(|(key, gap)| (key.into(), RadioGroupGeometryStylesheet { gap: Some(gap), ..Default::default() }))
                .collect(),
        }
    }
}
