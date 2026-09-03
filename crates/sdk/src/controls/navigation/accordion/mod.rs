mod control;
mod model;
mod template;
mod theme;

pub use control::{AccordionControl, AccordionEvent};
pub use model::{
    AccordionBuilder, AccordionContent, AccordionItem, AccordionItemRenderModel, AccordionModel, AccordionRenderModel,
    AccordionSelectionMode, AccordionTrigger,
};
pub use template::{
    AccordionTemplate, AccordionTemplateHandlers, AccordionTemplateModifier, ThemedAccordionTemplate,
    default_accordion_template,
};
pub use theme::{
    AccordionContentPalette, AccordionPalette, AccordionScale, AccordionTheme, DefaultAccordionTheme,
    default_accordion_theme,
};

pub use crate::infra::state::{CompositeItemState as AccordionItemState, ControlFocusState};
use gpui::{Entity, SharedString};

/// Entity handle for an accordion control. Use this type directly — do not wrap in [`Entity`] again.
pub type Accordion = Entity<AccordionControl>;

/// Creates a new builder for an Accordion.
pub fn new(id: impl Into<SharedString>) -> AccordionBuilder {
    AccordionBuilder::new(id)
}
