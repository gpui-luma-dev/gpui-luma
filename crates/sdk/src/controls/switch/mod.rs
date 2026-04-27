mod control;
mod model;
mod template;

pub use control::{Switch, SwitchEvent};
pub use model::{SwitchBuilder, SwitchModel, SwitchRenderModel};
pub use template::{SwitchTemplate, ThemedSwitchTemplate, default_template as default_switch_template};

pub use crate::theme::InteractionState as SwitchState;
