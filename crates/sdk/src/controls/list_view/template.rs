use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Div, Stateful, Window, div, px, prelude::*};

use super::model::ListViewRenderModel;

/// GPUI draws `border_1` inset; match section fills to the inner curve.
pub(crate) const SHELL_BORDER_WIDTH: f32 = 1.0;

pub type ListViewTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ListViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

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

/// Paints list shell from resolved [`ListViewRenderModel`] only (no theme lookup).
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultListViewShellTemplate;

impl DefaultListViewShellTemplate {
    pub fn paint_shell(model: &ListViewRenderModel<'_>, header: Option<AnyElement>, body: AnyElement) -> Stateful<Div> {
        let list = &model.look;
        let inner_radius = list.inner_radius(SHELL_BORDER_WIDTH);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(px(list.radius))
            .border_1()
            .border_color(list.border);

        if let Some(height) = model.shell_height {
            root = root.h(px(height));
        } else {
            root = root.h_full();
        }

        let mut column = div().w_full().flex_1().min_h(px(0.0)).flex().flex_col();
        let has_header = header.is_some();

        if let Some(header) = header {
            column = column.child(
                div()
                    .w_full()
                    .flex_none()
                    .rounded_tl(px(inner_radius))
                    .rounded_tr(px(inner_radius))
                    .bg(list.header_background)
                    .px(px(list.padding_x))
                    .pt(px(list.padding_y))
                    .pb(px(list.padding_y * 0.75))
                    .border_b_1()
                    .border_color(list.border)
                    .text_color(list.header_label_color)
                    .text_size(px(list.header_typography.size))
                    .line_height(px(list.header_typography.line_height))
                    .font_weight(list.header_typography.weight)
                    .child(header),
            );
        }

        let mut body_slot = if let Some(body_rows_height) = model.body_rows_height {
            div().w_full().flex_none().h(px(body_rows_height)).overflow_hidden().bg(list.background)
        } else {
            div().w_full().flex_1().min_h(px(0.0)).overflow_hidden().bg(list.background)
        };
        if has_header {
            body_slot = body_slot.rounded_bl(px(inner_radius)).rounded_br(px(inner_radius));
        } else {
            body_slot = body_slot.rounded(px(inner_radius));
        }

        column = column.child(body_slot.child(div().w_full().h_full().child(body)));

        root.child(column)
    }

    pub fn shell_height_for_model(model: &ListViewRenderModel<'_>) -> Option<f32> {
        model.shell_height
    }
}

impl ListViewTemplate for DefaultListViewShellTemplate {
    fn render(
        &self,
        model: &ListViewRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        Self::paint_shell(model, header, body)
    }
}

struct ModifiedListViewTemplate {
    base: Arc<dyn ListViewTemplate>,
    modifiers: Vec<ListViewTemplateModifier>,
}

impl ModifiedListViewTemplate {
    fn new(base: Arc<dyn ListViewTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: ListViewTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &ListViewRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = modifier(root, model);
        }
        root
    }
}

impl ListViewTemplate for ModifiedListViewTemplate {
    fn render(
        &self,
        model: &ListViewRenderModel<'_>,
        header: Option<AnyElement>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, header, body, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub fn default_list_view_template() -> Arc<dyn ListViewTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ListViewTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(DefaultListViewShellTemplate)).clone()
}

/// Shell template only; row/list colors come from [`ListViewTheme`] on the control builder.
pub fn list_view_template_with_theme(
    _theme: Arc<dyn crate::controls::list_view::ListViewTheme>,
) -> Arc<dyn ListViewTemplate> {
    default_list_view_template()
}

pub fn list_view_template_with_modifier<F>(
    template: Arc<dyn ListViewTemplate>,
    modifier: F,
) -> Arc<dyn ListViewTemplate>
where
    F: Fn(Stateful<Div>, &ListViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    ModifiedListViewTemplate::new(template).with_modifier(Box::new(modifier)).into()
}

impl ModifiedListViewTemplate {
    fn into(self) -> Arc<dyn ListViewTemplate> {
        Arc::new(self)
    }
}
