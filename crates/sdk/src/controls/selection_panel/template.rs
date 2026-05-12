use std::sync::Arc;

use gpui::{AnyElement, App, ClickEvent, Div, SharedString, Stateful, Window, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

use crate::controls::icon::lucide_icon;
use crate::controls::selection_panel::model::{SelectionPanelItemLike, SelectionPanelPresenterModel};
use crate::controls::selection_panel::theme::SelectionPanelAppearance;
use crate::controls::state::ControlFocusState;

pub type SelectionPanelClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type SelectionPanelHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

pub struct SelectionPanelTemplateHandlers {
    pub item_hovers: Vec<SelectionPanelHoverHandler>,
    pub item_clicks: Vec<SelectionPanelClickHandler>,
}

pub type SelectionPanelPresenterRef<'a, T> =
    &'a dyn for<'b> Fn(&SelectionPanelPresenterModel<'b, T>, &mut App) -> AnyElement;

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
    pub presenter: Option<SelectionPanelPresenterRef<'a, T>>,
    pub appearance: SelectionPanelAppearance,
    pub show_selection_marker: bool,
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
        let SelectionPanelTemplateHandlers { item_hovers, item_clicks } = handlers;
        let appearance = model.appearance.clone();

        let mut root = div()
            .id(format!("{}-rows", model.panel_id))
            .relative()
            .flex()
            .flex_col()
            .min_w(px(appearance.min_width))
            .p(px(appearance.padding))
            .bg(appearance.background)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .shadow(appearance.shadow.clone())
            .occlude();

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
            let color = if row_enabled {
                appearance.foreground
            } else {
                appearance.item_disabled_foreground
            };

            let content = if let Some(presenter) = model.presenter {
                presenter(
                    &SelectionPanelPresenterModel {
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
                    },
                    cx,
                )
            } else {
                div().flex_1().min_w(px(0.0)).truncate().child(item.label().clone()).into_any_element()
            };

            let mut row = div()
                .id(format!("{}-row-{}", model.panel_id, visible_index))
                .flex()
                .items_center()
                .gap(px(appearance.item_gap))
                .min_h(px(appearance.item_height))
                .px(px(appearance.item_padding_x))
                .rounded(px(appearance.item_radius))
                .text_color(color)
                .text_size(px(appearance.item_typography.size))
                .line_height(px(appearance.item_typography.line_height))
                .font_weight(appearance.item_typography.weight)
                .when_some(item.icon(), |row, icon| {
                    if let Some(icon) = icon.lucide() {
                        row.child(lucide_icon(icon, color, appearance.item_icon_size))
                    } else if let Some(path) = icon.svg_path() {
                        row.child(
                            gpui::svg()
                                .external_path(path.clone())
                                .size(px(appearance.item_icon_size))
                                .text_color(color),
                        )
                    } else {
                        row
                    }
                })
                .child(div().flex_1().child(content));

            if model.show_selection_marker && selected {
                row = row.child(lucide_icon(LucideIcon::Check, color, appearance.item_icon_size));
            }

            if row_enabled {
                row = row.cursor_pointer().on_hover(hover).hover({
                    let hover_background = appearance.item_hover_background;
                    move |style| style.bg(hover_background)
                });

                if active {
                    row = row.bg(appearance.item_hover_background);
                }

                if let Some(click) = clicks.next() {
                    row = row.on_click(click);
                }
            } else {
                row = row.opacity(0.56);
            }

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

pub fn render_selection_panel<T>(
    template: Arc<dyn SelectionPanelTemplate<T>>,
    model: &SelectionPanelRenderModel<'_, T>,
    item_hovers: Vec<SelectionPanelHoverHandler>,
    item_clicks: Vec<SelectionPanelClickHandler>,
    cx: &mut App,
) -> Stateful<Div>
where
    T: SelectionPanelItemLike + 'static,
{
    template.render(model, SelectionPanelTemplateHandlers { item_hovers, item_clicks }, cx)
}
