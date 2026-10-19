#![allow(dead_code)]

mod control;
mod model;

pub(super) use control::{PopoverPanel, PopoverPanelEvent};
pub(super) use model::{PopoverPanelDismissPolicy, PopoverPanelPlacement};

pub use model::PopoverMotion;

crate::theme::stylesheet::geometry::geometry_family!(
    PopoverStylesheet,
    PopoverGeometryStylesheet,
    PopoverGeometry,
    ResolvedPopoverGeometry,
    "popover",
    [offset_y, window_margin]
);

impl PopoverStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: PopoverGeometryStylesheet { window_margin: Some(8.0), ..Default::default() },
            sizes: [("button", 0.0), ("panel", 4.0)]
                .into_iter()
                .map(|(key, offset)| {
                    (key.into(), PopoverGeometryStylesheet { offset_y: Some(offset), ..Default::default() })
                })
                .collect(),
        }
    }
}
