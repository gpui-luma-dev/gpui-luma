use gpui::{Context, Entity};
use gpui_luma::controls::selector::{Selector, SelectorItem};
use gpui_luma_look_radix::Look;

pub(crate) fn spawn_range<M: 'static>(look: &Look, cx: &mut Context<M>) -> Entity<Selector> {
    gpui_luma_look_radix::Selector::new("visitors-date-range")
        .look(look)
        .items([
            SelectorItem::new("90").label("Last 3 months"),
            SelectorItem::new("30").label("Last 30 days"),
            SelectorItem::new("7").label("Last 7 days"),
        ])
        .selected_id("90")
        .spawn(cx)
}
