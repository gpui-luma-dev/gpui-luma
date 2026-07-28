use std::sync::Arc;

use gpui::{AnyElement, App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use gpui_luma::controls::control_group::ControlGroupItemHandlerExt;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigationItemOverlay, TabsNavigationOverlayPlacement, TabsNavigationOverlayState, TabsNavigationRenderModel,
    TabsNavigationTemplate, TabsNavigationTemplateHandlers, TabsNavigationTheme, render_tabs_navigation_item_button,
    render_tabs_navigation_item_overlay_host, resolve_tabs_navigation_uniform_item_width,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::paint::floating_menu_look;
use gpui_luma_look_shadcn::ShadcnLook;

use super::super::controls::control_catalog_picker::estimate_control_catalog_picker_size;
use super::host::ContentPaneHost;

pub fn luma_studio_tabs_navigation_template(
    look: Arc<ShadcnLook>,
    tab_size: ControlSize,
) -> Arc<dyn TabsNavigationTemplate> {
    luma_studio_tabs_navigation_template_with_controls_dropdown(look, tab_size, None)
}

pub fn luma_studio_tabs_navigation_template_with_controls_dropdown(
    look: Arc<ShadcnLook>,
    tab_size: ControlSize,
    controls_dropdown: Option<ControlsCatalogDropdown>,
) -> Arc<dyn TabsNavigationTemplate> {
    let full_bar_color = look.token_color("border").unwrap_or(look.chrome().border);
    Arc::new(LumaStudioTabsNavigationTemplate {
        look: look.clone(),
        theme: look.tabs_navigation_theme(),
        full_bar_color,
        tab_size,
        controls_dropdown,
    })
}

#[derive(Clone)]
pub struct ControlsCatalogDropdown {
    host: gpui::Entity<ContentPaneHost>,
    overlay_state: TabsNavigationOverlayState,
}

impl ControlsCatalogDropdown {
    pub fn new(host: gpui::Entity<ContentPaneHost>) -> Self {
        Self { host, overlay_state: TabsNavigationOverlayState::new() }
    }
}

struct LumaStudioTabsNavigationTemplate {
    look: Arc<ShadcnLook>,
    theme: Arc<dyn TabsNavigationTheme>,
    full_bar_color: Hsla,
    tab_size: ControlSize,
    controls_dropdown: Option<ControlsCatalogDropdown>,
}

impl TabsNavigationTemplate for LumaStudioTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, self.tab_size);
        let uniform_width =
            resolve_tabs_navigation_uniform_item_width(model, self.theme.as_ref(), self.tab_size, window);

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px(px(24.0))
            .gap(px(list_look.gap))
            .p(px(list_look.padding))
            .rounded(px(list_look.radius))
            .child(div().absolute().left(px(0.0)).right(px(0.0)).bottom(px(0.0)).h(px(1.0)).bg(self.full_bar_color));

        if let Some(background) = list_look.background {
            root = root.bg(background);
        }

        if let Some(border) = list_look.border {
            root = root.border_1().border_color(border);
        }

        for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
            let look = self.theme.resolve_item(item.active, item.state.interaction_state(), self.tab_size);
            let mut tab = render_tabs_navigation_item_button(
                model.id,
                item,
                look,
                self.theme.font_family(),
                self.tab_size,
                window,
                cx,
            )
            .control_group_item_handlers(item_handlers);

            if let Some(width) = uniform_width {
                tab = tab.w(px(width)).flex_none();
            }

            if !item.state.disabled {
                tab = tab.cursor_pointer();
            }

            if item.id.as_ref() == "controls" {
                root = root.child(self.render_controls_tab_wrapper(item, tab, uniform_width, window, cx));
            } else {
                root = root.child(tab);
            }
        }

        root
    }
}

impl LumaStudioTabsNavigationTemplate {
    fn render_controls_tab_wrapper(
        &self,
        item: &gpui_luma::controls::tabs_navigation::TabsNavigationRenderItem<'_>,
        tab: Stateful<Div>,
        uniform_width: Option<f32>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let Some(dropdown) = self.controls_dropdown.clone() else {
            return tab.into_any_element();
        };
        let open = item
            .trailing_accessory
            .and_then(gpui_luma::controls::tabs_navigation::TabsNavigationItemAccessory::is_disclosure_open)
            .unwrap_or(false);

        let overlay = if open {
            let menu_look = floating_menu_look(self.look.mode_tokens().as_ref(), self.look.mode(), ControlSize::Md);
            let content_size = estimate_control_catalog_picker_size(
                &menu_look,
                self.look.font(gpui_luma_look_shadcn::ShadcnFont::Sans),
                window,
            );
            let content = dropdown.host.update(cx, |host, cx| host.render_catalog_picker(cx));
            Some(TabsNavigationItemOverlay {
                content,
                content_size,
                placement: TabsNavigationOverlayPlacement::BelowCenter,
                offset_y: px(menu_look.item_gap),
                window_margin: px(8.0),
            })
        } else {
            None
        };

        render_tabs_navigation_item_overlay_host(
            "luma-studio-controls-tab-dropdown-host",
            tab,
            uniform_width.map(px),
            dropdown.overlay_state,
            overlay,
            {
                let host = dropdown.host.clone();
                move |_, _, cx| {
                    host.update(cx, |host, cx| host.close_catalog_picker(cx));
                }
            },
        )
    }
}
