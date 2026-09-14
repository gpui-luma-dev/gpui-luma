//! Look-owned radio builder. Spawn synthesizes the SDK [`luma::controls::radio_button::RadioButton`].

use gpui::{App, Context, SharedString};
use luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use luma::controls::button_family::ButtonFamilyRole;
use luma::controls::radio_button::{RadioButtonBuilder, RadioButtonData};
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a radio: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Radio {
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
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
            style: ShadcnButtonStyle::Primary,
            size: ShadcnSize::Md,
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
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn style(mut self, style: ShadcnButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn primary(self) -> Self {
        self.style(ShadcnButtonStyle::Primary)
    }

    pub fn secondary(self) -> Self {
        self.style(ShadcnButtonStyle::Secondary)
    }

    pub fn outline(self) -> Self {
        self.style(ShadcnButtonStyle::Outline)
    }

    pub fn ghost(self) -> Self {
        self.style(ShadcnButtonStyle::Ghost)
    }

    pub fn content_only(self) -> Self {
        self.style(ShadcnButtonStyle::ContentOnly).without_elevation()
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::radio_button::RadioButton {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> RadioButtonBuilder {
        let template = look.radio_button_template(self.style);
        let mut builder = luma::controls::radio_button::new(self.id)
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
    use luma::controls::button::HasPresenter;

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let radio = Radio::new("opt").outline().size(ShadcnSize::Sm).with_data(true);
        assert!(matches!(radio.style, ShadcnButtonStyle::Outline));
        assert_eq!(radio.size, ShadcnSize::Sm);
        assert!(radio.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Radio::new("opt").selected(true).selected);
        assert!(Radio::new("opt").with_data(true).selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Radio::new("ok").look(&look).primary().with_data(true).label("A").into_sdk_builder(look);
    }
}
