use std::sync::Arc;

use gpui::{App, Div, ScrollWheelEvent, Stateful, Window, div, prelude::*, px};

use gpui_luma::controls::scroll_container::ScrollContainer;
use gpui_luma_look_shadcn::LumaTypographyExt;

use super::model::EventLogViewRenderModel;
use super::theme::EventLogTheme;

pub struct EventLogViewTemplateHandlers {
    pub scroll_wheel: Box<dyn Fn(&ScrollWheelEvent, &mut Window, &mut App) + 'static>,
}

pub trait EventLogViewTemplate: Send + Sync {
    fn theme(&self) -> Arc<dyn EventLogTheme>;

    fn render(
        &self,
        model: &EventLogViewRenderModel<'_>,
        scroll: &ScrollContainer,
        handlers: EventLogViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedEventLogViewTemplate {
    theme: Arc<dyn EventLogTheme>,
}

impl ThemedEventLogViewTemplate {
    pub fn new(theme: Arc<dyn EventLogTheme>) -> Self {
        Self { theme }
    }
}

impl EventLogViewTemplate for ThemedEventLogViewTemplate {
    fn theme(&self) -> Arc<dyn EventLogTheme> {
        Arc::clone(&self.theme)
    }

    fn render(
        &self,
        model: &EventLogViewRenderModel<'_>,
        scroll: &ScrollContainer,
        handlers: EventLogViewTemplateHandlers,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = &model.look;
        let show_placeholder = model.text.is_empty();
        let text_color = if show_placeholder {
            look.placeholder
        } else {
            look.foreground
        };

        let content = div()
            .id(format!("{}-content", model.id))
            .w_full()
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .child(
                div()
                    .w_full()
                    .font_family(look.font_family.clone())
                    .typography_style(look.typography)
                    .text_color(text_color)
                    .child(if show_placeholder {
                        model.placeholder.to_string()
                    } else {
                        model.text.to_owned()
                    }),
            )
            .into_any_element();

        let mut root = div()
            .id(model.id.clone())
            .overflow_hidden()
            .rounded(px(look.radius))
            .border(px(look.border_width))
            .border_color(look.border)
            .bg(look.background)
            .h(px(look.min_height))
            .min_h(px(look.min_height))
            .when(model.full_width, |element| element.w_full());

        root =
            root.child(scroll.render_with_scroll_wheel(content, handlers.scroll_wheel).w_full().h(px(look.min_height)));
        root
    }
}

pub fn default_event_log_template(theme: Arc<dyn EventLogTheme>) -> Arc<dyn EventLogViewTemplate> {
    Arc::new(ThemedEventLogViewTemplate::new(theme))
}
