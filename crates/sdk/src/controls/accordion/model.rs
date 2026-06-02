use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, IntoElement, SharedString};
use lucide_icons::Icon as LucideIcon;

use super::{AccordionControl, AccordionTemplate, default_accordion_template};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AccordionSelectionMode {
    /// Only one item can be expanded at a time. Expanding another collapses the active one.
    #[default]
    Single,
    /// Multiple items can be expanded simultaneously.
    Multiple,
}

/// Represents the header/trigger area of an accordion item.
#[derive(Clone)]
pub struct AccordionTrigger {
    pub(crate) label: Option<SharedString>,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) custom_element: Option<Arc<dyn Fn() -> AnyElement + Send + Sync>>,
}

impl AccordionTrigger {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: Some(label.into()), icon: None, custom_element: None }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn custom(custom: impl Fn() -> AnyElement + Send + Sync + 'static) -> Self {
        Self { label: None, icon: None, custom_element: Some(Arc::new(custom)) }
    }
}

/// Represents the collapsible panel content.
#[derive(Clone)]
pub struct AccordionContent {
    pub(crate) element: Option<Arc<dyn Fn() -> AnyElement + Send + Sync>>,
}

impl AccordionContent {
    pub fn new(element: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        Self { element: Some(Arc::new(move || element.clone().into_any_element())) }
    }

    pub fn custom(custom: impl Fn() -> AnyElement + Send + Sync + 'static) -> Self {
        Self { element: Some(Arc::new(custom)) }
    }
}

/// Represents an item configuration in the accordion.
#[derive(Clone)]
pub struct AccordionItem {
    pub(crate) id: SharedString,
    pub(crate) trigger: AccordionTrigger,
    pub(crate) content: AccordionContent,
    pub(crate) enabled: bool,
    pub(crate) initially_expanded: bool,
}

impl AccordionItem {
    pub fn new(id: impl Into<SharedString>, trigger: AccordionTrigger, content: AccordionContent) -> Self {
        Self { id: id.into(), trigger, content, enabled: true, initially_expanded: false }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }
}

#[derive(Clone)]
pub struct AccordionModel {
    pub(crate) id: SharedString,
    pub(crate) items: Vec<AccordionItem>,
    pub(crate) selection_mode: AccordionSelectionMode,
    pub(crate) collapsible: bool,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn AccordionTemplate>,
}

pub struct AccordionItemRenderModel<'a> {
    pub id: &'a SharedString,
    pub trigger: &'a AccordionTrigger,
    pub content: &'a AccordionContent,
    pub expanded: bool,
    pub enabled: bool,
    pub state: crate::controls::state::CompositeItemState,
}

pub struct AccordionRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<AccordionItemRenderModel<'a>>,
    pub selection_mode: AccordionSelectionMode,
    pub collapsible: bool,
    pub enabled: bool,
    pub focus: crate::controls::state::ControlFocusState,
}

pub struct AccordionBuilder {
    pub(crate) model: AccordionModel,
}

impl AccordionBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: AccordionModel {
                id: id.into(),
                items: Vec::new(),
                selection_mode: AccordionSelectionMode::Single,
                collapsible: true,
                enabled: true,
                template: default_accordion_template(),
            },
        }
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    pub fn single(self) -> Self {
        self.mode(AccordionSelectionMode::Single)
    }

    pub fn multiple(self) -> Self {
        self.mode(AccordionSelectionMode::Multiple)
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.model.collapsible = collapsible;
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<AccordionControl> {
        cx.new(|cx| AccordionControl::from_builder(self, cx))
    }
}
