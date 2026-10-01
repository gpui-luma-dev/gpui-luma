//! Look-owned slider builder. Spawn synthesizes the SDK [`gpui_luma::controls::slider::Slider`].

use gpui::{App, Context, Div, SharedString, Stateful};
use gpui_luma::controls::slider::{
    SliderBuilder, SliderOrientation, SliderRenderModel, SliderTemplateModifier, SliderThumbPolicy, TrackPresentation,
};
use gpui_luma::infra::value::ControlRange;

use crate::look::{Look, resolve_look};
use crate::slider::{SliderSize, SliderVariant, slider_template};

/// Builder in the guise of a slider: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Slider {
    id: SharedString,
    look: Option<Look>,
    variant: SliderVariant,
    size: SliderSize,
    range: ControlRange,
    step: f64,
    value: Option<f64>,
    enabled: bool,
    animated: bool,
    wrapping: bool,
    reversed: bool,
    orientation: Option<SliderOrientation>,
    presentation: Option<TrackPresentation>,
    thumb_policy: Option<SliderThumbPolicy>,
    modifiers: Vec<SliderTemplateModifier>,
}

impl Slider {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            variant: SliderVariant::default(),
            size: SliderSize::default(),
            range: ControlRange::default(),
            step: 1.0,
            value: None,
            enabled: true,
            animated: true,
            wrapping: false,
            reversed: false,
            orientation: None,
            presentation: None,
            thumb_policy: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: SliderVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(SliderVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(SliderVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(SliderVariant::Soft)
    }

    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.range = range.into();
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.step = step.into();
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn wrapping(mut self, wrapping: bool) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SliderOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SliderOrientation::Vertical)
    }

    pub fn fill(mut self) -> Self {
        self.presentation = Some(TrackPresentation::Fill);
        self
    }

    pub fn domain(mut self) -> Self {
        self.presentation = Some(TrackPresentation::Domain);
        self
    }

    pub fn thumb_policy(mut self, policy: SliderThumbPolicy) -> Self {
        self.thumb_policy = Some(policy);
        self
    }

    pub fn multi_stop(self) -> Self {
        self.thumb_policy(SliderThumbPolicy::multi_stop()).domain()
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &SliderRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> gpui_luma::controls::slider::Slider {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> SliderBuilder {
        let template = slider_template(&look, self.variant);
        let mut builder = gpui_luma::controls::slider::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .range(self.range)
            .step(self.step)
            .enabled(self.enabled)
            .animated(self.animated)
            .wrapping(self.wrapping)
            .reversed(self.reversed);
        if let Some(orientation) = self.orientation {
            builder = builder.orientation(orientation);
        }
        match self.presentation {
            Some(TrackPresentation::Fill) => builder = builder.fill(),
            Some(TrackPresentation::Domain) => builder = builder.domain(),
            None => {}
        }
        if let Some(policy) = self.thumb_policy {
            builder = builder.thumb_policy(policy);
        }
        if let Some(value) = self.value {
            builder = builder.value(value);
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Slider::new("s").size(SliderSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Slider::new("s").size(SliderSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Slider::new("s").size(SliderSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn value_keeps_radix_axes() {
        let slider = Slider::new("gain").soft().size(SliderSize::Three).range(0.0f32..10.0).step(0.5).value(2.0);
        assert!(matches!(slider.variant, SliderVariant::Soft));
        assert_eq!(slider.size, SliderSize::Three);
        assert_eq!(slider.range, ControlRange::new(0.0, 10.0));
        assert_eq!(slider.step, 0.5);
        assert_eq!(slider.value, Some(2.0));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Slider::new("ok")
            .look(&look)
            .classic()
            .range(0..100)
            .value(40)
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
