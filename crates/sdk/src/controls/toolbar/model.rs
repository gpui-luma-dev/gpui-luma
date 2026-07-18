use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, FocusHandle, IntoElement, ParentElement, SharedString, Window, div};

use super::control::Toolbar;
use super::template::{ToolbarTemplate, default_toolbar_template, modified_toolbar_template};
use super::theme::ToolbarVariant;
use crate::controls::control_group::{ControlGroupArrowPolicy, ControlGroupItemLike};
use crate::controls::presenter::{HostedContent, Presenter};
use crate::controls::state::CompositeItemState;
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarItemKind {
    Hosted,
    Separator,
}

#[derive(Clone)]
pub struct ToolbarItem {
    pub(crate) id: SharedString,
    pub(crate) kind: ToolbarItemKind,
    pub(crate) enabled: bool,
    pub(crate) label: SharedString,
    pub(crate) focus_handle: Option<FocusHandle>,
    pub(crate) arrow_policy: ControlGroupArrowPolicy,
    pub(crate) content: Option<Presenter<ToolbarItemRenderModel>>,
}

impl ToolbarItem {
    pub fn button<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn toggle<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn menu<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        Self::hosted(id, content)
    }

    pub fn hosted<F, E>(id: impl Into<SharedString>, content: F) -> Self
    where
        F: Fn(&ToolbarItemRenderModel, &mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        let id = id.into();
        Self {
            label: id.clone(),
            id,
            kind: ToolbarItemKind::Hosted,
            enabled: true,
            focus_handle: None,
            arrow_policy: ControlGroupArrowPolicy::GroupOwns,
            content: Some(Presenter::new(move |model, window, cx| HostedContent {
                element: content(model, window, cx).into_any_element(),
                focus_handle: None,
            })),
        }
    }

    pub fn separator(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            label: id.clone(),
            id,
            kind: ToolbarItemKind::Separator,
            enabled: false,
            focus_handle: None,
            arrow_policy: ControlGroupArrowPolicy::GroupOwns,
            content: None,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn arrow_policy(mut self, arrow_policy: ControlGroupArrowPolicy) -> Self {
        self.arrow_policy = arrow_policy;
        self
    }

    pub fn is_separator(&self) -> bool {
        self.kind == ToolbarItemKind::Separator
    }
}

impl ControlGroupItemLike for ToolbarItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled && !self.is_separator()
    }
}

#[derive(Clone)]
pub(crate) struct ToolbarModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ToolbarItem>,
    pub(crate) enabled: bool,
    pub(crate) size: ControlSize,
    pub(crate) variant: ToolbarVariant,
    pub(crate) template: Arc<dyn ToolbarTemplate>,
}

#[derive(Clone)]
pub struct ToolbarItemRenderModel {
    pub id: SharedString,
    pub kind: ToolbarItemKind,
    pub index: usize,
    pub enabled: bool,
    pub state: CompositeItemState,
}

pub struct ToolbarRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ToolbarItemRenderModel>,
    pub enabled: bool,
    pub size: Option<ControlSize>,
    pub variant: ToolbarVariant,
}

pub struct ToolbarBuilder {
    pub(crate) model: ToolbarModel,
}

impl ToolbarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ToolbarModel {
                id: id.into(),
                items: Vec::new(),
                enabled: true,
                size: ControlSize::Md,
                variant: ToolbarVariant::Outline,
                template: default_toolbar_template(),
            },
        }
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn variant(mut self, variant: ToolbarVariant) -> Self {
        self.model.variant = variant;
        self
    }

    pub fn outline(self) -> Self {
        self.variant(ToolbarVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ToolbarVariant::Ghost)
    }

    pub fn template(mut self, template: Arc<dyn ToolbarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ToolbarRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_toolbar_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Toolbar> {
        cx.new(|cx| Toolbar::from_builder(self, cx))
    }

    pub(crate) fn control_group_template(&self) -> crate::controls::control_group::ControlGroupTemplate<ToolbarItem> {
        super::template::toolbar_control_group_template(
            self.model.size,
            self.model.variant,
            Arc::clone(&self.model.template),
        )
    }
}

pub(crate) fn fallback_item(model: &ToolbarItemRenderModel) -> AnyElement {
    div().child(model.id.clone()).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::control_group::ControlGroupArrowPolicy;

    #[test]
    fn separator_is_excluded_from_navigation() {
        let item = ToolbarItem::separator("break");
        assert!(item.is_separator());
        assert!(!item.is_enabled());
    }

    #[test]
    fn hosted_item_defaults_to_group_arrow_policy() {
        let item = ToolbarItem::hosted("field", |_, _, _| div());
        assert_eq!(item.arrow_policy, ControlGroupArrowPolicy::GroupOwns);
        assert!(item.is_enabled());
    }

    #[test]
    fn builder_defaults_to_outline_variant() {
        let builder = ToolbarBuilder::new("toolbar-test");
        assert_eq!(builder.model.variant, ToolbarVariant::Outline);

        let ghost = ToolbarBuilder::new("toolbar-test").ghost();
        assert_eq!(ghost.model.variant, ToolbarVariant::Ghost);
    }
}
