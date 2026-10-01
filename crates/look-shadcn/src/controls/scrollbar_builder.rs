//! Look-owned scrollbar builder. Spawn synthesizes the SDK [`gpui_luma::controls::scrollbar::Scrollbar`].

use gpui::{Context, Entity, SharedString};
use gpui_luma::controls::scrollbar::{ScrollbarBuilder, ScrollbarOrientation, ScrollbarStyle};
use gpui_luma::infra::value::ControlRange;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a scrollbar: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Scrollbar {
    look: Option<ShadcnLook>,
    builder: ScrollbarBuilder,
    custom_template: bool,
    size: ShadcnSize,
}

impl Scrollbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: gpui_luma::controls::scrollbar::Scrollbar::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn orientation(mut self, orientation: ScrollbarOrientation) -> Self {
        self.builder = self.builder.orientation(orientation);
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.builder = self.builder.vertical();
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.builder = self.builder.range(range);
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.builder = self.builder.step(step);
        self
    }

    pub fn page_step(mut self, page_step: impl Into<f64>) -> Self {
        self.builder = self.builder.page_step(page_step);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.builder = self.builder.value(value);
        self
    }

    pub fn thumb_fraction(mut self, thumb_fraction: impl Into<f64>) -> Self {
        self.builder = self.builder.thumb_fraction(thumb_fraction);
        self
    }

    pub fn length(mut self, length: impl Into<f64>) -> Self {
        self.builder = self.builder.length(length);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn style(mut self, style: ScrollbarStyle) -> Self {
        self.builder = self.builder.style(style);
        self
    }

    pub fn ghost(mut self) -> Self {
        self.builder = self.builder.ghost();
        self
    }

    pub fn soft(mut self) -> Self {
        self.builder = self.builder.soft();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn template(mut self, template: std::sync::Arc<dyn gpui_luma::controls::scrollbar::ScrollbarTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<gpui_luma::controls::scrollbar::Scrollbar> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ScrollbarBuilder {
        let builder = self.builder.size(self.size.control_size());
        if self.custom_template {
            builder
        } else {
            builder.template(look.scrollbar_template())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Scrollbar::new("ok").look(&look).vertical().range(0..100).into_sdk_builder(look);
    }
}
