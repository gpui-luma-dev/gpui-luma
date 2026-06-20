mod colors;
mod other;
mod typography;

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{App, Context, Entity, SharedString};
use gpui_luma::controls::accordion::AccordionControl;
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

pub(super) use colors::ColorsPanel;
pub(super) use other::OtherPanel;
pub(super) use typography::render_typography_panel;

pub(super) fn category_item_id(prefix: &str, category: &str) -> String {
    format!("{prefix}-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

pub(super) fn expanded_category_ids<'a>(
    accordion: &Entity<AccordionControl>,
    categories: impl IntoIterator<Item = &'a str>,
    prefix: &str,
    cx: &App,
) -> HashSet<String> {
    let accordion = accordion.read(cx);
    categories
        .into_iter()
        .filter_map(|category| {
            let id = category_item_id(prefix, category);
            accordion.is_expanded(&id.clone().into()).then_some(id)
        })
        .collect()
}

pub(super) fn spawn_compact_textfield<T>(
    look: &Arc<ShadcnLook>,
    id: &str,
    value: impl Into<SharedString>,
    cx: &mut Context<T>,
) -> TextField {
    colors::apply_token_field_style(look.textfield(format!("theme-studio-{id}-field")))
        .value(value)
        .full_width(true)
        .spawn(cx)
}

pub(super) fn spawn_slider<T>(
    look: &Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    cx: &mut Context<T>,
) -> Slider {
    look.slider(format!("theme-studio-{id}-slider")).range(min..max).step(step).value(value).spawn(cx)
}
