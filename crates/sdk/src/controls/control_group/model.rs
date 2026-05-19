use std::sync::Arc;

use gpui::{App, AppContext, Entity, IntoElement, SharedString};

use super::control::ControlGroupControl;
use super::template::{
    ControlGroupItemTemplate, ControlGroupTemplate, default_control_group_template, make_control_group_item_template,
};
use crate::controls::state::{CompositeItemState, ControlFocusState};

pub trait ControlGroupItemLike {
    fn id(&self) -> &SharedString;

    fn label(&self) -> &SharedString {
        self.id()
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct ControlGroupItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl ControlGroupItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl ControlGroupItemLike for ControlGroupItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlSelectionMode {
    SingleRequired,
    #[default]
    SingleAllowNone,
    Multiple,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlGroupStateMode {
    #[default]
    Unmanaged,
    Managed,
}

#[derive(Clone)]
pub struct ControlGroupModel<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) default_selected_ids: Vec<SharedString>,
    pub(crate) managed_selected_ids: Option<Vec<SharedString>>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) selection_mode: ControlSelectionMode,
    pub(crate) state_mode: ControlGroupStateMode,
    pub(crate) enabled: bool,
    pub(crate) template: ControlGroupTemplate<T>,
    pub(crate) item_template: Option<ControlGroupItemTemplate<T>>,
}

impl<T> ControlGroupModel<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub fn effective_selected_ids(&self) -> &[SharedString] {
        self.managed_selected_ids.as_deref().unwrap_or(self.default_selected_ids.as_slice())
    }

    pub fn effective_selected_id(&self) -> Option<&SharedString> {
        self.effective_selected_ids().first()
    }

    pub fn set_managed_selected_ids<I, S>(&mut self, selected_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.state_mode = ControlGroupStateMode::Managed;
        self.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
    }

    pub fn clear_managed_selected_ids(&mut self) {
        self.managed_selected_ids = None;
    }
}

#[derive(Clone, Debug)]
pub struct ControlGroupItemRenderModel<'a, T>
where
    T: ControlGroupItemLike + 'static,
{
    pub group_id: &'a SharedString,
    pub item: &'a T,
    pub index: usize,
    pub sibling_count: usize,
    pub selected: bool,
    pub active: bool,
    pub enabled: bool,
    pub state: CompositeItemState,
    pub selection_mode: ControlSelectionMode,
}

pub struct ControlGroupRenderModel<'a, T>
where
    T: ControlGroupItemLike + 'static,
{
    pub id: &'a SharedString,
    pub items: Vec<ControlGroupItemRenderModel<'a, T>>,
    pub selected_ids: &'a [SharedString],
    pub active_id: Option<&'a SharedString>,
    pub selection_mode: ControlSelectionMode,
    pub state_mode: ControlGroupStateMode,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: Option<&'a ControlGroupItemTemplate<T>>,
}

pub struct ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(crate) model: ControlGroupModel<T>,
}

impl<T> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ControlGroupModel {
                id: id.into(),
                items: Vec::new(),
                default_selected_ids: Vec::new(),
                managed_selected_ids: None,
                active_id: None,
                selection_mode: ControlSelectionMode::SingleAllowNone,
                state_mode: ControlGroupStateMode::Unmanaged,
                enabled: true,
                template: default_control_group_template(),
                item_template: None,
            },
        }
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn mode(mut self, selection_mode: ControlSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn selection_mode(self, selection_mode: ControlSelectionMode) -> Self {
        self.mode(selection_mode)
    }

    pub fn single_required(self) -> Self {
        self.mode(ControlSelectionMode::SingleRequired)
    }

    pub fn single_allow_none(self) -> Self {
        self.mode(ControlSelectionMode::SingleAllowNone)
    }

    pub fn multiple(self) -> Self {
        self.mode(ControlSelectionMode::Multiple)
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.default_selected_ids = vec![selected_id.into()];
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.default_selected_ids = selected_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn managed(mut self) -> Self {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self
    }

    pub fn unmanaged(mut self) -> Self {
        self.model.state_mode = ControlGroupStateMode::Unmanaged;
        self.model.managed_selected_ids = None;
        self
    }

    pub fn managed_selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(vec![selected_id.into()]);
        self
    }

    pub fn managed_selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.state_mode = ControlGroupStateMode::Managed;
        self.model.managed_selected_ids = Some(selected_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.model.active_id = Some(active_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn item_template(mut self, template: ControlGroupItemTemplate<T>) -> Self {
        self.model.item_template = Some(template);
        self
    }

    pub fn with_item_template<F, E>(self, template: F) -> Self
    where
        F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.item_template(make_control_group_item_template(template))
    }

    pub fn clear_item_template(mut self) -> Self {
        self.model.item_template = None;
        self
    }

    pub fn template(mut self, template: ControlGroupTemplate<T>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template<F>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(
                &ControlGroupRenderModel<'a, T>,
                super::template::ControlGroupTemplateHandlers,
                &mut gpui::Window,
                &mut App,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = Arc::new(template);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ControlGroupControl<T>> {
        cx.new(|cx| ControlGroupControl::from_builder(self, cx))
    }
}
