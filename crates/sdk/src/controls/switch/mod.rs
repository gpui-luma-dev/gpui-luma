mod template;
mod theme;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};
pub use theme::{DefaultSwitchTheme, SwitchAppearance, SwitchTheme, default_switch_theme};

pub use crate::theme::InteractionState as SwitchState;

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};

use crate::controls::command::button::{Button, ButtonBuilder, ButtonRenderModel, ButtonTemplate};
use crate::controls::presenter::{ControlPresenter, HasPresenter};

pub type Switch = Entity<Button<bool>>;

/// Builder for [`Switch`] controls. Distinct from [`ButtonBuilder<bool>`] so Radix style
/// helpers apply the switch template rather than the checkbox template.
pub struct SwitchBuilder(ButtonBuilder<bool>);

impl SwitchBuilder {
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Switch {
        self.0.spawn(cx)
    }
}

impl HasPresenter<ButtonRenderModel<bool>> for SwitchBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<bool>>) {
        self.0.set_presenter(content);
    }
}

pub fn new(id: impl Into<SharedString>) -> SwitchBuilder {
    SwitchBuilder(ButtonBuilder::new(id).typed(false).template(default_switch_template()))
}
