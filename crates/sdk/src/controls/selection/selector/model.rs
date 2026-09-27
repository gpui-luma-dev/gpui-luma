use std::sync::Arc;

use gpui::{App, AppContext, Bounds, Entity, IntoElement, Pixels, SharedString};

use super::{ControlFocusState, Selector, SelectorTemplate, SelectorVisualState, default_selector_template};
use crate::theme::InteractionState;
use super::template::modified_selector_template;
pub use crate::controls::selector_list::{
    SelectorItem, SelectorItemLike, SelectorItemRenderModel, SelectorItemTemplate, SelectorItemsTemplate, SelectorPath,
    default_selector_items_template, make_selector_item_template, normalize_selector_items,
};
use crate::controls::selector_list::items_template_with_modifier;
use crate::theme::ControlSize;
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::icon::IconSource;

/// Trigger chrome aligned with command button variants (read-only select).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SelectorTriggerStyle {
    #[default]
    Outline,
    Ghost,
}

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
    pub(crate) scroll_interaction: crate::interaction::ScrollInteraction,
    pub(crate) invalid: bool,
    pub(crate) tab_stop: bool,
    pub(crate) size: ControlSize,
    pub(crate) placement: SelectorPlacement,
    pub(crate) trigger_style: SelectorTriggerStyle,
    pub(crate) icons: SelectorIcons,
    pub(crate) without_elevation: bool,
    pub(crate) item_template: Option<SelectorItemTemplate<T>>,
    pub(crate) panel_template: Arc<dyn SelectorItemsTemplate<T>>,
    pub(crate) template: Arc<dyn SelectorTemplate<T>>,
}

#[derive(Clone, Debug)]
pub struct SelectorIcons {
    pub trigger: IconSource,
    pub selected: IconSource,
}

impl Default for SelectorIcons {
    fn default() -> Self {
        Self {
            trigger: lucide_svg_static::Icon::ChevronsUpDown.into(),
            selected: lucide_svg_static::Icon::Check.into(),
        }
    }
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
    pub presence: OverlayPresence,
    /// Retained popup geometry. Custom templates must pair this with the popup wheel handler.
    pub popup_scroll: Option<&'a gpui::ScrollHandle>,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: SelectorPlacement,
    pub active_path: Option<SelectorPath>,
    pub enabled: bool,
    pub size: ControlSize,
    pub trigger_style: SelectorTriggerStyle,
    pub icons: &'a SelectorIcons,
    pub without_elevation: bool,
    pub item_template: Option<&'a SelectorItemTemplate<T>>,
    pub panel_template: Option<&'a dyn SelectorItemsTemplate<T>>,
    pub focus: ControlFocusState,
    pub state: InteractionState,
    pub visual_state: SelectorVisualState,
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
                scroll_interaction: crate::interaction::ScrollInteraction::VIEWPORT,
                invalid: false,
                tab_stop: true,
                size: ControlSize::Md,
                placement: SelectorPlacement::Smart,
                trigger_style: SelectorTriggerStyle::default(),
                icons: SelectorIcons::default(),
                without_elevation: false,
                item_template: None,
                panel_template: default_selector_items_template(),
                template: default_selector_template::<T>(),
            },
            initial_selected_id: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn trigger_style(mut self, style: SelectorTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.model.trigger_style = SelectorTriggerStyle::Ghost;
        self
    }

    pub fn icons(mut self, icons: SelectorIcons) -> Self {
        self.model.icons = icons;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.model.items.push(item);
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

    /// Choose popup wheel eligibility. Defaults to pointer scrolling without acquiring focus.
    pub fn wheel_scroll_policy(mut self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Choose containment or whole-event chaining independently of wheel eligibility.
    pub fn scroll_boundary_policy(mut self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Choose which real focus handles qualify for RequireFocus. No row gains focus implicitly.
    pub fn wheel_focus_scope(mut self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.model.invalid = invalid;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
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

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(gpui::Stateful<gpui::Div>, &SelectorRenderModel<'a, T>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_selector_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SelectorItemsTemplate<T>>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(
                gpui::Stateful<gpui::Div>,
                &crate::controls::selector_list::SelectorItemsRenderModel<'a, T>,
            ) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.panel_template = items_template_with_modifier(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Selector<T>> {
        cx.new(|cx| Selector::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_selector_template::<SelectorItem>();
        let builder = SelectorBuilder::new("selector-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn with_panel_template_modifier_wraps_panel_template() {
        let panel_template = default_selector_items_template::<SelectorItem>();
        let builder = SelectorBuilder::new("selector-test")
            .panel_template(panel_template.clone())
            .with_panel_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.panel_template, &panel_template));
    }
}
