mod control;
mod model;
mod template;

// Canonical semantic API
pub use control::{Toggle, ToggleEvent};
pub use model::{ToggleBuilder, ToggleModel, ToggleRenderModel};
pub use template::{ThemedToggleTemplate, ToggleTemplate, default_toggle_template};
pub use crate::controls::button_family::{
    ButtonInteractionState as ToggleState, ButtonKind as ToggleKind, ButtonSize as ToggleSize,
};

// Backward-compatible API aliases
pub use control::{ToggleButton, ToggleButtonEvent};
pub use model::{ToggleButtonBuilder, ToggleButtonModel, ToggleButtonRenderModel};
pub use template::{ThemedToggleButtonTemplate, ToggleButtonTemplate, default_toggle_button_template};
pub use crate::controls::button_family::{
    ButtonInteractionState as ToggleButtonState, ButtonKind as ToggleButtonKind, ButtonSize as ToggleButtonSize,
};
