use std::sync::{Arc, OnceLock};

use gpui::{App, Stateful, Window, div, px, prelude::*};

use super::{CardRenderModel, CardTheme, default_card_theme};

pub trait CardTemplate: Send + Sync {
    fn render(&self, model: &CardRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<gpui::Div>;
}

pub struct ThemedCardTemplate {
    theme: Arc<dyn CardTheme>,
}

impl ThemedCardTemplate {
    pub fn new(theme: Arc<dyn CardTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_card_template() -> Arc<dyn CardTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn CardTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedCardTemplate::new(default_card_theme()))).clone()
}

impl CardTemplate for ThemedCardTemplate {
    fn render(&self, model: &CardRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<gpui::Div> {
        let look = self.theme.resolve(model.size);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(look.section_gap))
            .rounded(px(look.radius))
            .border_1()
            .border_color(look.border)
            .overflow_hidden()
            .bg(look.background)
            .p(px(look.padding))
            .text_color(look.body_color)
            .font_family(look.font_family.clone())
            .text_size(px(look.body.size))
            .line_height(px(look.body.line_height))
            .font_weight(look.body.weight);

        if model.elevated {
            root = root.shadow(look.shadow.clone());
        }

        if model.full_height {
            root = root.h_full().min_h(px(0.0));
        }

        if let Some(header) = model.header {
            root = root.child(header(window, cx));
        } else if model.title.is_some() || model.description.is_some() {
            let mut header = div().w_full().flex().flex_col().gap(px(look.header_gap));

            if let Some(title) = model.title {
                header = header.child(
                    div()
                        .text_size(px(look.title.size))
                        .line_height(px(look.title.line_height))
                        .font_weight(look.title.weight)
                        .text_color(look.title_color)
                        .child(title.clone()),
                );
            }

            if let Some(description) = model.description {
                header = header.child(
                    div()
                        .text_size(px(look.description.size))
                        .line_height(px(look.description.line_height))
                        .font_weight(look.description.weight)
                        .text_color(look.description_color)
                        .child(description.clone()),
                );
            }

            root = root.child(header);
        }

        if !model.body.is_empty() {
            let mut body = div().w_full().flex().flex_col().gap(px(look.body_gap));

            if model.body_fill {
                body = body.flex_1().min_h(px(0.0));
            }

            for child in model.body {
                body = body.child(child(window, cx));
            }

            root = root.child(body);
        }

        if let Some(footer) = model.footer {
            root = root.child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_color(look.body_color)
                    .child(footer(window, cx)),
            );
        }

        root
    }
}
