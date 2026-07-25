use std::sync::Arc;

use gpui::{App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use gpui_luma::controls::control_group::ControlGroupItemHandlerExt;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers, TabsNavigationTheme,
    render_tabs_navigation_item_button, resolve_tabs_navigation_uniform_item_width,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;

pub fn graph_viz_tabs_navigation_template(
    look: Arc<ShadcnLook>,
    tab_size: ControlSize,
) -> Arc<dyn TabsNavigationTemplate> {
    let full_bar_color = look.token_color("border").unwrap_or(look.chrome().border);
    Arc::new(GraphVizTabsNavigationTemplate { theme: look.tabs_navigation_theme(), full_bar_color, tab_size })
}

struct GraphVizTabsNavigationTemplate {
    theme: Arc<dyn TabsNavigationTheme>,
    full_bar_color: Hsla,
    tab_size: ControlSize,
}

impl TabsNavigationTemplate for GraphVizTabsNavigationTemplate {
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

            root = root.child(tab);
        }

        root
    }
}
