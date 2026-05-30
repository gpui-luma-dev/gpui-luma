use gpui::{AnyElement, Context, FontWeight, MouseButton, Window, div, prelude::*, px};

use super::control::ListViewControl;
use super::layout::DEFAULT_PAGE_SIZE_OPTIONS;
use super::model::ListViewPagingContext;

pub fn render_default_paging_toolbar<T>(
    context: &ListViewPagingContext<'_>,
    page_size_options: &[usize],
    _window: &mut Window,
    cx: &mut Context<ListViewControl<T>>,
) -> AnyElement
where
    T: 'static,
{
    let appearance = &context.appearance;
    let page_size_options = if page_size_options.is_empty() {
        DEFAULT_PAGE_SIZE_OPTIONS.as_slice()
    } else {
        page_size_options
    };

    let selection_summary = match context.selected_count {
        0 => "0 row(s) selected.".to_string(),
        1 => "1 row(s) selected.".to_string(),
        count => format!("{count} row(s) selected."),
    };

    let page_indicator = format!("Page {} of {}", context.current_page + 1, context.page_count.max(1));

    let mut page_size_controls = div().flex().items_center().gap(px(4.0));
    for option in page_size_options.iter().copied() {
        let is_active = option == context.page_size;
        page_size_controls = page_size_controls.child(
            div()
                .id(format!("{}-page-size-{}", context.id, option))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(appearance.radius.min(6.0)))
                .when(is_active, |slot| slot.bg(appearance.header_background).text_color(appearance.header_label_color))
                .when(!is_active, |slot| slot.text_color(appearance.header_label_color).opacity(0.72))
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .cursor_pointer()
                .child(format!("{option}"))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_page_size(option, cx);
                })),
        );
    }

    div()
        .w_full()
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .px(px(appearance.padding_x.max(12.0)))
        .py(px(appearance.padding_y))
        .bg(appearance.header_background)
        .border_t_1()
        .border_color(appearance.border)
        .text_color(appearance.header_label_color)
        .text_size(px(11.0))
        .line_height(px(14.0))
        .child(div().flex_1().min_w(px(0.0)).truncate().child(selection_summary))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().child("Rows per page"))
                .child(page_size_controls),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(page_indicator)
                .child(render_nav_button(
                    context,
                    format!("{}-page-first", context.id),
                    "<<",
                    |this, cx| this.first_page(cx),
                    context.current_page == 0,
                    cx,
                ))
                .child(render_nav_button(
                    context,
                    format!("{}-page-prev", context.id),
                    "<",
                    |this, cx| this.prev_page(cx),
                    context.current_page == 0,
                    cx,
                ))
                .child(render_nav_button(
                    context,
                    format!("{}-page-next", context.id),
                    ">",
                    |this, cx| this.next_page(cx),
                    context.current_page + 1 >= context.page_count.max(1),
                    cx,
                ))
                .child(render_nav_button(
                    context,
                    format!("{}-page-last", context.id),
                    ">>",
                    |this, cx| this.last_page(cx),
                    context.current_page + 1 >= context.page_count.max(1),
                    cx,
                )),
        )
        .into_any_element()
}

fn render_nav_button<T, F>(
    context: &ListViewPagingContext<'_>,
    id: impl Into<gpui::ElementId>,
    label: &'static str,
    action: F,
    disabled: bool,
    cx: &mut Context<ListViewControl<T>>,
) -> AnyElement
where
    T: 'static,
    F: Fn(&mut ListViewControl<T>, &mut Context<ListViewControl<T>>) + 'static,
{
    div()
        .id(id)
        .px(px(8.0))
        .py(px(4.0))
        .rounded(px(context.appearance.radius.min(6.0)))
        .when(!disabled, |slot| slot.cursor_pointer())
        .when(disabled, |slot| slot.opacity(0.4))
        .font_weight(FontWeight::SEMIBOLD)
        .child(label)
        .when(!disabled, |slot| {
            slot.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(cx.listener(move |this, _, _, cx| action(this, cx)))
        })
        .into_any_element()
}
