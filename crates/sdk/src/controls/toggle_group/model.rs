use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{ToggleGroup, ToggleGroupTemplate, default_toggle_group_template};
use crate::controls::button_family::{ButtonKind as ToggleGroupKind, ButtonSize as ToggleGroupSize};
use crate::controls::toggle_group::{ControlFocusState, ToggleGroupItemState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleGroupSelectionMode {
    #[default]
    Single,
    Multiple,
}

#[derive(Clone, Debug)]
pub struct ToggleGroupItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) enabled: bool,
}

impl ToggleGroupItem {
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

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Clone)]
pub struct ToggleGroupModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<ToggleGroupItem>,
    pub(crate) selected_ids: Vec<SharedString>,
    pub(crate) active_id: Option<SharedString>,
    pub(crate) selection_mode: ToggleGroupSelectionMode,
    pub(crate) kind: ToggleGroupKind,
    pub(crate) size: ToggleGroupSize,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ToggleGroupTemplate>,
}

pub struct ToggleGroupRenderItem<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected: bool,
    pub enabled: bool,
    pub position: ToggleGroupItemPosition,
    pub state: ToggleGroupItemState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToggleGroupItemPosition {
    Only,
    First,
    Middle,
    Last,
}

pub struct ToggleGroupRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<ToggleGroupRenderItem<'a>>,
    pub selected_ids: &'a [SharedString],
    pub active_id: Option<&'a SharedString>,
    pub selection_mode: ToggleGroupSelectionMode,
    pub kind: ToggleGroupKind,
    pub size: ToggleGroupSize,
    pub enabled: bool,
    pub focus: ControlFocusState,
}

pub struct ToggleGroupBuilder {
    pub(crate) model: ToggleGroupModel,
}

impl ToggleGroupBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ToggleGroupModel {
                id: id.into(),
                items: Vec::new(),
                selected_ids: Vec::new(),
                active_id: None,
                selection_mode: ToggleGroupSelectionMode::Single,
                kind: ToggleGroupKind::Default,
                size: ToggleGroupSize::Md,
                enabled: true,
                template: default_toggle_group_template(),
            },
        }
    }

    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn selection_mode(mut self, selection_mode: ToggleGroupSelectionMode) -> Self {
        self.model.selection_mode = selection_mode;
        self
    }

    pub fn single(self) -> Self {
        self.selection_mode(ToggleGroupSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.selection_mode(ToggleGroupSelectionMode::Multiple)
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_ids = vec![selected_id.into()];
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.model.selected_ids = selected_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn kind(mut self, kind: ToggleGroupKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ToggleGroupSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ToggleGroupTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ToggleGroup> {
        cx.new(|cx| ToggleGroup::from_builder(self, cx))
    }
}
