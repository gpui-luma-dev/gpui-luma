use std::sync::Arc;

use gpui::{Context, Render, ScrollWheelEvent, Subscription, Window, px};

use gpui_luma::controls::scroll_container::ScrollContainer;
use gpui_luma::controls::scrollbar::ScrollbarEvent;
use gpui_luma::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale, observe_theme_revision};

use super::model::{EventLogViewBuilder, EventLogViewModel, EventLogViewRenderModel};
use super::template::{EventLogViewTemplate, EventLogViewTemplateHandlers, default_event_log_template};
use super::theme::{EventLogLook, EventLogTheme};

pub struct EventLogView {
    model: EventLogViewModel,
    template: Arc<dyn EventLogViewTemplate>,
    text: String,
    scroll: ScrollContainer,
    scroll_to_end_pending: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventLogView {
    pub fn from_builder(builder: EventLogViewBuilder, cx: &mut Context<Self>) -> Self {
        let theme = Arc::clone(&builder.model.theme);
        let template = default_event_log_template(theme);
        let scroll = ScrollContainer::new(
            format!("{}-scroll", builder.model.id),
            Arc::clone(&builder.model.scrollbar_template),
            cx,
        );
        let scrollbar = scroll.scrollbar();
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
            let ScrollbarEvent::Change { value } = event else {
                return;
            };
            this.scroll.set_vertical_offset(*value, cx);
            this.scroll_to_end_pending = false;
        })];

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();

        Self {
            model: builder.model,
            template,
            text: String::new(),
            scroll,
            scroll_to_end_pending: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn append_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if text.is_empty() {
            return;
        }

        self.text.push_str(text);
        self.scroll_to_end_pending = true;
        cx.notify();
    }

    pub fn append_line(&mut self, line: &str, cx: &mut Context<Self>) {
        if line.is_empty() {
            return;
        }

        self.text.push_str(line);
        if !line.ends_with('\n') {
            self.text.push('\n');
        }
        self.scroll_to_end_pending = true;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        if self.text.is_empty() {
            return;
        }

        self.text.clear();
        self.scroll_to_end_pending = false;
        self.scroll.set_vertical_offset(0.0, cx);
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: Arc<dyn EventLogTheme>, cx: &mut Context<Self>) {
        self.model.theme = Arc::clone(&theme);
        self.template = default_event_log_template(theme);
        cx.notify();
    }

    pub fn set_font_family(&mut self, font_family: impl Into<String>, cx: &mut Context<Self>) {
        self.model.font_family = Some(font_family.into());
        cx.notify();
    }

    fn resolved_look(&self, scale: &StandardBoxScale) -> EventLogLook {
        let mut look = self.model.theme.resolve_look(self.model.rows, scale);
        if let Some(font_family) = &self.model.font_family {
            look.font_family = font_family.clone();
        }
        look
    }

    fn scroll_to_end_if_needed(&mut self, cx: &mut Context<Self>) {
        if !self.scroll_to_end_pending {
            return;
        }

        let max_scroll = self.scroll.max_vertical_offset().as_f32();
        let current = self.scroll.vertical_offset().as_f32();
        if max_scroll <= 0.5 {
            if self.text.is_empty() {
                self.scroll_to_end_pending = false;
            }
            return;
        }

        if (current - max_scroll).abs() <= 0.5 {
            self.scroll_to_end_pending = false;
            return;
        }

        self.scroll.set_vertical_offset(max_scroll, cx);
    }

    fn scroll_wheel_step(&self, window: &Window, cx: &mut Context<Self>) -> f32 {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: gpui_luma::theme::ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(gpui_luma::theme::ControlSize::Md, metrics, scale_factor),
        );
        self.resolved_look(&scale).typography.line_height
    }

    fn handle_scroll_wheel(&mut self, event: &ScrollWheelEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.scroll.max_vertical_offset().as_f32() <= 0.5 {
            return;
        }

        let step = px(self.scroll_wheel_step(window, cx));
        let delta_y = event.delta.pixel_delta(step).y.as_f32();
        if !delta_y.is_finite() || delta_y.abs() <= f32::EPSILON {
            return;
        }

        self.scroll_to_end_pending = false;
        cx.stop_propagation();
    }
}

impl Render for EventLogView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.model.theme.metrics(),
            LayoutCacheKey { size: gpui_luma::theme::ControlSize::Md, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(gpui_luma::theme::ControlSize::Md, metrics, scale_factor),
        );
        let look = self.resolved_look(&scale);

        self.scroll.sync_scrollbar(cx);
        self.scroll_to_end_if_needed(cx);

        let render_model = EventLogViewRenderModel {
            id: &self.model.id,
            text: &self.text,
            placeholder: &self.model.placeholder,
            rows: self.model.rows,
            full_width: self.model.full_width,
            look,
        };

        let handlers = EventLogViewTemplateHandlers { scroll_wheel: Box::new(cx.listener(Self::handle_scroll_wheel)) };

        self.template.render(&render_model, &self.scroll, handlers, window, cx)
    }
}
