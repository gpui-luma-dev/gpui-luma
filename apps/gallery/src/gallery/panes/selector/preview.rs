use std::sync::Arc;

use gpui::{AnyElement, App, Context, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::selector::{
    ControlFocusState, SelectorPlacement, SelectorRenderModel, SelectorTemplate, SelectorTemplateHandlers,
};
use gpui_luma::theme::{InteractionState};
use gpui_luma_look_shadcn::ShadcnLook;

use super::pane::selector_items;

#[derive(Clone)]
pub(super) struct SelectorStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn SelectorTemplate>,
}

struct SelectorStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
    focus: ControlFocusState,
}

impl SelectorStatePreview {
    pub(super) fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look: look.clone(), template: look.selector_template() }
    }
}

impl Render for SelectorStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            SelectorStateSample {
                id: "default",
                label: "Standard",
                state: InteractionState::default(),
                focus: ControlFocusState::default(),
            },
            SelectorStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
            },
            SelectorStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            SelectorStateSample {
                id: "active",
                label: "Active",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
                focus: ControlFocusState { focused: true, focus_visible: true },
            },
            SelectorStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
                focus: ControlFocusState::default(),
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(14.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_trigger_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_trigger_sample(
    template: &Arc<dyn SelectorTemplate>,
    sample: SelectorStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("popup-selector-preview-trigger-{}", sample.id));
    let label = SharedString::from("Selector");
    let items = selector_items().into_iter().collect::<Vec<_>>();
    let model = SelectorRenderModel {
        id: &id,
        label: &label,
        selected_index: None,
        items: &items,
        open: false,
        trigger_bounds: None,
        placement: SelectorPlacement::BelowStart,
        active_path: None,
        enabled: !sample.state.disabled,
        item_template: None,
        panel_template: None,
        focus: sample.focus,
        state: sample.state,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, SelectorTemplateHandlers::default(), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
