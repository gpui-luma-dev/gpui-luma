mod control;
mod model;
mod template;

pub use control::ProgressControl;
pub use model::{ProgressBuilder, ProgressModel, ProgressRenderModel};
pub use template::{ProgressTemplate, ThemedProgressTemplate, default_progress_template};

use gpui::{Entity, SharedString};

pub type Progress = Entity<ProgressControl>;

pub fn new(id: impl Into<SharedString>) -> ProgressBuilder {
    ProgressBuilder::new(id)
}
