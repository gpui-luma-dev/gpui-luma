use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, IntoElement, SharedString, Stateful, Window};
use lucide_icons::Icon as LucideIcon;

use super::{AccordionControl, AccordionTemplate, default_accordion_template};
use super::template::modified_accordion_template;
use crate::theme::ControlSize;

pub type AccordionElementRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

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
    pub(crate) custom_element: Option<AccordionElementRenderer>,
}

impl AccordionTrigger {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: Some(label.into()), icon: None, custom_element: None }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn custom(custom: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
        Self { label: None, icon: None, custom_element: Some(Arc::new(custom)) }
    }
}

/// Represents the collapsible panel content.
#[derive(Clone)]
pub struct AccordionContent {
    pub(crate) element: Option<AccordionElementRenderer>,
}

impl AccordionContent {
    pub fn new(element: impl IntoElement + Clone + Send + Sync + 'static) -> Self {
        Self { element: Some(Arc::new(move |_, _| element.clone().into_any_element())) }
    }

    pub fn custom(custom: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static) -> Self {
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
    pub(crate) animated: bool,
    pub(crate) size: ControlSize,
    pub(crate) item_dividers: bool,
    /// When set, overrides themed vertical padding inside expanded item panels (both edges).
    pub(crate) content_padding_y: Option<f32>,
    /// When set, overrides top padding inside expanded item panels.
    pub(crate) content_padding_top: Option<f32>,
    /// When set, overrides bottom padding inside expanded item panels.
    pub(crate) content_padding_bottom: Option<f32>,
    /// When set, overrides themed minimum height of item triggers.
    pub(crate) trigger_min_height: Option<f32>,
    /// When set, overrides themed vertical padding on item triggers.
    pub(crate) trigger_padding_y: Option<f32>,
    pub(crate) template: Arc<dyn AccordionTemplate>,
}

pub struct AccordionItemRenderModel<'a> {
    pub id: &'a SharedString,
    pub trigger: &'a AccordionTrigger,
    pub content: &'a AccordionContent,
    pub expanded: bool,
    /// Expand/collapse transition progress in range `0.0`..`1.0`.
    pub progress: f32,
    /// Cached natural content height in pixels (0.0 until measured).
    pub content_height_px: f32,
    pub enabled: bool,
    pub state: crate::controls::state::CompositeItemState,
}

pub struct AccordionRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: Vec<AccordionItemRenderModel<'a>>,
    pub selection_mode: AccordionSelectionMode,
    pub collapsible: bool,
    pub enabled: bool,
    pub size: ControlSize,
    pub item_dividers: bool,
    pub content_padding_y: Option<f32>,
    pub content_padding_top: Option<f32>,
    pub content_padding_bottom: Option<f32>,
    pub trigger_min_height: Option<f32>,
    pub trigger_padding_y: Option<f32>,
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
                animated: true,
                size: ControlSize::Md,
                item_dividers: true,
                content_padding_y: None,
                content_padding_top: None,
                content_padding_bottom: None,
                trigger_min_height: None,
                trigger_padding_y: None,
                template: default_accordion_template(),
            },
        }
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.model.selection_mode = mode;
        self
    }

    /// Enables or disables expand/collapse animation (default `true`).
    pub fn animated(mut self, animated: bool) -> Self {
        self.model.animated = animated;
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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn item_dividers(mut self, item_dividers: bool) -> Self {
        self.model.item_dividers = item_dividers;
        self
    }

    pub fn content_padding_y(mut self, padding_y: f32) -> Self {
        self.model.content_padding_y = Some(padding_y);
        self
    }

    pub fn content_padding_top(mut self, padding_top: f32) -> Self {
        self.model.content_padding_top = Some(padding_top);
        self
    }

    pub fn content_padding_bottom(mut self, padding_bottom: f32) -> Self {
        self.model.content_padding_bottom = Some(padding_bottom);
        self
    }

    pub fn trigger_min_height(mut self, height: f32) -> Self {
        self.model.trigger_min_height = Some(height);
        self
    }

    pub fn trigger_padding_y(mut self, padding_y: f32) -> Self {
        self.model.trigger_padding_y = Some(padding_y);
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &AccordionRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.model.template = modified_accordion_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<AccordionControl> {
        cx.new(|cx| AccordionControl::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::AccordionBuilder;

    #[test]
    fn test_accordion_builder_animated_option() {
        let builder_default = AccordionBuilder::new("accordion");
        assert!(builder_default.model.animated);

        let builder_opt_out = AccordionBuilder::new("accordion").animated(false);
        assert!(!builder_opt_out.model.animated);
    }
}
