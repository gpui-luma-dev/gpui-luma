//! Trigger-owned popover. Placement and popup lifecycle live in the private panel engine.

mod control;
mod model;
mod panel;

pub use control::PopoverButton;
pub use model::{
    PopoverButtonBuilder, PopoverButtonEvent, PopoverContent, PopoverDismissPolicy, PopoverPlacement,
    PopoverRenderModel, PopoverTrigger,
};
