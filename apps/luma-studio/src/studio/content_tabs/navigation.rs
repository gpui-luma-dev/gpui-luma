use std::sync::Arc;

use gpui::{App, Div, ElementId, Hsla, SharedString, Stateful, TextRun, Window, div, font, prelude::*, px};
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::controls::control_group::ControlGroupItemHandlerExt;
use gpui_luma::controls::icon::lucide_icon;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigationItemLook, TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers,
    TabsNavigationTheme, TabsNavigationWidthMode,
};
use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use super::controls_tab_chrome::ControlsTabChrome;

const CONTROLS_TAB_ID: &str = "controls";
const TAB_CHEVRON_SIZE: f32 = 12.0;
const TAB_CHEVRON_GAP: f32 = 4.0;

pub fn luma_studio_tabs_navigation_template(
    look: Arc<ShadcnLook>,
    tab_size: ControlSize,
    controls_tab_chrome: Option<ControlsTabChrome>,
) -> Arc<dyn TabsNavigationTemplate> {
    let full_bar_color = look.token_color("border").unwrap_or(look.chrome().border);
    Arc::new(LumaStudioTabsNavigationTemplate {
        theme: look.tabs_navigation_theme(),
        full_bar_color,
        tab_size,
        controls_tab_chrome,
    })
}

struct LumaStudioTabsNavigationTemplate {
    theme: Arc<dyn TabsNavigationTheme>,
    full_bar_color: Hsla,
    tab_size: ControlSize,
    controls_tab_chrome: Option<ControlsTabChrome>,
}

struct TabsNavigationItemVisualModel<'a> {
    id: ElementId,
    label: &'a SharedString,
    state: gpui_luma::controls::tabs_navigation::TabsNavigationItemState,
}

impl TabsNavigationTemplate for LumaStudioTabsNavigationTemplate {
    fn render(
        &self,
        model: &TabsNavigationRenderModel<'_>,
        handlers: TabsNavigationTemplateHandlers,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let list_look = self.theme.resolve_list(model.enabled, self.tab_size);
        let uniform_width = resolve_uniform_tab_width(
            model,
            self.theme.as_ref(),
            self.tab_size,
            self.controls_tab_chrome.is_some(),
            window,
        );

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
            let is_controls_tab = item.id.as_ref() == CONTROLS_TAB_ID;
            let mut tab = render_tabs_navigation_item_visual(
                TabsNavigationItemVisualModel {
                    id: ElementId::NamedChild(Arc::new(model.id.clone().into()), format!("tab-{}", item.id).into()),
                    label: item.label,
                    state: item.state,
                },
                look,
                if is_controls_tab {
                    self.controls_tab_chrome.as_ref()
                } else {
                    None
                },
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

fn resolve_uniform_tab_width(
    model: &TabsNavigationRenderModel<'_>,
    theme: &dyn TabsNavigationTheme,
    tab_size: ControlSize,
    controls_has_chevron: bool,
    window: &mut Window,
) -> Option<f32> {
    if model.width_mode != TabsNavigationWidthMode::Uniform {
        return None;
    }

    let font_family = theme.font_family();
    let mut max_width = 0.0_f32;

    for item in &model.items {
        let look = theme.resolve_item(item.active, item.state.interaction_state(), tab_size);
        let run = TextRun {
            len: item.label.len(),
            font: {
                let mut font = font(font_family.clone());
                font.weight = look.label_typography.weight;
                font
            },
            color: look.label_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(item.label.clone(), px(look.label_typography.size), &[run], None);
        let mut width = line.x_for_index(item.label.len()).as_f32() + look.padding_x * 2.0;
        if controls_has_chevron && item.id.as_ref() == CONTROLS_TAB_ID {
            width += TAB_CHEVRON_SIZE + TAB_CHEVRON_GAP;
        }
        max_width = max_width.max(width);
    }

    Some(max_width)
}

fn render_tabs_navigation_item_visual(
    model: TabsNavigationItemVisualModel<'_>,
    look: TabsNavigationItemLook,
    controls_tab_chrome: Option<&ControlsTabChrome>,
) -> Stateful<Div> {
    let label_content = if let Some(chrome) = controls_tab_chrome {
        let picker_open = chrome.is_picker_open();
        div()
            .flex()
            .items_center()
            .gap(px(TAB_CHEVRON_GAP))
            .child(model.label.clone())
            .child(lucide_icon(
                if picker_open {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                },
                look.label_color,
                TAB_CHEVRON_SIZE,
            ))
            .into_any_element()
    } else {
        div().child(model.label.clone()).into_any_element()
    };

    let mut root = div()
        .id(model.id)
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .min_h(px(look.height))
        .px(px(look.padding_x))
        .rounded(px(look.radius))
        .text_color(look.label_color)
        .text_size(px(look.label_typography.size))
        .line_height(px(look.label_typography.line_height))
        .font_weight(look.label_typography.weight)
        .child(label_content);

    if let Some(indicator) = look.indicator {
        root = root.child(
            div()
                .absolute()
                .left(px(look.padding_x))
                .right(px(look.padding_x))
                .bottom(px(0.0))
                .h(px(look.indicator_height))
                .rounded(px(look.indicator_height))
                .bg(indicator),
        );
    }

    if let Some(chrome) = controls_tab_chrome {
        let chrome = chrome.clone();
        root = root.on_prepaint(move |bounds, _, _| {
            chrome.set_tab_bounds(bounds);
        });
    }

    if model.state.disabled {
        root = root.opacity(0.56);
    }

    root
}
