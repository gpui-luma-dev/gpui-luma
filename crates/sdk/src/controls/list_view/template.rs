use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use super::model::ListViewRenderModel;
use super::theme::{ListViewTheme, default_list_view_theme};

pub trait ListViewTemplate: Send + Sync {
    fn render(
        &self,
        model: &ListViewRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedListViewTemplate {
    theme: Arc<dyn ListViewTheme>,
}

impl ThemedListViewTemplate {
    pub fn new(theme: Arc<dyn ListViewTheme>) -> Self {
        Self { theme }
    }
}

impl ListViewTemplate for ThemedListViewTemplate {
    fn render(
        &self,
        model: &ListViewRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let appearance = self.theme.resolve_list(model.enabled, model.focus.focused, model.size);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(px(appearance.radius))
            .bg(appearance.background)
            .border_1()
            .border_color(appearance.border);

        if let Some(header) = header {
            root = root.child(
                div()
                    .w_full()
                    .flex_none()
                    .px(px(appearance.padding_x))
                    .pt(px(appearance.padding_y))
                    .pb(px(appearance.padding_y * 0.75))
                    .border_b_1()
                    .border_color(appearance.border)
                    .text_color(appearance.header_label_color)
                    .text_size(px(appearance.header_typography.size))
                    .line_height(px(appearance.header_typography.line_height))
                    .font_weight(appearance.header_typography.weight)
                    .child(header),
            );
        }

        root = root.child(
            div()
                .w_full()
                .flex_1()
                .min_h(px(0.0))
                .overflow_hidden()
                .px(px(appearance.padding_x))
                .py(px(appearance.padding_y))
                .child(div().w_full().h_full().child(body)),
        );

        root
    }
}

pub fn default_list_view_template() -> Arc<dyn ListViewTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ListViewTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedListViewTemplate::new(default_list_view_theme()))).clone()
}

pub fn list_view_template_with_theme(theme: Arc<dyn ListViewTheme>) -> Arc<dyn ListViewTemplate> {
    Arc::new(ThemedListViewTemplate::new(theme))
}
