use std::collections::HashSet;
use std::sync::Arc;

use gpui::{App, Context, Entity, IntoElement};
use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};
use gpui_luma_look_shadcn::ShadcnLook;

use super::model::{OTHER_CATEGORIES, TOKEN_CATEGORIES};
use super::panels::{category_token_content, other_category_content};
use super::ThemeSidebar;

impl ThemeSidebar {
    pub(super) fn build_token_accordion(
        sidebar: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("theme-studio-token-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let items = *tokens;
            let sidebar = sidebar.clone();
            let id = category_item_id("token", category);
            let expanded = if expanded_categories.is_empty() {
                *category == "BASE"
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| {
                        category_token_content(sidebar.read(cx), items).into_any_element()
                    }),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    pub(super) fn build_other_accordion(
        sidebar: Entity<Self>,
        look: Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("theme-studio-other-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for category in OTHER_CATEGORIES {
            let sidebar = sidebar.clone();
            let id = category_item_id("other", category);
            let expanded = if expanded_categories.is_empty() {
                true
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |window, cx| {
                        other_category_content(sidebar.read(cx), category, window).into_any_element()
                    }),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    pub(super) fn expanded_token_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(
            &self.token_accordion,
            TOKEN_CATEGORIES.iter().map(|(category, _)| *category),
            "token",
            cx,
        )
    }

    pub(super) fn expanded_other_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(&self.other_accordion, OTHER_CATEGORIES.iter().copied(), "other", cx)
    }
}

fn category_item_id(prefix: &str, category: &str) -> String {
    format!("{prefix}-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

fn expanded_category_ids<'a>(
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
