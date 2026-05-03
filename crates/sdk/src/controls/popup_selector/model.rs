use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Bounds, Entity, IntoElement, Pixels, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{
    ControlFocusState, MenuPath, PopupSelector, PopupSelectorState, PopupSelectorTemplate,
    default_popup_selector_template,
};

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

pub(crate) fn normalize_popup_selector_items(items: impl IntoIterator<Item = SelectorItem>) -> Vec<SelectorItem> {
    items.into_iter().filter(SelectorItem::is_enabled).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopupSelectorPlacement {
    Smart,
    BelowStart,
    AboveStart,
    CenteredOnTrigger,
    OverlayOnTrigger,
}

pub type PopupSelectorItemTemplate =
    Arc<dyn for<'a> Fn(&PopupSelectorItemRenderModel<'a>, &mut App) -> AnyElement + Send + Sync + 'static>;

#[derive(Clone, Debug)]
pub struct PopupSelectorItemRenderModel<'a> {
    pub selector_id: &'a SharedString,
    pub item: &'a SelectorItem,
    pub index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct PopupSelectorModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) items: Vec<SelectorItem>,
    pub(crate) enabled: bool,
    pub(crate) placement: PopupSelectorPlacement,
    pub(crate) item_template: Option<PopupSelectorItemTemplate>,
    pub(crate) template: Arc<dyn PopupSelectorTemplate>,
}

pub struct PopupSelectorRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub selected_icon: Option<&'a SelectorItemIcon>,
    pub selected_index: Option<usize>,
    pub items: &'a [SelectorItem],
    pub open: bool,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: PopupSelectorPlacement,
    pub active_path: Option<MenuPath>,
    pub enabled: bool,
    pub item_template: Option<&'a PopupSelectorItemTemplate>,
    pub focus: ControlFocusState,
    pub state: PopupSelectorState,
}

pub struct PopupSelectorBuilder {
    pub(crate) model: PopupSelectorModel,
}

impl PopupSelectorBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: PopupSelectorModel {
                label: id.clone(),
                id,
                items: Vec::new(),
                enabled: true,
                placement: PopupSelectorPlacement::Smart,
                item_template: None,
                template: default_popup_selector_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn item(mut self, item: SelectorItem) -> Self {
        if item.is_enabled() {
            self.model.items.push(item);
        }
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectorItem>) -> Self {
        self.model.items = normalize_popup_selector_items(items);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn placement(mut self, placement: PopupSelectorPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&PopupSelectorItemRenderModel<'a>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.item_template = Some(Arc::new(move |model, cx| template(model, cx).into_any_element()));
        self
    }

    pub fn template(mut self, template: Arc<dyn PopupSelectorTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<PopupSelector> {
        cx.new(|cx| PopupSelector::from_builder(self, cx))
    }
}
