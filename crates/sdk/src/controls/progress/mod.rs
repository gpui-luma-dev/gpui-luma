mod control;
mod model;
mod template;
mod theme;

pub use model::{ProgressBuilder, ProgressModel, ProgressRenderModel};
pub use template::{ProgressTemplate, ThemedProgressTemplate, default_progress_template};
pub use theme::{DefaultProgressTheme, ProgressLook, ProgressTheme, default_progress_theme};

use gpui::{Entity, SharedString};

use self::control::ProgressControl;

pub type Progress = Entity<ProgressControl>;

pub fn new(id: impl Into<SharedString>) -> ProgressBuilder {
    ProgressBuilder::new(id)
}
