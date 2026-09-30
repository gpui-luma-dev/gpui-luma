use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, FontWeight, MouseDownEvent, MouseUpEvent, Stateful, Window, SharedString, div,
    prelude::*, px,
};
use lucide_svg_static::Icon as LucideIcon;

use super::{FlatTreeNode, TreeViewRenderModel, TreeViewTheme, default_tree_view_theme};
use crate::infra::icon::render_disclosure_icon;
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt};

pub type TreeViewHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;
pub type TreeViewMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TreeViewMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;
pub type TreeViewClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub struct TreeViewTemplateHandlers {
    pub hover: TreeViewHoverHandler,
    pub mouse_down: TreeViewMouseDownHandler,
    pub mouse_up: TreeViewMouseUpHandler,
    pub click: TreeViewClickHandler,
    /// Independent disclosure gesture; the row shell prevents row selection/drag.
    pub disclosure: TreeViewClickHandler,
}

pub type TreeViewTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &TreeViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub trait TreeViewTemplate<T>: Send + Sync
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;

    /// Content-only customization. Legacy full templates retain their renderer
    /// unless they opt into this seam; the built-in and Shadcn templates support it.
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        _content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_node(node, handlers, window, cx)
    }

    /// Stable, noninteractive preview; content callbacks are not rebuilt during drag.
    fn render_drag_preview(
        &self,
        label: SharedString,
        _size: crate::theme::ControlSize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        div().px_3().py_1().child(label).into_any_element()
    }

    /// Color used by the SDK's keyed drop markers and preview outline.
    fn drop_color(&self, _size: crate::theme::ControlSize) -> gpui::Hsla {
        gpui::rgb(0x64748b).into()
    }
}

pub struct ThemedTreeViewTemplate {
    theme: Arc<dyn TreeViewTheme>,
}

impl ThemedTreeViewTemplate {
    pub fn new(theme: Arc<dyn TreeViewTheme>) -> Self {
        Self { theme }
    }
}

struct ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    base: Arc<dyn TreeViewTemplate<T>>,
    modifiers: Vec<TreeViewTemplateModifier>,
}

impl<T> ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    fn new(base: Arc<dyn TreeViewTemplate<T>>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: TreeViewTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &TreeViewRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub fn default_tree_view_template<T>() -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
{
    Arc::new(ThemedTreeViewTemplate::new(default_tree_view_theme()))
}

pub(super) fn modified_tree_view_template<T, F>(
    template: Arc<dyn TreeViewTemplate<T>>,
    modifier: F,
) -> Arc<dyn TreeViewTemplate<T>>
where
    T: Send + Sync + 'static,
    F: Fn(Stateful<Div>, &TreeViewRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedTreeViewTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl<T> TreeViewTemplate<T> for ModifiedTreeViewTemplate<T>
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, body, window, cx);
        self.apply_modifiers(root, model)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.base.render_node(node, handlers, window, cx)
    }
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.base.render_node_with_content(node, handlers, content, window, cx)
    }
    fn render_drag_preview(
        &self,
        label: SharedString,
        size: crate::theme::ControlSize,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.base.render_drag_preview(label, size, window, cx)
    }
    fn drop_color(&self, size: crate::theme::ControlSize) -> gpui::Hsla {
        self.base.drop_color(size)
    }
}

impl<T> TreeViewTemplate<T> for ThemedTreeViewTemplate
where
    T: Send + Sync + 'static,
{
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        div().id(model.id.clone()).size_full().overflow_hidden().child(body)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row(node, handlers, None, window, cx)
    }
    fn render_node_with_content(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: AnyElement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.render_row(node, handlers, Some(content), window, cx)
    }
    fn render_drag_preview(
        &self,
        label: SharedString,
        size: crate::theme::ControlSize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> AnyElement {
        let palette =
            self.theme
                .resolve_row(crate::theme::InteractionState { hovered: true, ..Default::default() }, true, size);
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(palette.chevron_color)
            .when_some(palette.background, |preview, color| preview.bg(color))
            .text_color(palette.foreground)
            .font_family(palette.font_family)
            .text_size(px(palette.typography.size))
            .shadow_sm()
            .child(label)
            .into_any_element()
    }
    fn drop_color(&self, size: crate::theme::ControlSize) -> gpui::Hsla {
        self.theme.resolve_row(Default::default(), true, size).chevron_color
    }
}

impl ThemedTreeViewTemplate {
    fn render_row<T: Send + Sync + 'static>(
        &self,
        node: &FlatTreeNode<'_, T>,
        handlers: TreeViewTemplateHandlers,
        content: Option<AnyElement>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: node.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| super::theme::TreeViewScale::compute(node.size, metrics, scale_factor),
        );

        let palette = self.theme.resolve_row(node.state.interaction_state(), node.state.selected, node.size);
        let icon_size = scale.icon_size;
        let chevron_size = scale.chevron_size;
        let left_padding = scale.base_padding_x + (node.depth as f32 * scale.indentation_width);

        let height_factor = node.row_height_factor.clamp(0.0, 1.0);
        let row_height = scale.row_height * height_factor;

        let mut row = div()
            .id(format!("{}-row", node.id))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(row_height))
            .overflow_hidden()
            .opacity(height_factor.max(0.0))
            .pl(px(left_padding))
            .pr(px(scale.base_padding_x))
            .rounded(px(scale.radius))
            .text_color(palette.foreground)
            .text_size(px(palette.typography.size))
            .line_height(px(palette.typography.line_height))
            .font_family(palette.font_family.clone())
            .font_weight(palette.typography.weight)
            .cursor_pointer();

        if let Some(background) = palette.background {
            row = row.bg(background);
        }

        if node.enabled {
            row = row
                .on_hover(handlers.hover)
                .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
                .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
                .on_click(handlers.click);
        } else {
            row = row.opacity(0.40 * height_factor.max(0.0));
        }

        if node.has_children {
            let debug_id = format!("tree-disclosure-{}", node.id);
            let chevron = div()
                .id(format!("{}-chevron", node.id))
                .debug_selector(move || debug_id.clone())
                .size(px(chevron_size))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .when(node.enabled, |chevron| chevron.on_click(handlers.disclosure))
                .child(render_disclosure_icon(
                    node.disclosure_icons,
                    node.expand_progress.clamp(0.0, 1.0),
                    palette.chevron_color,
                    chevron_size,
                ));

            row = row.child(chevron);
        } else {
            row = row.child(div().size(px(chevron_size)));
        }

        row = row.child(div().w(px(scale.inner_gap)));

        let display_icon = node.icon.unwrap_or(if node.has_children {
            if node.expanded {
                LucideIcon::FolderOpen
            } else {
                LucideIcon::Folder
            }
        } else {
            LucideIcon::File
        });
        row = row.child(render_icon(display_icon, palette.icon_color, icon_size));
        row = row.child(div().w(px(scale.inner_gap)));
        row = row.child(match content {
            Some(content) => div().flex_1().min_w(px(0.0)).child(content),
            None => div().flex_1().truncate().child(node.label.clone()),
        });

        if let Some(color) = self.theme.row_outline(node.state.interaction_state(), node.size) {
            row = row.child(div().absolute().inset_0().rounded(px(scale.radius)).border_1().border_color(color));
        }

        row.into_any_element()
    }
}

fn render_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(crate::infra::icon::lucide_icon(icon, color, size))
        .into_any_element()
}
