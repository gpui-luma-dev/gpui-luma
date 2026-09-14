//! Look-owned checkbox builder. Spawn synthesizes the SDK [`luma::controls::checkbox::Checkbox`].

use gpui::{App, Context, SharedString};
use luma::controls::button::{ButtonContentContext, ControlPresenter, HasPresenter};
use luma::controls::checkbox::{CheckboxBuilder, CheckboxData};
use luma::infra::icon::SelectionStatusIcons;
use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a checkbox: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Checkbox {
    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
    checked: bool,
    enabled: bool,
    tab_stop: bool,
    compact: bool,
    without_elevation: bool,
    indicator_only: bool,
    animated: bool,
    icons: Option<SelectionStatusIcons>,
    content: Option<ControlPresenter<ButtonContentContext<CheckboxData>>>,
}

impl Checkbox {
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
            indicator_only: false,
            animated: true,
            icons: None,
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

    /// SDK payload seam (`checked`). Checkbox is not generic over `D`.
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

    pub fn indicator_only(mut self) -> Self {
        self.indicator_only = true;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.icons = Some(icons);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::checkbox::Checkbox {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> CheckboxBuilder {
        let template = look.checkbox_template(self.style);
        let mut builder = luma::controls::checkbox::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .with_data(self.checked)
            .enabled(self.enabled)
            .tab_stop(self.tab_stop)
            .animated(self.animated);
        if self.indicator_only {
            builder = builder.indicator_only();
        }
        if self.compact {
            builder = builder.compact();
        }
        if self.without_elevation {
            builder = builder.without_elevation();
        }
        if let Some(icons) = self.icons {
            builder = builder.icons(icons);
        }
        if let Some(content) = self.content {
            HasPresenter::set_presenter(&mut builder, content);
        }
        builder
    }
}

impl HasPresenter<ButtonContentContext<CheckboxData>> for Checkbox {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonContentContext<CheckboxData>>) {
        self.content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::controls::button::HasPresenter;

    #[test]
    fn with_data_keeps_shadcn_axes() {
        let box_ = Checkbox::new("task").outline().size(ShadcnSize::Lg).with_data(true);
        assert!(matches!(box_.style, ShadcnButtonStyle::Outline));
        assert_eq!(box_.size, ShadcnSize::Lg);
        assert!(box_.checked);
    }

    #[test]
    fn checked_alias_sets_same_field_as_with_data() {
        assert!(Checkbox::new("task").checked(true).checked);
        assert!(Checkbox::new("task").with_data(true).checked);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Checkbox::new("ok").look(&look).primary().with_data(true).label("Done").into_sdk_builder(look);
    }
}
