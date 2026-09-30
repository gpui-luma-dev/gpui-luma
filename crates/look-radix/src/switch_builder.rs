//! Look-owned switch builder. Spawn synthesizes the SDK [`luma::controls::switch::Switch`].

use gpui::{App, Context, SharedString};
use luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use luma::controls::switch::{SwitchBuilder, SwitchData, SwitchOrientation};

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::{Look, resolve_look};
use crate::switch::{SwitchSize, SwitchVariant, switch_template_for};
use crate::tone::Tone;

/// Builder in the guise of a switch: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Switch {
    id: SharedString,
    look: Option<Look>,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
    checked: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    animated: bool,
    orientation: Option<SwitchOrientation>,
    content: Option<ControlPresenter<ButtonContentContext<SwitchData>>>,
}

impl Switch {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            variant: SwitchVariant::default(),
            paint: Paint::accent(),
            size: SwitchSize::default(),
            radius: Radius::default(),
            checked: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            animated: true,
            orientation: None,
            content: None,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: SwitchVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(SwitchVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(SwitchVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(SwitchVariant::Soft)
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

    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }

    /// SDK payload seam (`checked`). Switch is not generic over `D`.
    pub fn with_data(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Same field as [`Self::with_data`].
    pub fn checked(self, checked: bool) -> Self {
        self.with_data(checked)
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

    pub fn orientation(mut self, orientation: SwitchOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SwitchOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SwitchOrientation::Vertical)
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::switch::Switch {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> SwitchBuilder {
        let template = switch_template_for(&look, self.variant, self.paint, self.size, self.radius);
        let mut builder = luma::controls::switch::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.checked)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .animated(self.animated);
        if let Some(orientation) = self.orientation {
            builder = builder.orientation(orientation);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        } else {
            builder = builder.without_label();
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<SwitchData>> for Switch {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<SwitchData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::controls::button::HasPresenter;
    use luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Switch::new("s").size(SwitchSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Switch::new("s").size(SwitchSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Switch::new("s").size(SwitchSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_radix_axes() {
        let switch = Switch::new("notify").soft().gray().size(SwitchSize::Three).with_data(true);
        assert!(matches!(switch.variant, SwitchVariant::Soft));
        assert_eq!(switch.paint.tone, Tone::Gray);
        assert_eq!(switch.size, SwitchSize::Three);
        assert!(switch.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Switch::new("notify").checked(true).checked);
        assert!(Switch::new("notify").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Switch::new("ok")
            .look(&look)
            .classic()
            .high_contrast(true)
            .with_data(true)
            .label("On")
            .into_sdk_builder(look);
    }
}
