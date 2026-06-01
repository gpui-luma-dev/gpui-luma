mod template;
mod theme;

pub use template::{ThemedCheckboxTemplate, default_template as default_checkbox_template};

pub use theme::{
    CheckboxAppearance, CheckboxPalette, CheckboxScale, CheckboxTheme, DefaultCheckboxTheme, default_checkbox_theme,
};

pub use crate::theme::InteractionState as CheckboxState;

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};

pub type Checkbox = Entity<Button<bool>>;

/// Builder for [`Checkbox`] controls. Distinct from [`ButtonBuilder<bool>`] so Radix style
/// helpers apply the checkbox template rather than the switch template.
pub struct CheckboxBuilder(ButtonBuilder<bool>);

impl CheckboxBuilder {
    pub fn with_data(self, data: bool) -> Self {
        Self(self.0.with_data(data))
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self(self.0.tab_stop(tab_stop))
    }

    pub fn template(self, template: Arc<dyn ButtonTemplate<bool>>) -> Self {
        Self(self.0.template(template))
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Checkbox {
        self.0.spawn(cx)
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for CheckboxBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> CheckboxBuilder {
    CheckboxBuilder(ButtonBuilder::new(id).typed(false).template(default_checkbox_template()))
}
