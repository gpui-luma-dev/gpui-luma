use std::sync::Arc;

use gpui::{App, Div, SharedString, Stateful, div, prelude::*, px};

pub use crate::controls::selection_panel::item_template::{
    SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler,
};
use crate::controls::selection_panel::item_template::{
    SelectionPanelItemRowHandlers, SelectionPanelItemRowModel, SelectionPanelItemTemplate,
    render_selection_panel_item_row,
};
use crate::controls::selection_panel::model::SelectionPanelItemLike;
use crate::controls::selection_panel::theme::SelectionPanelLook;
use crate::infra::state::ControlFocusState;
use crate::infra::icon::SelectionStatusIcons;

pub struct SelectionPanelTemplateHandlers {
    pub item_hovers: Vec<SelectionPanelHoverHandler>,
    pub item_mouse_downs: Vec<SelectionPanelMouseDownHandler>,
    pub item_mouse_ups: Vec<SelectionPanelMouseUpHandler>,
    pub item_mouse_up_outs: Vec<SelectionPanelMouseUpHandler>,
    pub item_clicks: Vec<SelectionPanelClickHandler>,
}

pub type SelectionPanelModifier<T> =
    Box<dyn for<'a> Fn(Stateful<Div>, &SelectionPanelRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct SelectionPanelRenderModel<'a, T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub panel_id: &'a SharedString,
    pub control_id: &'a SharedString,
    pub items: &'a [T],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub hovered_visible_index: Option<usize>,
    pub pressed_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub item_template: Option<&'a SelectionPanelItemTemplate<T>>,
    pub look: SelectionPanelLook,
    pub show_selection_marker: bool,
    pub icons: &'a SelectionStatusIcons,
    pub show_panel_chrome: bool,
}

pub trait SelectionPanelTemplate<T>: Send + Sync
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div>;
}

struct ModifiedSelectionPanelTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
{
    base: Arc<dyn SelectionPanelTemplate<T>>,
    modifiers: Vec<SelectionPanelModifier<T>>,
}

impl<T> ModifiedSelectionPanelTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn new(base: Arc<dyn SelectionPanelTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(Stateful<Div>, &SelectionPanelRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SelectionPanelRenderModel<'_, T>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

impl<T> SelectionPanelTemplate<T> for ModifiedSelectionPanelTemplate<T>
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, cx);
        self.apply_modifiers(root, model)
    }
}

pub(super) fn template_with_modifier<T, F>(
    template: Arc<dyn SelectionPanelTemplate<T>>,
    modifier: F,
) -> Arc<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
    F: for<'a> Fn(Stateful<Div>, &SelectionPanelRenderModel<'a, T>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedSelectionPanelTemplate::new(template).with_modifier(modifier))
}

pub struct DefaultSelectionPanelTemplate;

impl<T> SelectionPanelTemplate<T> for DefaultSelectionPanelTemplate
where
    T: SelectionPanelItemLike + 'static,
{
    fn render(
        &self,
        model: &SelectionPanelRenderModel<'_, T>,
        handlers: SelectionPanelTemplateHandlers,
        cx: &mut App,
    ) -> Stateful<Div> {
        let SelectionPanelTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        } = handlers;
        let look = model.look.clone();

        let mut root = div().id(format!("{}-rows", model.panel_id)).relative().flex().flex_col().w_full();

        if model.show_panel_chrome {
            root = root
                .min_w(px(look.min_width))
                .p(px(look.padding))
                .bg(look.background)
                .border_1()
                .border_color(look.border)
                .rounded(px(look.radius))
                .shadow(look.shadow.clone())
                .occlude();
        }

        let mut mouse_downs = item_mouse_downs.into_iter();
        let mut mouse_ups = item_mouse_ups.into_iter();
        let mut mouse_up_outs = item_mouse_up_outs.into_iter();
        let mut clicks = item_clicks.into_iter();

        for (visible_index, (source_index, hover)) in model.visible_indices.iter().copied().zip(item_hovers).enumerate()
        {
            let Some(item) = model.items.get(source_index) else {
                continue;
            };

            let row_enabled = model.enabled && item.is_enabled();
            let selected = model.selected_source_index == Some(source_index);
            let active = model.active_visible_index == Some(visible_index);
            let hovered = model.hovered_visible_index == Some(visible_index);
            let pressed = model.pressed_visible_index == Some(visible_index);

            let row = render_selection_panel_item_row(
                SelectionPanelItemRowModel {
                    panel_id: model.panel_id,
                    control_id: model.control_id,
                    item,
                    source_index,
                    visible_index,
                    selected,
                    active,
                    hovered,
                    pressed,
                    focused: model.focus.focused,
                    focus_visible: model.focus.focus_visible,
                    enabled: row_enabled,
                    sibling_count: model.visible_indices.len(),
                    item_template: model.item_template,
                    look: &look,
                    show_selection_marker: model.show_selection_marker,
                    icons: model.icons.clone(),
                },
                SelectionPanelItemRowHandlers {
                    hover,
                    mouse_down: mouse_downs.next(),
                    mouse_up: mouse_ups.next(),
                    mouse_up_out: mouse_up_outs.next(),
                    click: clicks.next(),
                },
                cx,
            );

            root = root.child(row);
        }

        root
    }
}

pub fn default_selection_panel_template<T>() -> Arc<dyn SelectionPanelTemplate<T>>
where
    T: SelectionPanelItemLike + 'static,
{
    Arc::new(DefaultSelectionPanelTemplate)
}

#[allow(clippy::too_many_arguments)]
pub fn render_selection_panel<T>(
    template: Arc<dyn SelectionPanelTemplate<T>>,
    model: &SelectionPanelRenderModel<'_, T>,
    item_hovers: Vec<SelectionPanelHoverHandler>,
    item_mouse_downs: Vec<SelectionPanelMouseDownHandler>,
    item_mouse_ups: Vec<SelectionPanelMouseUpHandler>,
    item_mouse_up_outs: Vec<SelectionPanelMouseUpHandler>,
    item_clicks: Vec<SelectionPanelClickHandler>,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectionPanelItemLike + 'static,
{
    template.render(
        model,
        SelectionPanelTemplateHandlers {
            item_hovers,
            item_mouse_downs,
            item_mouse_ups,
            item_mouse_up_outs,
            item_clicks,
        },
        cx,
    )
}
