mod template;
mod theme;

pub use template::{ThemedSwitchTemplate, default_template as default_switch_template};
pub use theme::{DefaultSwitchTheme, SwitchLook, SwitchPalette, SwitchScale, SwitchTheme, default_switch_theme};

pub use crate::theme::InteractionState as SwitchState;

use std::sync::Arc;

use gpui::{App, Context, Entity, IntoElement, SharedString};

use crate::controls::button_family::ButtonSize;
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

    pub fn without_elevation(self) -> Self {
        Self(self.0.without_elevation())
    }

    pub fn without_adorners(self) -> Self {
        Self(self.0.without_adorners())
    }

    pub fn compact(self) -> Self {
        Self(self.0.compact())
    }

    /// Adds extra width to the switch track for interior content such as ON/OFF labels.
    pub fn track_width_extra(mut self, extra_width: f32) -> Self {
        self.0.model.switch_track_width_extra = extra_width.max(0.0);
        self
    }

    /// Renders content inside the switch track behind the thumb.
    pub fn track_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.0.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    /// Renders content inside the moving switch thumb.
    pub fn thumb_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.0.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    pub fn size(self, size: ButtonSize) -> Self {
        Self(self.0.size(size))
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

impl Button<bool> {
    pub fn set_switch_track_width_extra(&mut self, extra_width: f32, cx: &mut Context<Self>) {
        self.model.switch_track_width_extra = extra_width.max(0.0);
        cx.notify();
    }

    pub fn set_switch_track_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_track_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }

    pub fn set_switch_thumb_content<F, E>(&mut self, builder: F, cx: &mut Context<Self>)
    where
        F: Fn(&ButtonRenderModel<bool>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.switch_thumb_content = Some(Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        cx.notify();
    }
}

pub fn new(id: impl Into<SharedString>) -> SwitchBuilder {
    SwitchBuilder(ButtonBuilder::new(id).typed(false).template(default_switch_template()))
}

#[cfg(test)]
mod tests {
    use gpui::div;

    use super::*;

    #[test]
    fn switch_builder_sets_slot_content_and_track_width_extra() {
        let builder =
            new("slot-switch").track_width_extra(12.0).track_content(|_, _| div()).thumb_content(|_, _| div());

        assert_eq!(builder.0.model.switch_track_width_extra, 12.0);
        assert!(builder.0.model.switch_track_content.is_some());
        assert!(builder.0.model.switch_thumb_content.is_some());
    }
}
