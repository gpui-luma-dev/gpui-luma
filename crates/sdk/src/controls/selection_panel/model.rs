use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, IntoElement, SharedString};

use crate::controls::icon::IconSource;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};
use crate::controls::selection_panel::control::SelectionPanelControl;
use crate::controls::selection_panel::template::{SelectionPanelTemplate, default_selection_panel_template};
use crate::controls::selection_panel::theme::{SelectionPanelAppearance, default_selection_panel_appearance};
use crate::theme::{ControlSize, ThemeTokens};

pub trait SelectionPanelItemLike {
    fn id(&self) -> &SharedString;

    fn label(&self) -> &SharedString {
        self.id()
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn icon(&self) -> Option<&IconSource> {
        None
    }

    fn is_selected(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct SelectionPanelItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<IconSource>,
    pub(crate) enabled: bool,
    pub(crate) selected: bool,
}

impl SelectionPanelItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self { label: id.clone(), id, icon: None, enabled: true, selected: false }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn label_text(&self) -> &SharedString {
        &self.label
    }
}

impl SelectionPanelItemLike for SelectionPanelItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn icon(&self) -> Option<&IconSource> {
        self.icon.as_ref()
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionPanelPath {
    Item(usize),
}

impl SelectionPanelPath {
    pub fn is_item(self, index: usize) -> bool {
        matches!(self, Self::Item(active) if active == index)
    }
}

#[derive(Clone, Debug)]
pub struct SelectionPanelPresenterModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub sibling_count: usize,
}

#[derive(Clone, Debug)]
pub struct SelectionPanelItemRenderModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub focus_visible: bool,
    pub enabled: bool,
    pub sibling_count: usize,
}

pub type SelectionPanelPresenter<T> =
    Arc<dyn for<'a> Fn(&SelectionPanelPresenterModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub type SelectionPanelItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_selection_panel_presenter<T, F, E>(presenter: F) -> SelectionPanelPresenter<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(&SelectionPanelPresenterModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| presenter(model, cx).into_any_element())
}

pub fn make_selection_panel_item_template<T, F, E>(template: F) -> SelectionPanelItemTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

pub type SelectionPanelAppearanceProvider =
    Arc<dyn Fn(ControlSize) -> SelectionPanelAppearance + Send + Sync + 'static>;

#[derive(Clone)]
pub struct SelectionPanelModel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) panel_id: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) visible_indices: Vec<usize>,
    pub(crate) selected_source_index: Option<usize>,
    pub(crate) open: bool,
    pub(crate) enabled: bool,
    pub(crate) show_selection_marker: bool,
    pub(crate) scrolling: bool,
    pub(crate) min_visible_rows: usize,
    pub(crate) max_visible_rows: usize,
    pub(crate) size: ControlSize,
    pub(crate) presenter: Option<SelectionPanelPresenter<T>>,
    pub(crate) template: Arc<dyn SelectionPanelTemplate<T>>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) appearance_provider: SelectionPanelAppearanceProvider,
}

pub struct SelectionPanelBuilder<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub(crate) model: SelectionPanelModel<T>,
    pub(crate) initial_active_visible_index: Option<usize>,
}

impl<T> SelectionPanelBuilder<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: SelectionPanelModel {
                panel_id: format!("{}-panel", id).into(),
                id,
                items: Vec::new(),
                visible_indices: Vec::new(),
                selected_source_index: None,
                open: true,
                enabled: true,
                show_selection_marker: true,
                scrolling: true,
                min_visible_rows: 1,
                max_visible_rows: 7,
                size: ControlSize::Md,
                presenter: None,
                template: default_selection_panel_template(),
                scrollbar_template: default_scrollbar_template(),
                appearance_provider: Arc::new(|size| default_selection_panel_appearance(&ThemeTokens::default(), size)),
            },
            initial_active_visible_index: None,
        }
    }

    pub fn panel_id(mut self, panel_id: impl Into<SharedString>) -> Self {
        self.model.panel_id = panel_id.into();
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn visible_indices(mut self, visible_indices: impl IntoIterator<Item = usize>) -> Self {
        self.model.visible_indices = visible_indices.into_iter().collect();
        self
    }

    pub fn selected_source_index(mut self, selected_source_index: Option<usize>) -> Self {
        self.model.selected_source_index = selected_source_index;
        self
    }

    pub fn active_visible_index(mut self, active_visible_index: Option<usize>) -> Self {
        self.initial_active_visible_index = active_visible_index;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn show_selection_marker(mut self, show_selection_marker: bool) -> Self {
        self.model.show_selection_marker = show_selection_marker;
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.model.scrolling = scrolling;
        self
    }

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.model.min_visible_rows = min_visible_rows.max(1);
        if self.model.max_visible_rows < self.model.min_visible_rows {
            self.model.max_visible_rows = self.model.min_visible_rows;
        }
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.model.max_visible_rows = max_visible_rows.max(1);
        if self.model.min_visible_rows > self.model.max_visible_rows {
            self.model.min_visible_rows = self.model.max_visible_rows;
        }
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn template(mut self, template: Arc<dyn SelectionPanelTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn presenter(mut self, presenter: SelectionPanelPresenter<T>) -> Self {
        self.model.presenter = Some(presenter);
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        let item_template = make_selection_panel_item_template(template);
        self.model.presenter = Some(Arc::new(move |model, cx| {
            item_template(
                &SelectionPanelItemRenderModel {
                    panel_id: model.panel_id,
                    control_id: model.control_id,
                    item: model.item,
                    source_index: model.source_index,
                    visible_index: model.visible_index,
                    selected: model.selected,
                    active: model.active,
                    hovered: model.hovered,
                    pressed: model.pressed,
                    focused: model.focused,
                    focus_visible: model.focus_visible,
                    enabled: model.enabled,
                    sibling_count: model.sibling_count,
                },
                cx,
            )
        }));
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn appearance_provider(mut self, provider: SelectionPanelAppearanceProvider) -> Self {
        self.model.appearance_provider = provider;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SelectionPanelControl<T>> {
        cx.new(|cx| SelectionPanelControl::from_builder(self, cx))
    }
}

pub fn new(id: impl Into<SharedString>) -> SelectionPanelBuilder<SelectionPanelItem> {
    SelectionPanelBuilder::new(id)
}
