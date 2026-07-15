use std::cell::Cell;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, SharedString, div, prelude::*};

use super::control::Button;
use super::template::{ButtonTemplate, modified_button_template};
pub use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyRole, ButtonInteractionState as ButtonState, ButtonSize,
};
use lucide_icons::Icon as LucideIcon;

#[derive(Clone)]
pub enum ControlIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl From<LucideIcon> for ControlIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

pub type ButtonLookSource<D> = Arc<dyn Fn(&ButtonRenderModel<D>) -> ButtonFamilyLook + Send + Sync>;

#[derive(Clone)]
pub struct ButtonModel<D = ()> {
    pub(crate) id: SharedString,
    pub(crate) data: D,
    pub(crate) content: ControlPresenter<ButtonRenderModel<D>>,
    pub(crate) role: ButtonFamilyRole,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) tab_stop: bool,
    pub(crate) round: bool,
    pub(crate) elevation: bool,
    pub(crate) compact: bool,
    pub(crate) suppress_adorners: bool,
    pub(crate) switch_track_width_extra: f32,
    pub(crate) switch_orientation: crate::controls::switch::SwitchOrientation,
    pub(crate) switch_track_content: Option<ControlPresenter<ButtonRenderModel<D>>>,
    pub(crate) switch_thumb_content: Option<ControlPresenter<ButtonRenderModel<D>>>,
    pub(crate) look: Option<ButtonLookSource<D>>,
    pub(crate) template: Arc<dyn ButtonTemplate<D>>,
}

pub struct ButtonRenderModel<D> {
    pub id: SharedString,
    pub data: D,
    pub content: ControlPresenter<ButtonRenderModel<D>>,
    pub role: ButtonFamilyRole,
    pub size: ButtonSize,
    pub state: ButtonState,
    pub round: bool,
    pub radius_override: Cell<Option<f32>>,
    pub elevation: bool,
    pub compact: bool,
    pub suppress_adorners: Cell<bool>,
    pub switch_track_width_extra: f32,
    pub switch_orientation: crate::controls::switch::SwitchOrientation,
    pub switch_track_content: Option<ControlPresenter<ButtonRenderModel<D>>>,
    pub switch_thumb_content: Option<ControlPresenter<ButtonRenderModel<D>>>,
    pub look: Option<ButtonLookSource<D>>,
}

impl<D: Default> Default for ButtonRenderModel<D> {
    fn default() -> Self {
        Self {
            id: SharedString::default(),
            data: D::default(),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::default(),
            size: ButtonSize::default(),
            state: ButtonState::default(),
            round: false,
            radius_override: Cell::new(None),
            elevation: true,
            compact: false,
            suppress_adorners: Cell::new(false),
            switch_track_width_extra: 0.0,
            switch_orientation: crate::controls::switch::SwitchOrientation::Horizontal,
            switch_track_content: None,
            switch_thumb_content: None,
            look: None,
        }
    }
}

pub struct ButtonBuilder<D = ()> {
    pub(crate) model: ButtonModel<D>,
}

impl ButtonBuilder<()> {
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<()> {
        let id = id.into();

        ButtonBuilder {
            model: ButtonModel {
                id: id.clone(),
                data: (),
                content: Arc::new(move |_, _| div().child(id.clone()).into_any_element()),
                role: ButtonFamilyRole::Text,
                size: ButtonSize::Md,
                enabled: true,
                tab_stop: true,
                round: false,
                elevation: true,
                compact: false,
                suppress_adorners: false,
                switch_track_width_extra: 0.0,
                switch_orientation: crate::controls::switch::SwitchOrientation::Horizontal,
                switch_track_content: None,
                switch_thumb_content: None,
                look: None,
                template: super::template::default_button_template(),
            },
        }
    }
}

impl ButtonBuilder<()> {
    /// Converts an untyped button builder into a typed one.
    ///
    /// # Why this exists
    ///
    /// [`ButtonBuilder`] starts as [`ButtonBuilder<()>`] — a plain command control with no
    /// application payload. Many controls (checkbox, toggle, custom reactive buttons) need a
    /// typed `D` that templates and handlers read via [`ButtonRenderModel::data`].
    ///
    /// Call `typed` once at the start of a builder chain to pick that payload type and its
    /// initial value. This is **not** the same as [`ButtonBuilder::with_data`]: `typed` creates a
    /// fresh [`ButtonBuilder<D>`] and resets content and template to button defaults. Use
    /// [`ButtonBuilder::with_data`] only after the builder is already configured (template,
    /// content, style, etc.) to change the initial payload without discarding that configuration.
    ///
    /// # Data binding
    ///
    /// Today, payload is set at build time and updated imperatively via
    /// [`Button::set_data`](super::control::Button::set_data). A richer binding layer — where
    /// external state drives control data automatically — is not implemented yet; `typed` /
    /// `with_data` are the low-level hooks that future binding would sit on top of.
    pub fn typed<D: Clone + 'static>(self, data: D) -> ButtonBuilder<D> {
        let old = self.model;
        let id = old.id.clone();
        ButtonBuilder {
            model: ButtonModel {
                id: old.id,
                data: data.clone(),
                content: Arc::new(move |_, _| div().child(id.clone()).into_any_element()),
                role: old.role,
                size: old.size,
                enabled: old.enabled,
                tab_stop: old.tab_stop,
                round: old.round,
                elevation: old.elevation,
                compact: old.compact,
                suppress_adorners: old.suppress_adorners,
                switch_track_width_extra: 0.0,
                switch_orientation: crate::controls::switch::SwitchOrientation::Horizontal,
                switch_track_content: None,
                switch_thumb_content: None,
                look: None,
                template: super::template::default_button_template(),
            },
        }
    }
}

impl<D: Clone + 'static> ButtonBuilder<D> {
    /// Sets the control payload without changing template, content, or other builder config.
    ///
    /// Prefer this over [`ButtonBuilder::typed`] when the builder is already specialized — for
    /// example after `checkbox::new(...)` — and you only need a different initial checked/on
    /// value.
    pub fn with_data(mut self, data: D) -> Self {
        self.model.data = data;
        self
    }

    pub fn with_look<F>(mut self, resolve: F) -> Self
    where
        F: Fn(&ButtonRenderModel<D>) -> ButtonFamilyLook + Send + Sync + 'static,
    {
        self.model.look = Some(Arc::new(resolve));
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.model.role = role;
        self
    }

    pub fn round(mut self, round: bool) -> Self {
        self.model.round = round;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Sets whether this button participates in Tab-key focus traversal.
    ///
    /// Setting this to `false` removes the button from the tab order, but does not
    /// prevent mouse interaction or programmatic focus. This is useful when composing
    /// buttons inside a container control that owns keyboard focus/navigation.
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    /// Disables indicator/body elevation shadow paint and layout reservation.
    pub fn without_elevation(mut self) -> Self {
        self.model.elevation = false;
        self
    }

    /// Suppresses template adorners and any layout reserve they require.
    ///
    /// The current default templates only render a focus adorner, but the flag is intentionally
    /// plural because templates may add more adorners over time.
    pub fn without_adorners(mut self) -> Self {
        self.model.suppress_adorners = true;
        self
    }

    /// Dense embedding: no elevation and no focus-ring layout reserve until focused.
    pub fn compact(mut self) -> Self {
        self.model.compact = true;
        self.model.elevation = false;
        self
    }

    pub fn template(mut self, template: Arc<dyn ButtonTemplate<D>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ButtonRenderModel<D>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_button_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<Button<D>> {
        cx.new(|cx| Button::from_builder(self, cx))
    }
}

impl<D: 'static> HasPresenter<ButtonRenderModel<D>> for ButtonBuilder<D> {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<D>>) {
        self.model.content = content;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::checkbox::default_checkbox_template;

    #[test]
    fn compact_sets_elevation_off() {
        let builder = ButtonBuilder::new("compact-test").compact();
        assert!(!builder.model.elevation);
        assert!(builder.model.compact);
    }

    #[test]
    fn without_adorners_sets_suppression_flag() {
        let builder = ButtonBuilder::new("content-only-test").without_adorners();
        assert!(builder.model.suppress_adorners);
    }

    #[test]
    fn with_data_preserves_specialized_template() {
        let template = default_checkbox_template();
        let builder = ButtonBuilder::new("checkbox-test").typed(false).template(template.clone()).with_data(true);

        assert!(std::sync::Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_checkbox_template();
        let builder = ButtonBuilder::new("checkbox-test")
            .typed(false)
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!std::sync::Arc::ptr_eq(&builder.model.template, &template));
    }
}
