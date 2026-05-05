use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Bounds, Entity, IntoElement, Pixels, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{ControlFocusState, MenuPath, Selector, SelectorState, SelectorTemplate, default_selector_template};

#[derive(Clone, Debug)]
pub enum SelectorItemIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl SelectorItemIcon {
    pub fn lucide(&self) -> Option<LucideIcon> {
        match self {
            Self::Lucide(icon) => Some(*icon),
            Self::SvgPath(_) => None,
        }
    }

    pub fn svg_path(&self) -> Option<&SharedString> {
        match self {
            Self::Lucide(_) => None,
            Self::SvgPath(path) => Some(path),
        }
    }
}

impl From<LucideIcon> for SelectorItemIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

impl From<&str> for SelectorItemIcon {
    fn from(icon: &str) -> Self {
        Self::SvgPath(icon.to_string().into())
    }
}

impl From<String> for SelectorItemIcon {
    fn from(icon: String) -> Self {
        Self::from(icon.as_str())
    }
}

impl From<SharedString> for SelectorItemIcon {
    fn from(icon: SharedString) -> Self {
        Self::SvgPath(icon)
    }
}

pub trait SelectorItemLike {
    fn id(&self) -> &SharedString;

    fn is_enabled(&self) -> bool {
        true
    }

    fn label_text(&self) -> &SharedString {
        self.id()
    }

    fn icon(&self) -> Option<&SelectorItemIcon> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct SelectorItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<SelectorItemIcon>,
    pub(crate) enabled: bool,
}

impl SelectorItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self { label: id.clone(), id, icon: None, enabled: true }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<SelectorItemIcon>) -> Self {
        self.icon = Some(icon.into());
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

    pub fn icon_ref(&self) -> Option<&SelectorItemIcon> {
        self.icon.as_ref()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl SelectorItemLike for SelectorItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn label_text(&self) -> &SharedString {
        &self.label
    }

    fn icon(&self) -> Option<&SelectorItemIcon> {
        self.icon.as_ref()
    }
}

pub(crate) fn normalize_selector_items<T: SelectorItemLike>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    items.into_iter().filter(SelectorItemLike::is_enabled).collect()
}

pub fn make_item_template<T, F, E>(template: F) -> SelectorItemTemplate<T>
where
    T: SelectorItemLike + 'static,
    F: for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPlacement {
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
    OverlayOnTrigger,
}

pub type SelectorItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct SelectorItemRenderModel<'a, T> {
    pub selector_id: &'a SharedString,
    pub item: &'a T,
    pub index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
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
    pub selected_icon: Option<&'a SelectorItemIcon>,
    pub selected_index: Option<usize>,
    pub items: &'a [T],
    pub open: bool,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: SelectorPlacement,
    pub active_path: Option<MenuPath>,
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
        self.model.item_template = Some(make_item_template(template));
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
