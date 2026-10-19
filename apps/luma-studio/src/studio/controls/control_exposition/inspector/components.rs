use std::sync::Arc;

use gpui::{AnyElement, IntoElement, RenderOnce, Window, div, prelude::*, px};
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::control_exposition::template::controls_mono_font;

#[derive(IntoElement)]
pub struct InspectorSection {
    look: Arc<ShadcnLook>,
    rows: Vec<AnyElement>,
}

impl InspectorSection {
    pub fn new(look: &Arc<ShadcnLook>, rows: Vec<AnyElement>) -> Self {
        Self { look: look.clone(), rows }
    }
}

impl RenderOnce for InspectorSection {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let chrome = self.look.chrome();
        div()
            .w_full()
            .min_w(px(0.0))
            .border_1()
            .border_color(chrome.border)
            .rounded(px(6.0))
            .overflow_hidden()
            .children(self.rows)
    }
}

#[derive(IntoElement)]
pub struct InspectorRow {
    look: Arc<ShadcnLook>,
    label: AnyElement,
    value: AnyElement,
    source: String,
    value_width: f32,
    show_separator: bool,
}

impl InspectorRow {
    pub fn new(
        look: &Arc<ShadcnLook>,
        label: AnyElement,
        value: AnyElement,
        source: String,
        value_width: f32,
        show_separator: bool,
    ) -> Self {
        Self { look: look.clone(), label, value, source, value_width, show_separator }
    }
}

impl RenderOnce for InspectorRow {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let chrome = self.look.chrome();
        let caption = &self.look.mode_tokens().typography.text.caption;
        let mono_font = controls_mono_font();

        luma_control_support::inspector::render_inspector_row(
            luma_control_support::inspector::InspectorRowStyle {
                border: chrome.border,
                muted_text: chrome.muted_text,
                caption: *caption,
                mono_font: mono_font.to_string(),
                wrap_source: false,
            },
            self.label,
            self.value,
            self.source,
            self.value_width,
            self.show_separator,
        )
    }
}
