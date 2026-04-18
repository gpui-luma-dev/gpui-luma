use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::{ProgressBuilder, ProgressRenderModel};
use crate::controls::progress::model::ProgressModel;
use crate::controls::value::{ControlRange, value_from_input};

pub struct Progress {
    model: ProgressModel,
}

impl Progress {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ProgressBuilder {
        ProgressBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: ProgressBuilder, _cx: &mut Context<Self>) -> Self {
        Self { model: builder.model }
    }

    pub fn value(&self) -> f32 {
        self.model.value
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        self.model.value = self.model.range.clamp(value_from_input(value));
        cx.notify();
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.value = self.model.range.clamp(self.model.value);
        cx.notify();
    }

    fn render_model(&self) -> ProgressRenderModel<'_> {
        ProgressRenderModel {
            id: &self.model.id,
            range: self.model.range,
            value: self.model.value,
            percentage: self.model.range.percentage(self.model.value),
        }
    }
}

impl Render for Progress {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let model = self.render_model();

        div().child(self.model.template.render(&model, window, cx)).into_any_element()
    }
}
