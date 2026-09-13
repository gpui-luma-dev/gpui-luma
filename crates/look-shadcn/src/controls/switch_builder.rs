//! Look-owned switch builder. Spawn synthesizes the SDK [`luma::controls::switch::Switch`].

use gpui::{App, Context, IntoElement, Pixels, SharedString};
use luma::controls::button::{ButtonRenderModel, ControlPresenter, HasPresenter};
use luma::controls::switch::{SwitchBuilder, SwitchData, SwitchOrientation};
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a switch: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Switch {
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
    checked: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    animated: bool,
    orientation: Option<SwitchOrientation>,
    track_length: Option<Pixels>,
    track_thickness: Option<Pixels>,
    thumb_size: Option<Pixels>,
    track_length_extra: f32,
    track_content: Option<ControlPresenter<ButtonRenderModel<SwitchData>>>,
    thumb_content: Option<ControlPresenter<ButtonRenderModel<SwitchData>>>,
    content: Option<ControlPresenter<ButtonRenderModel<SwitchData>>>,
}

impl Switch {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            style: ShadcnButtonStyle::Primary,
            size: ShadcnSize::Md,
            checked: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            animated: true,
            orientation: None,
            track_length: None,
            track_thickness: None,
            thumb_size: None,
            track_length_extra: 0.0,
            track_content: None,
            thumb_content: None,
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

    pub fn fixed_geometry(
        mut self,
        track_length: impl Into<Pixels>,
        track_thickness: impl Into<Pixels>,
        thumb_size: impl Into<Pixels>,
    ) -> Self {
        self.track_length = Some(track_length.into());
        self.track_thickness = Some(track_thickness.into());
        self.thumb_size = Some(thumb_size.into());
        self
    }

    pub fn track_length_extra(mut self, extra_length: f32) -> Self {
        self.track_length_extra = extra_length.max(0.0);
        self
    }

    pub fn track_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.track_content = Some(std::sync::Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    pub fn thumb_content<F, E>(mut self, builder: F) -> Self
    where
        F: Fn(&ButtonRenderModel<SwitchData>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.thumb_content = Some(std::sync::Arc::new(move |model, cx| builder(model, cx).into_any_element()));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::switch::Switch {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SwitchBuilder {
        let template = look.switch_template(self.style);
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
        if let (Some(track_length), Some(track_thickness), Some(thumb_size)) =
            (self.track_length, self.track_thickness, self.thumb_size)
        {
            builder = builder.fixed_geometry(track_length, track_thickness, thumb_size);
        }
        if self.track_length_extra > 0.0 {
            builder = builder.track_length_extra(self.track_length_extra);
        }
        if let Some(track_content) = self.track_content {
            builder = builder.track_content(move |model, cx| track_content(model, cx));
        }
        if let Some(thumb_content) = self.thumb_content {
            builder = builder.thumb_content(move |model, cx| thumb_content(model, cx));
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

impl HasPresenter<ButtonRenderModel<SwitchData>> for Switch {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<SwitchData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::controls::button::HasPresenter;

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let switch = Switch::new("notify").outline().size(ShadcnSize::Lg).with_data(true);
        assert!(matches!(switch.style, ShadcnButtonStyle::Outline));
        assert_eq!(switch.size, ShadcnSize::Lg);
        assert!(switch.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Switch::new("notify").checked(true).checked);
        assert!(Switch::new("notify").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Switch::new("ok").look(&look).primary().with_data(true).label("On").into_sdk_builder(look);
    }
}
