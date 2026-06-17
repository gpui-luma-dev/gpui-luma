use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, MouseDownEvent, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

use super::model::{PagerPageItem, PagerRenderModel, PagerStyle};
use super::theme::{PagerTheme, default_pager_theme};
use crate::controls::icon::lucide_glyph;

pub type PagerOutsideMouseDownHandler = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + Send + Sync + 'static>;
pub type PagerClickHandler = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync + 'static>;
pub type PagerPageClickHandler = Arc<dyn Fn(usize, &ClickEvent, &mut Window, &mut App) + Send + Sync + 'static>;
pub type PagerPageSizeClickHandler = Arc<dyn Fn(usize, &ClickEvent, &mut Window, &mut App) + Send + Sync + 'static>;

pub struct PagerTemplateHandlers {
    pub outside_mouse_down: PagerOutsideMouseDownHandler,
    pub toggle_page_size: PagerClickHandler,
    pub set_page: PagerPageClickHandler,
    pub set_page_size: PagerPageSizeClickHandler,
}

pub trait PagerTemplate: Send + Sync {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}

pub struct ThemedPagerTemplate {
    theme: Arc<dyn PagerTheme>,
}

impl ThemedPagerTemplate {
    pub fn new(theme: Arc<dyn PagerTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_pager_template() -> Arc<dyn PagerTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn PagerTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedPagerTemplate::new(default_pager_theme()))).clone()
}

impl PagerTemplate for ThemedPagerTemplate {
    fn render(
        &self,
        model: &PagerRenderModel<'_>,
        handlers: PagerTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let appearance = self.theme.resolve(model.enabled, model.style);
        let body = match model.style {
            PagerStyle::Minimal => render_minimal_pager(model, &appearance, &handlers),
            PagerStyle::MinimalEdge => render_minimal_edge_pager(model, &appearance, &handlers),
            PagerStyle::Numeric => render_numeric_pager(model, &appearance, &handlers),
        };

        body.into_any_element()
    }
}

pub fn numeric_page_items(current_page: usize, page_count: usize, slot_count: usize) -> Vec<PagerPageItem> {
    if page_count == 0 {
        return vec![PagerPageItem::Page(0)];
    }

    let slot_count = slot_count.max(5);
    if page_count <= slot_count {
        return (0..page_count).map(PagerPageItem::Page).collect();
    }

    let last = page_count - 1;
    let inner_slots = slot_count - 2;

    if current_page <= inner_slots.saturating_sub(2) {
        let mut items = vec![PagerPageItem::Page(0)];
        let end_run = inner_slots - 1;
        items.extend((1..=end_run).map(PagerPageItem::Page));
        items.push(PagerPageItem::Gap { target: end_run + 1 });
        items.push(PagerPageItem::Page(last));
        return items;
    }

    if current_page >= last.saturating_sub(inner_slots.saturating_sub(2)) {
        let start_run = last - (inner_slots - 1);
        let mut items = vec![PagerPageItem::Page(0), PagerPageItem::Gap { target: start_run - 1 }];
        items.extend((start_run..=last).map(PagerPageItem::Page));
        return items;
    }

    let middle_count = inner_slots - 2;
    let half = middle_count / 2;
    let start = current_page.saturating_sub(half);
    let end = start + middle_count - 1;

    let mut items = vec![PagerPageItem::Page(0), PagerPageItem::Gap { target: start - 1 }];
    items.extend((start..=end).map(PagerPageItem::Page));
    items.push(PagerPageItem::Gap { target: end + 1 });
    items.push(PagerPageItem::Page(last));
    items
}

fn render_minimal_pager(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(appearance.group_gap))
        .py(px(appearance.padding_y))
        .text_size(px(appearance.typography.size))
        .line_height(px(appearance.typography.line_height))
        .child(render_page_indicator(model, appearance))
        .child(render_nav_group(model, appearance, handlers, false))
}

fn render_minimal_edge_pager(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(appearance.group_gap))
        .py(px(appearance.padding_y))
        .text_size(px(appearance.typography.size))
        .line_height(px(appearance.typography.line_height))
        .child(render_page_indicator(model, appearance))
        .child(render_nav_group(model, appearance, handlers, true))
}

fn render_numeric_pager(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
) -> Div {
    let items = numeric_page_items(model.current_page, model.page_count.max(1), model.numeric_slot_count());
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(appearance.gap))
        .py(px(appearance.padding_y))
        .child(render_nav_group_leading(model, appearance, handlers, true))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(appearance.gap))
                .children(items.into_iter().map(|item| match item {
                    PagerPageItem::Page(page) => {
                        render_page_button(model, appearance, page, handlers).into_any_element()
                    }
                    PagerPageItem::Gap { target } => {
                        render_gap_button(model, appearance, target, handlers).into_any_element()
                    }
                })),
        )
        .child(render_nav_group_trailing(model, appearance, handlers, true))
}

pub(crate) fn render_page_indicator(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
) -> Div {
    div()
        .flex_none()
        .text_color(appearance.body_text)
        .font_weight(gpui::FontWeight::MEDIUM)
        .child(model.page_indicator())
}

pub(crate) fn render_info_slot(model: &PagerRenderModel<'_>, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
    if let Some(info_slot) = model.info_slot {
        return Some(info_slot(model, window, cx));
    }

    model
        .info_text
        .map(|text| div().flex_1().min_w(px(0.0)).truncate().child(text.clone()).into_any_element())
}

pub(crate) fn render_page_size_select(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
) -> Div {
    div()
        .relative()
        .flex_none()
        .w(px(model.page_size_trigger_width()))
        .on_mouse_down_out({
            let outside_mouse_down = handlers.outside_mouse_down.clone();
            move |event, window, cx| outside_mouse_down(event, window, cx)
        })
        .child(
            div()
                .id(format!("{}-page-size-trigger", model.id))
                .w_full()
                .h(px(appearance.control_height))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(appearance.gap))
                .px(px(appearance.padding_x))
                .rounded(px(appearance.radius))
                .border_1()
                .border_color(appearance.border)
                .bg(appearance.panel_background)
                .text_color(appearance.body_text)
                .text_size(px(appearance.typography.size))
                .line_height(px(appearance.typography.line_height))
                .when(model.enabled, |slot| slot.cursor_pointer())
                .child(format!("{}", model.page_size))
                .child(div().text_color(appearance.muted_text).child(lucide_glyph(if model.page_size_open {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                })))
                .when(model.enabled, |slot| {
                    slot.on_click({
                        let toggle_page_size = handlers.toggle_page_size.clone();
                        move |event, window, cx| toggle_page_size(event, window, cx)
                    })
                }),
        )
        .when(model.page_size_open, |slot| {
            slot.child(
                div()
                    .absolute()
                    .top(px(appearance.control_height + 4.0))
                    .left(px(0.0))
                    .occlude()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .p(px(4.0))
                    .min_w(px(model.page_size_trigger_width()))
                    .rounded(px(appearance.radius))
                    .border_1()
                    .border_color(appearance.border)
                    .bg(appearance.panel_background)
                    .shadow(appearance.shadow.clone())
                    .children(model.page_size_options.iter().copied().map(|option| {
                        let selected = option == model.page_size;
                        div()
                            .id(format!("{}-page-size-{option}", model.id))
                            .px(px(appearance.padding_x))
                            .py(px(4.0))
                            .rounded(px((appearance.radius - 2.0).max(0.0)))
                            .when(selected, |slot| {
                                slot.bg(appearance.selected_background).text_color(appearance.selected_foreground)
                            })
                            .when(!selected, |slot| slot.text_color(appearance.body_text))
                            .text_size(px(appearance.typography.size))
                            .line_height(px(appearance.typography.line_height))
                            .when(model.enabled, |slot| slot.cursor_pointer())
                            .child(format!("{option}"))
                            .when(model.enabled, |slot| {
                                slot.on_click({
                                    let set_page_size = handlers.set_page_size.clone();
                                    move |event, window, cx| set_page_size(option, event, window, cx)
                                })
                            })
                    })),
            )
        })
}

pub(crate) fn render_nav_group(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(appearance.gap))
        .child(render_nav_group_leading(model, appearance, handlers, include_edges))
        .child(render_nav_group_trailing(model, appearance, handlers, include_edges))
}

fn render_nav_group_leading(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
) -> Div {
    let mut group = div().flex().items_center().gap(px(appearance.gap));
    if include_edges {
        group = group.child(render_nav_button(
            model,
            appearance,
            NavButtonSpec {
                icon: LucideIcon::ChevronsLeft,
                label: model.first_label(),
                label_position: NavLabelPosition::AfterIcon,
                target: 0,
                disabled: model.at_first(),
            },
            handlers,
        ));
    }
    group.child(render_nav_button(
        model,
        appearance,
        NavButtonSpec {
            icon: LucideIcon::ChevronLeft,
            label: model.previous_label(),
            label_position: NavLabelPosition::AfterIcon,
            target: model.current_page.saturating_sub(1),
            disabled: model.at_first(),
        },
        handlers,
    ))
}

fn render_nav_group_trailing(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
) -> Div {
    let last_page = model.page_count.saturating_sub(1);
    let mut group = div().flex().items_center().gap(px(appearance.gap)).child(render_nav_button(
        model,
        appearance,
        NavButtonSpec {
            icon: LucideIcon::ChevronRight,
            label: model.next_label(),
            label_position: NavLabelPosition::BeforeIcon,
            target: (model.current_page + 1).min(last_page),
            disabled: model.at_last(),
        },
        handlers,
    ));
    if include_edges {
        group = group.child(render_nav_button(
            model,
            appearance,
            NavButtonSpec {
                icon: LucideIcon::ChevronsRight,
                label: model.last_label(),
                label_position: NavLabelPosition::BeforeIcon,
                target: last_page,
                disabled: model.at_last(),
            },
            handlers,
        ));
    }
    group
}

#[derive(Clone, Copy)]
enum NavLabelPosition {
    BeforeIcon,
    AfterIcon,
}

struct NavButtonSpec<'a> {
    icon: LucideIcon,
    label: Option<&'a SharedString>,
    label_position: NavLabelPosition,
    target: usize,
    disabled: bool,
}

fn render_nav_button(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    spec: NavButtonSpec<'_>,
    handlers: &PagerTemplateHandlers,
) -> AnyElement {
    let has_label = spec.label.is_some();
    let mut button = button_shell(model, appearance, spec.disabled, false)
        .min_w(px(appearance.button_min_width))
        .h(px(appearance.button_size));

    if has_label {
        button = button.px(px(appearance.padding_x));
    } else {
        button = button.size(px(appearance.button_size));
    }

    let icon_element = lucide_glyph(spec.icon).into_any_element();
    let label_element = spec.label.map(|label| div().child(label.clone()).into_any_element());
    button = button.child(match (spec.label_position, label_element) {
        (NavLabelPosition::BeforeIcon, Some(label_element)) => div()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .child(label_element)
            .child(icon_element)
            .into_any_element(),
        (NavLabelPosition::AfterIcon, Some(label_element)) => div()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .child(icon_element)
            .child(label_element)
            .into_any_element(),
        (_, None) => icon_element,
    });

    if model.enabled && !spec.disabled {
        let set_page = handlers.set_page.clone();
        button
            .id(format!("{}-nav-{:?}-{}", model.id, spec.icon, spec.target))
            .cursor_pointer()
            .on_click(move |event, window, cx| set_page(spec.target, event, window, cx))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn render_page_button(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    page: usize,
    handlers: &PagerTemplateHandlers,
) -> AnyElement {
    let selected = page == model.current_page.min(model.page_count.saturating_sub(1));
    let button = button_shell(model, appearance, false, selected)
        .h(px(appearance.button_size))
        .min_w(px(appearance.button_min_width.max(32.0)))
        .px(px(appearance.padding_x))
        .child(format!("{}", page + 1));

    if model.enabled && !selected {
        let set_page = handlers.set_page.clone();
        button
            .id(format!("{}-page-{page}", model.id))
            .cursor_pointer()
            .on_click(move |event, window, cx| set_page(page, event, window, cx))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn render_gap_button(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    target: usize,
    handlers: &PagerTemplateHandlers,
) -> AnyElement {
    let button = button_shell(model, appearance, false, false)
        .h(px(appearance.button_size))
        .min_w(px(appearance.button_min_width.max(32.0)))
        .px(px(appearance.padding_x))
        .child(lucide_glyph(LucideIcon::Ellipsis));

    if model.enabled {
        let set_page = handlers.set_page.clone();
        button
            .id(format!("{}-gap-{target}", model.id))
            .cursor_pointer()
            .on_click(move |event, window, cx| set_page(target, event, window, cx))
            .into_any_element()
    } else {
        button.into_any_element()
    }
}

fn button_shell(
    model: &PagerRenderModel<'_>,
    appearance: &crate::controls::pager::PagerAppearance,
    disabled: bool,
    selected: bool,
) -> Div {
    div()
        .rounded(px(appearance.radius))
        .border_1()
        .border_color(appearance.border)
        .bg(if selected {
            appearance.selected_background
        } else {
            appearance.panel_background
        })
        .text_color(if selected {
            appearance.selected_foreground
        } else {
            appearance.body_text
        })
        .text_size(px(appearance.typography.size))
        .line_height(px(appearance.typography.line_height))
        .font_weight(if selected {
            gpui::FontWeight::SEMIBOLD
        } else {
            appearance.typography.weight
        })
        .flex()
        .items_center()
        .justify_center()
        .when(model.enabled && !disabled, |slot| slot.cursor_pointer())
        .when(!model.enabled || disabled, |slot| slot.opacity(appearance.disabled_opacity))
}

#[cfg(test)]
mod tests {
    use super::{PagerPageItem, numeric_page_items};

    #[test]
    fn numeric_items_expand_small_page_sets() {
        let items = numeric_page_items(2, 5, 7);
        assert_eq!(
            items,
            vec![
                PagerPageItem::Page(0),
                PagerPageItem::Page(1),
                PagerPageItem::Page(2),
                PagerPageItem::Page(3),
                PagerPageItem::Page(4),
            ]
        );
    }

    #[test]
    fn numeric_items_insert_gaps_around_middle_window() {
        let items = numeric_page_items(5, 12, 7);
        assert_eq!(
            items,
            vec![
                PagerPageItem::Page(0),
                PagerPageItem::Gap { target: 3 },
                PagerPageItem::Page(4),
                PagerPageItem::Page(5),
                PagerPageItem::Page(6),
                PagerPageItem::Gap { target: 7 },
                PagerPageItem::Page(11),
            ]
        );
    }

    #[test]
    fn numeric_items_fill_single_page_gaps_without_ellipsis() {
        let items = numeric_page_items(3, 8, 7);
        assert_eq!(
            items,
            vec![
                PagerPageItem::Page(0),
                PagerPageItem::Page(1),
                PagerPageItem::Page(2),
                PagerPageItem::Page(3),
                PagerPageItem::Page(4),
                PagerPageItem::Gap { target: 5 },
                PagerPageItem::Page(7),
            ]
        );
    }

    #[test]
    fn numeric_items_use_configured_fixed_slot_count() {
        let items = numeric_page_items(5, 20, 5);
        assert_eq!(
            items,
            vec![
                PagerPageItem::Page(0),
                PagerPageItem::Gap { target: 4 },
                PagerPageItem::Page(5),
                PagerPageItem::Gap { target: 6 },
                PagerPageItem::Page(19),
            ]
        );
    }

    #[test]
    fn numeric_items_keep_fixed_slot_count_near_end() {
        let items = numeric_page_items(11, 12, 7);
        assert_eq!(items.len(), 7);
        assert_eq!(
            items,
            vec![
                PagerPageItem::Page(0),
                PagerPageItem::Gap { target: 6 },
                PagerPageItem::Page(7),
                PagerPageItem::Page(8),
                PagerPageItem::Page(9),
                PagerPageItem::Page(10),
                PagerPageItem::Page(11),
            ]
        );
    }
}
