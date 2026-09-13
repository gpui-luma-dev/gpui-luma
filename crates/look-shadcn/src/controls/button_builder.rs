//! Look-owned button builder. Spawn synthesizes the SDK [`luma::controls::button::Button`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use luma::controls::button::{
    ButtonBuilder, ButtonRenderModel, ButtonTemplateModifier, ControlIcon, ControlPresenter, DefaultButtonTemplate,
    HasPresenter,
};
use luma::controls::button_family::ButtonFamilyRole;
use super::button::{ButtonRadiusPreset, ShadcnButtonStyle, button_look_semantic};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a button: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Button<D = ()> {
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
    radius: Option<ButtonRadiusPreset>,
    icon: Option<ControlIcon>,
    role: ButtonFamilyRole,
    round: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    data: D,
    content: Option<ControlPresenter<ButtonRenderModel<D>>>,
    modifiers: Vec<ButtonTemplateModifier<D>>,
}

impl Button<()> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            style: ShadcnButtonStyle::Secondary,
            size: ShadcnSize::Md,
            radius: None,
            icon: None,
            role: ButtonFamilyRole::Text,
            round: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            data: (),
            content: None,
            modifiers: Vec::new(),
        }
    }

    /// Icon-only command button (square chrome).
    pub fn icon_button(id: impl Into<SharedString>, icon: impl Into<ControlIcon>) -> Self {
        Self::new(id).role(ButtonFamilyRole::Icon).round(true).icon(icon)
    }

    /// Pick a typed payload. Call once, usually at the start of the chain.
    ///
    /// Resets presenter content and queued template modifiers (they are typed on `D`).
    /// Shadcn axes and SDK settings (`icon`, `role`, `enabled`, `tab_stop`, `compact`) are kept.
    pub fn typed<D: Clone + 'static>(self, data: D) -> Button<D> {
        Button {
            id: self.id,
            look: self.look,
            style: self.style,
            size: self.size,
            radius: self.radius,
            icon: self.icon,
            role: self.role,
            round: self.round,
            enabled: self.enabled,
            tab_stop: self.tab_stop,
            compact: self.compact,
            without_elevation: self.without_elevation,
            data,
            content: None,
            modifiers: Vec::new(),
        }
    }
}

impl<D: Clone + 'static> Button<D> {
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

    pub fn radius(mut self, radius: ButtonRadiusPreset) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn with_icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Semantic leading icon on a labeled button.
    pub fn icon(self, icon: impl Into<ControlIcon>) -> Self {
        self.with_icon(icon)
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.role = role;
        self
    }

    pub fn round(mut self, round: bool) -> Self {
        self.round = round;
        self
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

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<luma::controls::button::Button<D>> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ButtonBuilder<D> {
        let style = self.style;
        let radius = self.radius;
        let template = std::sync::Arc::new(DefaultButtonTemplate::new(super::templates::styled_button_family_theme(
            look.clone(),
            style,
        )));
        let mut builder = luma::controls::button::Button::new(self.id)
            .typed(self.data)
            .template(template)
            .size(self.size.control_size())
            .role(self.role)
            .round(self.round)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop);
        if let Some(radius) = radius {
            let geometry_look = look.clone();
            builder = builder.with_look(move |model| {
                button_look_semantic(
                    geometry_look.mode_tokens().as_ref(),
                    geometry_look.mode(),
                    style,
                    model.role,
                    model.size,
                    Some(radius),
                    model.state,
                )
            });
        }
        if let Some(icon) = self.icon {
            builder = builder.icon(icon);
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
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

impl<D: 'static> HasPresenter<ButtonRenderModel<D>> for Button<D> {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<D>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::controls::button::HasPresenter;

    #[test]
    fn default_style_is_secondary() {
        assert!(matches!(Button::new("ok").style, ShadcnButtonStyle::Secondary));
    }

    #[test]
    fn style_helpers_set_axes() {
        assert!(matches!(Button::new("ok").primary().style, ShadcnButtonStyle::Primary));
        assert!(matches!(Button::new("ok").outline().style, ShadcnButtonStyle::Outline));
        assert!(matches!(Button::new("ok").ghost().style, ShadcnButtonStyle::Ghost));
        let content_only = Button::new("ok").content_only();
        assert!(matches!(content_only.style, ShadcnButtonStyle::ContentOnly));
        assert!(content_only.without_elevation);
    }

    #[test]
    fn typed_keeps_shadcn_axes_and_resets_d_typed_seams() {
        let button = Button::new("counter")
            .primary()
            .size(ShadcnSize::Lg)
            .label("ignored")
            .with_template_modifier(|root, _| root)
            .typed(7u32);
        assert!(matches!(button.style, ShadcnButtonStyle::Primary));
        assert_eq!(button.size, ShadcnSize::Lg);
        assert_eq!(button.data, 7);
        assert!(button.content.is_none());
        assert!(button.modifiers.is_empty());
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Button::new("ok")
            .look(&look)
            .primary()
            .label("Submit")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
