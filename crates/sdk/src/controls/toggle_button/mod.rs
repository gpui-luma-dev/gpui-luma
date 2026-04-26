//! Legacy compatibility shim for the former `toggle_button` module.
//!
//! Prefer importing from `crate::controls::toggle` for the canonical semantic API.

pub use crate::controls::toggle::{
    default_toggle_button_template, default_toggle_template, ThemedToggleButtonTemplate, ThemedToggleTemplate, Toggle,
    ToggleBuilder, ToggleButton, ToggleButtonBuilder, ToggleButtonEvent, ToggleButtonKind, ToggleButtonModel,
    ToggleButtonRenderModel, ToggleButtonSize, ToggleButtonState, ToggleButtonTemplate, ToggleEvent, ToggleKind,
    ToggleModel, ToggleRenderModel, ToggleSize, ToggleState, ToggleTemplate,
};
