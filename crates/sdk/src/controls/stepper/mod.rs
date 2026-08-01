mod control;
mod model;
mod template;
mod theme;

pub use model::{StepState, StepperBuilder, StepperItem, StepperLabelPlacement, StepperModel, StepperRenderModel};
pub use template::{StepperTemplate, ThemedStepperTemplate, default_stepper_template};
pub use theme::{DefaultStepperTheme, StepperLook, StepperTheme, default_stepper_theme};

use gpui::{Entity, SharedString};

use self::control::StepperControl;

pub type Stepper = Entity<StepperControl>;

pub fn stepper(id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
    StepperBuilder::new(id, step_count)
}
