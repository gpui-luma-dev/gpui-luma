//! Button family: LMTP button, icon preset, theme recipe, and popup-menu split preset.
//!
//! - [`Button`] / [`IconButton`] — spawnable controls
//! - [`family`] — shared look/metrics recipe (`button_family` re-export)
//! - [`split_button`] — `PopupMenu` type aliases, not a separate LMTP control
//! - [`CommandCore`] — presentation-agnostic activation/interaction core used by `Button`

mod control;
mod core;
mod model;
mod template;

pub mod family;
pub mod icon_button;
pub mod split_button;

pub use control::{Button, ButtonEvent};
pub use core::{CommandCore, CommandEvent};
pub use crate::infra::template::{ControlTemplate, Modifier, TemplateWithModifiers};
pub use model::{
    button_content_context, ButtonBuilder, ButtonContentContext, ButtonLookSource, ButtonModel, ButtonRenderModel,
    ControlIcon, ControlPresenter, HasPresenter,
};
pub use template::{ButtonTemplate, ButtonTemplateModifier, DefaultButtonTemplate, default_button_template};
pub use family::ButtonSize;
pub use icon_button::IconButton;
