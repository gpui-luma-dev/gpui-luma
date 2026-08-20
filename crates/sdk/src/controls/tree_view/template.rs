use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Div, FontWeight, Hsla, MouseDownEvent, MouseUpEvent, PathBuilder, Stateful,
    Transformation, Window, canvas, div, point, prelude::*, px, radians, svg,
};
use lucide_svg_static::Icon as LucideIcon;

use super::{FlatTreeNode, TreeViewRenderModel, TreeViewTheme, default_tree_view_theme};
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
            let chevron = div()
                .id(format!("{}-chevron", node.id))
                .size(px(chevron_size))
                .flex()
                .items_center()
                .justify_center()
                .child(render_rotating_chevron(
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
        row = row.child(div().flex_1().truncate().child(node.label.clone()));

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
        .child(crate::controls::icon::lucide_glyph(icon))
        .into_any_element()
}

/// Chevron-right glyph painted with PathBuilder and rotated `0° → 90°` by expand progress.
fn render_rotating_chevron(
    icons: &crate::controls::icon::DisclosureIcons,
    expand_progress: f32,
    color: Hsla,
    size: f32,
) -> AnyElement {
    let icon = if expand_progress >= 0.5 {
        &icons.expanded
    } else {
        &icons.collapsed
    };
    if let crate::controls::icon::IconSource::SvgPath(path) = icon {
        return svg()
            .size(px(size))
            .text_color(color)
            .external_path(path.clone())
            .with_transformation(Transformation::rotate(radians(expand_progress * std::f32::consts::FRAC_PI_2)))
            .into_any_element();
    }
    let angle_degrees = expand_progress * 90.0;
    canvas(move |_, _, _| (), {
        move |bounds, _, window, _| {
            let center = bounds.center();
            let extent = bounds.size.width.min(bounds.size.height).as_f32();
            if extent <= f32::EPSILON {
                return;
            }

            // Lucide-like chevron tip pointing right, centered in the icon box.
            let half = extent * 0.22;
            let mut builder = PathBuilder::stroke(px((extent * 0.12).max(1.0)));
            builder.move_to(point(px(-half), px(-half * 1.35)));
            builder.line_to(point(px(half), px(0.0)));
            builder.line_to(point(px(-half), px(half * 1.35)));
            // PathBuilder::rotate expects degrees.
            builder.rotate(angle_degrees);
            builder.translate(center);

            if let Ok(path) = builder.build() {
                window.paint_path(path, color);
            }
        }
    })
    .size(px(size))
    .into_any_element()
}
