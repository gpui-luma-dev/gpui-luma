use std::sync::Arc;

use gpui::{App, ClickEvent, Corner, Div, Stateful, Window, anchored, deferred, div, px, prelude::*};
use gpui_luma::controls::context_menu::{ContextMenuRenderModel, ContextMenuTemplate, ContextMenuTemplateHandlers};
use gpui_luma::controls::floating_menu::render_floating_menu;
use gpui_luma::controls::context_menu::{ContextMenuAppearance, ContextMenuTheme};

type ContextMenuClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub(in crate::gallery) fn gallery_context_menu_template(
    theme: Arc<dyn ContextMenuTheme>,
) -> Arc<dyn ContextMenuTemplate> {
    Arc::new(GalleryContextMenuTemplate { theme })
}

struct GalleryContextMenuTemplate {
    theme: Arc<dyn ContextMenuTheme>,
}

impl ContextMenuTemplate for GalleryContextMenuTemplate {
    fn render(
        &self,
        model: &ContextMenuRenderModel<'_>,
        handlers: ContextMenuTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let ContextMenuTemplateHandlers {
            target_bounds,
            target_aux_click,
            target_hover: _,
            target_mouse_down: _,
            target_mouse_up: _,
            target_mouse_up_out: _,
            root_mouse_down_out,
            item_hovers,
            item_clicks,
        } = handlers;
        let appearance = self.theme.resolve(model.state);
        let mut target = render_gallery_target(model, &appearance, target_aux_click);

        if !model.enabled {
            target = target.opacity(0.56);
        }

        let mut root = div()
            .on_children_prepainted(move |bounds, window, cx| {
                if let Some(bounds) = bounds.first() {
                    target_bounds(bounds, window, cx);
                }
            })
            .id(model.id.clone())
            .relative()
            .on_mouse_down_out(root_mouse_down_out)
            .child(target);

        if let Some(position) = model.menu_position {
            let menu = render_floating_menu(
                model.id,
                model.items,
                model.open_submenu,
                model.active_path,
                appearance.floating_menu,
                item_hovers,
                item_clicks,
            );
            let overlay = anchored()
                .snap_to_window_with_margin(px(8.0))
                .anchor(Corner::TopLeft)
                .position(position)
                .child(menu);

            root = root.child(deferred(overlay).with_priority(1));
        }

        root
    }
}

pub(super) fn render_gallery_target(
    model: &ContextMenuRenderModel<'_>,
    appearance: &ContextMenuAppearance,
    target_aux_click: ContextMenuClickHandler,
) -> Stateful<Div> {
    div()
        .id(format!("{}-target", model.id))
        .w(px(180.0))
        .h(px(128.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .text_color(appearance.target_foreground)
        .on_aux_click(target_aux_click)
        .child(model.label.clone())
}
