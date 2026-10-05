//! Live dropdown and selected-item-centered selector examples.
use gpui::{AnyElement, Context, Entity, IntoElement, div, prelude::*, px};
use gpui_luma::controls::selector::{Selector, SelectorItem, SelectorOpeningMode};
use gpui_luma_look_radix::{self as radix, Look};

#[derive(Clone)]
pub(crate) struct SelectorExamples {
    previews: Vec<(&'static str, Entity<Selector>)>,
}

impl SelectorExamples {
    pub(crate) fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let previews = [
            ("Dropdown", SelectorOpeningMode::Dropdown, 6),
            ("Centered", SelectorOpeningMode::Centered, 6),
            ("Centered · long list", SelectorOpeningMode::Centered, 60),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (label, mode, count))| {
            let selector = radix::Selector::new(format!("radix-guide-selector-{index}"))
                .look(look)
                .label("Choose an item")
                .items((0..count).map(|item| {
                    SelectorItem::new(item.to_string()).label(format!("Item {}", item + 1)).enabled(item != 1)
                }))
                .selected_id((count / 2).to_string())
                .opening_mode(mode)
                .spawn(cx);
            (label, selector)
        })
        .collect();
        Self { previews }
    }

    pub(crate) fn render(&self, look: &Look) -> AnyElement {
        let label_color = look.resolve_role(radix::SemanticRole::MutedForeground).hsla();
        div()
            .flex()
            .flex_wrap()
            .gap(px(24.0))
            .children(self.previews.iter().map(|(label, selector)| {
                div()
                    .w(px(220.0))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(div().text_color(label_color).child(*label))
                    .child(selector.clone())
            }))
            .into_any_element()
    }
}
