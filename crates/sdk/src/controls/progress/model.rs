use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::template::template_with_modifier;
use super::{ProgressTemplate, default_progress_template};
use super::control::ProgressControl;
use crate::controls::value::{ControlRange, value_from_input};

#[derive(Clone)]
pub struct ProgressModel {
    pub(crate) id: SharedString,
    pub(crate) range: ControlRange,
    pub(crate) value: f32,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ProgressTemplate>,
}

pub struct ProgressRenderModel<'a> {
    pub id: &'a SharedString,
    pub range: ControlRange,
    pub value: f32,
    pub percentage: f32,
    pub enabled: bool,
}

pub struct ProgressBuilder {
    pub(crate) model: ProgressModel,
}

impl ProgressBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ProgressModel {
                id: id.into(),
                range: ControlRange::default(),
                value: 0.0,
                enabled: true,
                template: default_progress_template(),
            },
        }
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.value = self.model.range.clamp(self.model.value);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value = self.model.range.clamp(value_from_input(value));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ProgressTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ProgressRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ProgressControl> {
        cx.new(|cx| ProgressControl::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_progress_template();
        let builder = ProgressBuilder::new("progress-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
