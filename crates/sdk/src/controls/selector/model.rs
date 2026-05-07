use std::sync::Arc;

use gpui::{App, AppContext, Bounds, Entity, IntoElement, Pixels, SharedString};

use super::{ControlFocusState, Selector, SelectorState, SelectorTemplate, default_selector_template};
pub use crate::controls::selector_item_template::{
    SelectorItemRenderModel, SelectorItemTemplate, make_selector_item_template,
};
pub use crate::controls::selector_panel::{SelectorItem, SelectorItemLike, SelectorPath, normalize_selector_items};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPlacement {
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
    OverlayOnTrigger,
}

pub struct SelectorModel<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<T>,
    pub(crate) enabled: bool,
    pub(crate) placement: SelectorPlacement,
    pub(crate) item_template: Option<SelectorItemTemplate<T>>,
    pub(crate) template: Arc<dyn SelectorTemplate<T>>,
}

pub struct SelectorRenderModel<'a, T>
where
    T: SelectorItemLike + 'static,
{
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected_index: Option<usize>,
    pub items: &'a [T],
    pub open: bool,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: SelectorPlacement,
    pub active_path: Option<SelectorPath>,
    pub enabled: bool,
    pub item_template: Option<&'a SelectorItemTemplate<T>>,
    pub focus: ControlFocusState,
    pub state: SelectorState,
}

pub struct SelectorBuilder<T = SelectorItem>
where
    T: SelectorItemLike + 'static,
{
    pub(crate) model: SelectorModel<T>,
    pub(crate) initial_selected_id: Option<SharedString>,
}

impl<T> SelectorBuilder<T>
where
    T: SelectorItemLike + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: SelectorModel {
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                placement: SelectorPlacement::Smart,
                item_template: None,
                template: default_selector_template::<T>(),
            },
            initial_selected_id: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn item(mut self, item: T) -> Self {
        if item.is_enabled() {
            self.model.items.push(item);
        }
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.model.items = normalize_selector_items(items);
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.initial_selected_id = Some(selected_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn placement(mut self, placement: SelectorPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(make_selector_item_template(template));
        self
    }

    pub fn template(mut self, template: Arc<dyn SelectorTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Selector<T>> {
        cx.new(|cx| Selector::from_builder(self, cx))
    }
}
