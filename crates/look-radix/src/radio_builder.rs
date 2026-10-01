//! Look-owned radio builder. Spawn synthesizes the SDK [`gpui_luma::controls::radio_button::RadioButton`].

use gpui::{App, Context, SharedString};
use gpui_luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::radio_button::{RadioButtonBuilder, RadioButtonData};

use crate::button::Paint;
use crate::look::{Look, resolve_look};
use crate::radio::{RadioSize, RadioVariant, radio_template_for};
use crate::tone::Tone;

/// Builder in the guise of a radio: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Radio {
    id: SharedString,
    look: Option<Look>,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    selected: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    animated: bool,
    role: Option<ButtonFamilyRole>,
    content: Option<ControlPresenter<ButtonContentContext<RadioButtonData>>>,
}

impl Radio {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            variant: RadioVariant::default(),
            paint: Paint::accent(),
            size: RadioSize::default(),
            selected: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            animated: true,
            role: None,
            content: None,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: RadioVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(RadioVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(RadioVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(RadioVariant::Soft)
    }

    pub fn accent(mut self) -> Self {
        self.paint.tone = Tone::Accent;
        self
    }

    pub fn gray(mut self) -> Self {
        self.paint.tone = Tone::Gray;
        self
    }

    pub fn high_contrast(mut self, high_contrast: bool) -> Self {
        self.paint.high_contrast = high_contrast;
        self
    }

    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }

    /// SDK payload seam (`selected`). Radio is not generic over `D`.
    pub fn with_data(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Same field as [`Self::with_data`].
    pub fn selected(self, selected: bool) -> Self {
        self.with_data(selected)
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.tab_stop = tab_stop;
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.without_elevation = true;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> gpui_luma::controls::radio_button::RadioButton {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> RadioButtonBuilder {
        let template = radio_template_for(&look, self.variant, self.paint, self.size);
        let mut builder = gpui_luma::controls::radio_button::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.selected)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .animated(self.animated);
        if let Some(role) = self.role {
            builder = builder.role(role);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<RadioButtonData>> for Radio {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<RadioButtonData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::controls::button::HasPresenter;
    use gpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Radio::new("s").size(RadioSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Radio::new("s").size(RadioSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Radio::new("s").size(RadioSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_radix_axes() {
        let radio = Radio::new("opt").soft().gray().size(RadioSize::One).with_data(true);
        assert!(matches!(radio.variant, RadioVariant::Soft));
        assert_eq!(radio.paint.tone, Tone::Gray);
        assert_eq!(radio.size, RadioSize::One);
        assert!(radio.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Radio::new("opt").selected(true).selected);
        assert!(Radio::new("opt").with_data(true).selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Radio::new("ok")
            .look(&look)
            .classic()
            .high_contrast(true)
            .with_data(true)
            .label("A")
            .into_sdk_builder(look);
    }
}
