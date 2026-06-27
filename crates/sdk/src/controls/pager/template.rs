use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, ClickEvent, Div, MouseDownEvent, SharedString, Window, div, prelude::*, px};
use lucide_icons::Icon as LucideIcon;

use super::model::{PagerPageItem, PagerRenderModel, PagerStyle};
use super::theme::{PagerLook, PagerTheme, default_pager_theme};
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::command::button::{ButtonRenderModel, ControlPresenter};
use crate::controls::icon::lucide_glyph;
use crate::theme::InteractionState;

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
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let look = self.theme.resolve(model.enabled, model.style);
        let body = match model.style {
            PagerStyle::Minimal => render_minimal_pager(model, &look, &self.theme, &handlers, window, cx),
            PagerStyle::MinimalEdge => render_minimal_edge_pager(model, &look, &self.theme, &handlers, window, cx),
            PagerStyle::Numeric => render_numeric_pager(model, &look, &self.theme, &handlers, window, cx),
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
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(look.group_gap))
        .py(px(look.padding_y))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .child(render_page_indicator(model, look))
        .child(render_nav_group(model, look, theme, handlers, false, window, cx))
}

fn render_minimal_edge_pager(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(look.group_gap))
        .py(px(look.padding_y))
        .text_size(px(look.typography.size))
        .line_height(px(look.typography.line_height))
        .child(render_page_indicator(model, look))
        .child(render_nav_group(model, look, theme, handlers, true, window, cx))
}

fn render_numeric_pager(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let items = numeric_page_items(model.current_page, model.page_count.max(1), model.numeric_slot_count());
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(look.gap))
        .py(px(look.padding_y))
        .child(render_nav_group_leading(model, look, theme, handlers, true, window, cx))
        .child(div().flex().items_center().gap(px(look.gap)).children(items.into_iter().map(|item| match item {
            PagerPageItem::Page(page) => render_page_button(model, look, theme, page, handlers, window, cx),
            PagerPageItem::Gap { target } => render_gap_button(model, look, theme, target, handlers, window, cx),
        })))
        .child(render_nav_group_trailing(model, look, theme, handlers, true, window, cx))
}

pub(crate) fn render_page_indicator(model: &PagerRenderModel<'_>, look: &PagerLook) -> Div {
    div()
        .flex_none()
        .text_color(look.body_text)
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
    look: &PagerLook,
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
                .h(px(look.control_height))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(look.gap))
                .px(px(look.padding_x))
                .rounded(px(look.radius))
                .border_1()
                .border_color(look.border)
                .bg(look.panel_background)
                .text_color(look.body_text)
                .text_size(px(look.typography.size))
                .line_height(px(look.typography.line_height))
                .when(model.enabled, |slot| slot.cursor_pointer())
                .child(format!("{}", model.page_size))
                .child(div().text_color(look.muted_text).child(lucide_glyph(if model.page_size_open {
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
                    .top(px(look.control_height + 4.0))
                    .left(px(0.0))
                    .occlude()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .p(px(4.0))
                    .min_w(px(model.page_size_trigger_width()))
                    .rounded(px(look.radius))
                    .border_1()
                    .border_color(look.border)
                    .bg(look.panel_background)
                    .shadow(look.shadow.clone())
                    .children(model.page_size_options.iter().copied().map(|option| {
                        let selected = option == model.page_size;
                        div()
                            .id(format!("{}-page-size-{option}", model.id))
                            .px(px(look.padding_x))
                            .py(px(4.0))
                            .rounded(px((look.radius - 2.0).max(0.0)))
                            .when(selected, |slot| {
                                slot.bg(look.selected_background).text_color(look.selected_foreground)
                            })
                            .when(!selected, |slot| slot.text_color(look.body_text))
                            .text_size(px(look.typography.size))
                            .line_height(px(look.typography.line_height))
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
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(look.gap))
        .child(render_nav_group_leading(model, look, theme, handlers, include_edges, window, cx))
        .child(render_nav_group_trailing(model, look, theme, handlers, include_edges, window, cx))
}

fn render_nav_group_leading(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let mut group = div().flex().items_center().gap(px(look.gap));
    if include_edges {
        group = group.child(render_nav_button(
            model,
            look,
            theme,
            NavButtonSpec {
                id_suffix: "nav-first-0".to_string(),
                icon: LucideIcon::ChevronsLeft,
                label: model.first_label(),
                label_position: NavLabelPosition::AfterIcon,
                target: 0,
                disabled: model.at_first(),
            },
            handlers,
            window,
            cx,
        ));
    }
    group.child(render_nav_button(
        model,
        look,
        theme,
        NavButtonSpec {
            id_suffix: format!("nav-prev-{}", model.current_page.saturating_sub(1)),
            icon: LucideIcon::ChevronLeft,
            label: model.previous_label(),
            label_position: NavLabelPosition::AfterIcon,
            target: model.current_page.saturating_sub(1),
            disabled: model.at_first(),
        },
        handlers,
        window,
        cx,
    ))
}

fn render_nav_group_trailing(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    handlers: &PagerTemplateHandlers,
    include_edges: bool,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let last_page = model.page_count.saturating_sub(1);
    let mut group = div().flex().items_center().gap(px(look.gap)).child(render_nav_button(
        model,
        look,
        theme,
        NavButtonSpec {
            id_suffix: format!("nav-next-{}", (model.current_page + 1).min(last_page)),
            icon: LucideIcon::ChevronRight,
            label: model.next_label(),
            label_position: NavLabelPosition::BeforeIcon,
            target: (model.current_page + 1).min(last_page),
            disabled: model.at_last(),
        },
        handlers,
        window,
        cx,
    ));
    if include_edges {
        group = group.child(render_nav_button(
            model,
            look,
            theme,
            NavButtonSpec {
                id_suffix: format!("nav-last-{last_page}"),
                icon: LucideIcon::ChevronsRight,
                label: model.last_label(),
                label_position: NavLabelPosition::BeforeIcon,
                target: last_page,
                disabled: model.at_last(),
            },
            handlers,
            window,
            cx,
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
    id_suffix: String,
    icon: LucideIcon,
    label: Option<&'a SharedString>,
    label_position: NavLabelPosition,
    target: usize,
    disabled: bool,
}

struct PagerButtonLayout {
    min_width: f32,
    square: bool,
    inactive: bool,
}

fn render_nav_button(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    spec: NavButtonSpec<'_>,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let has_label = spec.label.is_some();
    let gap = look.gap;
    let icon = spec.icon;
    let label_position = spec.label_position;
    let label = spec.label.cloned();
    let content: ControlPresenter<ButtonRenderModel<()>> = Arc::new(move |_, _| {
        let icon_element = lucide_glyph(icon).into_any_element();
        let label_element = label.as_ref().map(|label| div().child(label.clone()).into_any_element());
        match (label_position, label_element) {
            (NavLabelPosition::BeforeIcon, Some(label_element)) => {
                div().flex().items_center().gap(px(gap)).child(label_element).child(icon_element).into_any_element()
            }
            (NavLabelPosition::AfterIcon, Some(label_element)) => {
                div().flex().items_center().gap(px(gap)).child(icon_element).child(label_element).into_any_element()
            }
            (_, None) => icon_element,
        }
    });

    let click = (model.enabled && !spec.disabled).then(|| {
        let set_page = handlers.set_page.clone();
        let target = spec.target;
        Arc::new(move |event: &ClickEvent, window: &mut Window, cx: &mut App| {
            set_page(target, event, window, cx);
        }) as PagerClickHandler
    });

    render_pager_button(
        model,
        look,
        theme,
        PagerButtonSpec {
            id_suffix: spec.id_suffix,
            role: if has_label {
                ButtonFamilyRole::Text
            } else {
                ButtonFamilyRole::Icon
            },
            content,
            interaction_disabled: spec.disabled,
            layout: PagerButtonLayout { min_width: look.button_min_width, square: !has_label, inactive: false },
        },
        click,
        window,
        cx,
    )
}

fn render_page_button(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    page: usize,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let selected = page == model.current_page.min(model.page_count.saturating_sub(1));
    let label = SharedString::from(format!("{}", page + 1));
    let content: ControlPresenter<ButtonRenderModel<()>> =
        Arc::new(move |_, _| div().child(label.clone()).into_any_element());

    let click = (model.enabled && !selected).then(|| {
        let set_page = handlers.set_page.clone();
        Arc::new(move |event: &ClickEvent, window: &mut Window, cx: &mut App| set_page(page, event, window, cx))
            as PagerClickHandler
    });

    render_pager_button(
        model,
        look,
        theme,
        PagerButtonSpec {
            id_suffix: format!("page-{page}"),
            role: ButtonFamilyRole::Toggle { selected },
            content,
            interaction_disabled: false,
            layout: PagerButtonLayout { min_width: look.button_min_width.max(32.0), square: false, inactive: selected },
        },
        click,
        window,
        cx,
    )
}

fn render_gap_button(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    target: usize,
    handlers: &PagerTemplateHandlers,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let content: ControlPresenter<ButtonRenderModel<()>> =
        Arc::new(move |_, _| lucide_glyph(LucideIcon::Ellipsis).into_any_element());

    let click = model.enabled.then(|| {
        let set_page = handlers.set_page.clone();
        Arc::new(move |event: &ClickEvent, window: &mut Window, cx: &mut App| set_page(target, event, window, cx))
            as PagerClickHandler
    });

    render_pager_button(
        model,
        look,
        theme,
        PagerButtonSpec {
            id_suffix: format!("gap-{target}"),
            role: ButtonFamilyRole::Icon,
            content,
            interaction_disabled: false,
            layout: PagerButtonLayout { min_width: look.button_min_width.max(32.0), square: false, inactive: false },
        },
        click,
        window,
        cx,
    )
}

struct PagerButtonSpec {
    id_suffix: String,
    role: ButtonFamilyRole,
    content: ControlPresenter<ButtonRenderModel<()>>,
    interaction_disabled: bool,
    layout: PagerButtonLayout,
}

fn render_pager_button(
    model: &PagerRenderModel<'_>,
    look: &PagerLook,
    theme: &Arc<dyn PagerTheme>,
    spec: PagerButtonSpec,
    click: Option<PagerClickHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let disabled = !model.enabled || spec.interaction_disabled;
    let id = SharedString::from(format!("{}-{}", model.id, spec.id_suffix));
    let pager_look = look.clone();
    let look_theme = Arc::clone(theme);
    let button_template = theme.button_template();
    let clickable = click.is_some() && !disabled;
    let button_model = ButtonRenderModel {
        id,
        data: (),
        content: spec.content,
        role: spec.role,
        size: crate::controls::button_family::ButtonSize::Sm,
        state: InteractionState { disabled, ..InteractionState::default() },
        round: false,
        radius_override: std::cell::Cell::new(Some(pager_look.radius)),
        elevation: true,
        compact: false,
        look: Some(Arc::new(move |model| look_theme.resolve_button_look(&pager_look, model))),
    };

    let mut button = button_template.render(&button_model, window, cx);
    if let Some(click) = click.filter(|_| !disabled) {
        button = button.on_click(move |event, window, cx| click(event, window, cx));
    }

    let mut slot = div().min_w(px(spec.layout.min_width)).child(button);
    if spec.layout.square {
        slot = slot.w(px(look.button_size));
    }
    if spec.layout.inactive || disabled {
        slot = slot.cursor_default();
    } else if clickable {
        slot = slot.cursor_pointer();
    }

    slot.into_any_element()
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
