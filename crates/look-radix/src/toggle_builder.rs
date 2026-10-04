//! Look-owned toggle builder. Spawn synthesizes the SDK [`gpui_luma::controls::toggle::Toggle`].

use gpui_luma::infra::attachments::TooltipEntityExt;
use gpui::{App, Context, SharedString};
use gpui_luma::controls::button::{ButtonContentContext, ControlIcon, ControlPresenter, HasPresenter};
use gpui_luma::controls::toggle::{ToggleBuilder, ToggleData};

use crate::button::Paint;
use crate::button_layout::ButtonSize;
use crate::look::{Look, resolve_look};
use crate::toggle::toggle_template_for_size;
use crate::tone::Tone;

/// Builder in the guise of a toggle: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Toggle {
    id: SharedString,
    look: Option<Look>,
    page: bool,
    paint: Paint,
    size: ButtonSize,
    selected: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    animated: bool,
    round: bool,
    icon: Option<ControlIcon>,
    content: Option<ControlPresenter<ButtonContentContext<ToggleData>>>,
}

impl Toggle {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            page: false,
            paint: Paint::accent(),
            size: ButtonSize::default(),
            selected: false,
            enabled: true,
            tab_stop: true,
            compact: false,
            without_elevation: false,
            animated: true,
            round: false,
            icon: None,
            content: None,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Screen-nav / page chrome (`ButtonVariant::Page`) instead of a soft toggle.
    /// Paint is kept but ignored — Page is a recipe, not a Soft paint axis.
    pub fn page(mut self) -> Self {
        self.page = true;
        self
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

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// SDK payload seam (`selected`). Toggle is not generic over `D`.
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

    pub fn round(mut self) -> Self {
        self.round = true;
        self
    }

    pub fn icon(mut self, icon: impl Into<ControlIcon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> gpui_luma::controls::toggle::Toggle {
        let look = self.resolve_look(cx);
        let theme = crate::tooltip::tooltip_theme(&look);
        self.into_sdk_builder(look).spawn(cx).with_tooltip_theme(theme, cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> ToggleBuilder {
        let template = toggle_template_for_size(&look, self.paint, self.page, self.size);
        let mut builder = gpui_luma::controls::toggle::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.selected)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .animated(self.animated);
        if let Some(icon) = self.icon {
            builder = builder.icon(icon);
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if self.round {
            builder = builder.round(true);
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<ToggleData>> for Toggle {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<ToggleData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Toggle::new("s").size(ButtonSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Toggle::new("s").size(ButtonSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Toggle::new("s").size(ButtonSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn with_data_keeps_page_axis() {
        let toggle = Toggle::new("page").page().size(ButtonSize::One).with_data(true);
        assert!(toggle.page);
        assert_eq!(toggle.size, ButtonSize::One);
        assert!(toggle.selected);
    }

    #[test]
    fn selected_alias_sets_same_field_as_with_data() {
        assert!(Toggle::new("t").selected(true).selected);
        assert!(Toggle::new("t").with_data(true).selected);
    }

    #[test]
    fn paint_kept_across_page_and_selected() {
        let toggle = Toggle::new("t").gray().high_contrast(true).page().selected(true);
        assert_eq!(toggle.paint.tone, Tone::Gray);
        assert!(toggle.paint.high_contrast);
        assert!(toggle.page);
        assert!(toggle.selected);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Toggle::new("ok").look(&look).page().with_data(true).label("Colors").into_sdk_builder(look);
    }
}
