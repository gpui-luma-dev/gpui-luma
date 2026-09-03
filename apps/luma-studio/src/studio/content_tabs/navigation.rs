use std::sync::Arc;

use gpui::{App, Div, Hsla, Stateful, Window, div, prelude::*, px};
use luma::controls::color::style::ElementExt;
use luma::controls::control_group::ControlGroupItemHandlerExt;
use luma::controls::tabs_navigation::{
    TabsNavigationIndicatorMotion, TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers,
    TabsNavigationTheme, render_tabs_navigation_item_button, resolve_tabs_navigation_uniform_item_width,
};
use luma::theme::{ControlSize, InteractionState};
use luma_look_shadcn::ShadcnLook;

pub fn luma_studio_tabs_navigation_template(
    look: Arc<ShadcnLook>,
    tab_size: ControlSize,
) -> Arc<dyn TabsNavigationTemplate> {
    let full_bar_color = look.token_color("border").unwrap_or(look.chrome().border);
    Arc::new(LumaStudioTabsNavigationTemplate { theme: look.tabs_navigation_theme(), full_bar_color, tab_size })
}

struct LumaStudioTabsNavigationTemplate {
    theme: Arc<dyn TabsNavigationTheme>,
    full_bar_color: Hsla,
    tab_size: ControlSize,
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

        let active_item = model.items.iter().find(|item| item.active);
        let active_look =
            active_item.map(|item| self.theme.resolve_item(true, item.state.interaction_state(), self.tab_size));
        if let Some(motion) = model.indicator_motion {
            let look = active_look
                .unwrap_or_else(|| self.theme.resolve_item(true, InteractionState::default(), self.tab_size));
            motion.set_metrics(look.padding_x, look.indicator_height, look.indicator);
        }

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

        if let Some(motion) = model.indicator_motion.cloned() {
            root = root.on_prepaint(move |bounds, _, _| {
                motion.set_list_bounds(bounds);
            });
        }

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

        if let Some(indicator) =
            model.indicator.or_else(|| model.indicator_motion.and_then(TabsNavigationIndicatorMotion::paint))
        {
            root = root.child(
                div()
                    .absolute()
                    .left(px(indicator.left))
                    .bottom(px(0.0))
                    .w(px(indicator.width))
                    .h(px(indicator.height))
                    .rounded(px(indicator.height))
                    .bg(indicator.color),
            );
        }

        root
    }
}
