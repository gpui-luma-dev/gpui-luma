//! Studio prototype for issue #17. This deliberately keeps the interaction
//! sources visible until the unified SDK control contract is validated.

use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{
    ControlFocusState, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplate, PopupMenuTemplateHandlers,
    PopupMenuTriggerModel, PopupMenuTriggerStyle,
};
use gpui_luma::controls::overlay_presence::OverlayPresence;
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma::{hstack, vstack};
use gpui_luma::controls::split_button::{SplitButton, SplitButtonEvent};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::split_button_inspector_adapter::{split_button_inspector_resolver, SPLIT_BUTTON_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;
use crate::studio::controls::catalog::{catalog_entry, ControlDocEntry};
use crate::studio::style::shared::preview_handlers::{
    input_noop_bounds, input_noop_click, input_noop_hover, input_noop_mouse_down, input_noop_mouse_up,
};
use crate::studio::style::shared::samples::ButtonStateSample;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};
use crate::studio::style::shared::button_matrix::render_icon_button_state_header_cell;
use super::template::controls_mono_font;

struct SplitButtonPair {
    control: Entity<SplitButton>,
    label: &'static str,
}

pub struct SplitButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    pairs: Vec<SplitButtonPair>,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SplitButtonLeftPane>,
    inspector_split: Entity<InspectorSplitShell>,
    theme_inspector: Entity<super::theme_inspector::ThemeInspector>,
    _subscriptions: Vec<Subscription>,
}

impl SplitButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("split-button").expect("split-button catalog entry");
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-split-button-event-log",
                "Activate the action face or open the menu face; prototype events appear below.",
            )
        });
        let variants = [
            ("Primary", look.primary_split_button("controls-split-button-primary")),
            ("Secondary", look.secondary_split_button("controls-split-button-secondary")),
            ("Outline", look.outline_split_button("controls-split-button-outline")),
            ("Ghost", look.ghost_split_button("controls-split-button-ghost")),
        ];
        let mut pairs = Vec::new();
        let mut subscriptions = Vec::new();

        for (label, builder) in variants {
            let control = builder.label(label).items(split_button_items()).spawn(cx);

            let event_log = event_stream.clone();
            let variant_label = label.to_string();
            subscriptions.push(cx.subscribe(&control, move |_, _, event: &SplitButtonEvent, cx| {
                let line = match event {
                    SplitButtonEvent::ActionClick => format!("{variant_label}: ActionClick"),
                    SplitButtonEvent::Select { item_id, label } => {
                        format!("{variant_label}: Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
                    }
                    SplitButtonEvent::OpenChanged { open } => {
                        format!("{variant_label}: OpenChanged {{ open: {open} }}")
                    }
                    SplitButtonEvent::Dismiss => format!("{variant_label}: Dismiss"),
                    _ => format!("{variant_label}: other SplitButtonEvent"),
                };
                event_log.update(cx, |stream, cx| stream.append_line(&line, cx));
            }));

            pairs.push(SplitButtonPair { control, label });
        }
        let content_example = look
            .primary_split_button("controls-split-content-example")
            .label("Label")
            .content(|model: &PopupMenuTriggerModel, _| {
                hstack! {
                    gap=6 align=center;
                    gpui_luma::controls::icon::lucide_glyph(LucideIcon::Pencil),
                    div().child(model.label.clone()),
                }
            })
            .items(split_button_items())
            .spawn(cx);
        let left_pane = cx.new(|_| SplitButtonLeftPane {
            look: look.clone(),
            entry,
            pairs: pairs
                .iter()
                .map(|pair| SplitButtonPair { control: pair.control.clone(), label: pair.label })
                .collect(),
            content_example,
            event_stream: event_stream.clone(),
        });
        let left_pane_for_inspector = left_pane.clone();
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-split-button-pane",
            move || left_pane_for_inspector.clone().into_any_element(),
            &SPLIT_BUTTON_INSPECTOR_SPEC,
            split_button_inspector_resolver(),
        );

        Self {
            look,
            entry,
            pairs,
            event_stream,
            left_pane,
            inspector_split,
            theme_inspector,
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }
    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for pair in &self.pairs {
            pair.control.update(cx, |_, cx| cx.notify());
        }
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }
}

impl Render for SplitButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-split-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

struct SplitButtonLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    pairs: Vec<SplitButtonPair>,
    content_example: Entity<SplitButton>,
    event_stream: Entity<ControlEventStream>,
}

impl SplitButtonLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for pair in &self.pairs {
            pair.control.update(cx, |_, cx| cx.notify());
        }
        self.content_example.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for SplitButtonLeftPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = vstack! {
                gap=18.0;
                div().w_full().flex().flex_wrap().items_center().gap(px(16.0)).children(self.pairs.iter().map(|pair| {
                    div().id(format!("split-button-preview-{}", pair.label)).flex().child(pair.control.clone())
                })),
                self.event_stream.clone(),
            };
            let template_preview = vstack! {
                gap=12.0;
                div().text_sm().text_color(self.look.chrome().muted_text).child("Template Preview"),
                div().w_full().child(render_split_button_template_preview(&self.look, window, cx)),
            }
            .w_full()
            .px(px(16.0))
            .pb(px(16.0));
            let content_example = vstack! {
                gap=8.0;
                div().text_sm().text_color(self.look.chrome().muted_text).child("Icon + Text"),
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(12.0))
                    .child(self.content_example.clone()),
                render_split_button_content_code(&self.look),
            }
            .w_full()
            .px(px(16.0));
            div()
                .id("controls-doc-split-button-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(vstack! {
                    gap=18.0;
                    render_control_exposition_card(
                        &self.look,
                        self.entry,
                        preview.into_any_element(),
                        None,
                        ControlExpositionLayout::BORDERLESS,
                    ),
                    content_example,
                    template_preview,
                })
        })
    }
}

fn render_split_button_content_code(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Custom construction"),
        )
        .child(
            div()
                .w_full()
                .text_size(px(12.0))
                .line_height(px(18.0))
                .font_family(controls_mono_font())
                .text_color(chrome.muted_text)
                .child(
                    "use gpui_luma::controls::icon::lucide_glyph;\nuse gpui_luma::controls::popup_menu::PopupMenuTriggerModel;\nuse gpui_luma::controls::presenter::HasPresenter;\n\nlet open = look\n    .primary_split_button(\"open\")\n    .label(\"Label\")\n    .content(|model: &PopupMenuTriggerModel, _| {\n        hstack! {\n            gap = 6.0;\n            align = center;\n            lucide_glyph(LucideIcon::Pencil),\n            div().child(model.label.clone()),\n        }\n    })\n    .items(items)\n    .spawn(cx);",
                ),
        )
        .into_any_element()
}

fn render_split_button_template_preview(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let samples = [
        ButtonStateSample { id: "default", header: "default", state: InteractionState::default() },
        ButtonStateSample {
            id: "hover",
            header: "hover",
            state: InteractionState { hovered: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "focused",
            header: "focused",
            state: InteractionState { focused: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "pressed",
            header: "pressed",
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        },
        ButtonStateSample {
            id: "disabled",
            header: "disabled",
            state: InteractionState { disabled: true, ..InteractionState::default() },
        },
    ];
    let variants = [
        ("Primary", PopupMenuTriggerStyle::Primary, "High emphasis actions"),
        ("Secondary", PopupMenuTriggerStyle::Secondary, "Secondary actions"),
        ("Outline", PopupMenuTriggerStyle::Outline, "Bordered actions"),
        ("Ghost", PopupMenuTriggerStyle::Ghost, "Quiet utility actions"),
    ];
    let template = look.popup_menu_template();
    let chrome = look.chrome();

    VariantStateTable::new(VariantStateTableStyle::from_chrome(&chrome).state_column_width(140.0))
        .row_group_label("Text")
        .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
        .rows(variants.iter().map(|(label, style, description)| {
            VariantStateTableRow {
                label: SharedString::from(*label),
                description: SharedString::from(*description),
                cells: samples
                    .iter()
                    .map(|sample| render_split_button_template_cell(&template, *style, label, sample, window, cx))
                    .collect(),
            }
        }))
        .build()
}

fn render_split_button_template_cell(
    template: &Arc<dyn PopupMenuTemplate>,
    style: PopupMenuTriggerStyle,
    label: &'static str,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("controls-split-template-{}-{}", label.to_lowercase(), sample.id));
    let label = SharedString::from("Text");
    let items = split_button_items();
    let model = PopupMenuRenderModel {
        id: &id,
        label: &label,
        content: Arc::new(|model, _| {
            div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element()
        }),
        items: &items,
        open: false,
        presence: OverlayPresence::new(false, false),
        trigger_bounds: None,
        placement: PopupMenuPlacement::BelowStart,
        trigger_style: style,
        trigger_size: ControlSize::Md,
        menu_size: ControlSize::Md,
        icon_only: false,
        end_icon: None,
        open_trigger_icon: LucideIcon::ChevronUp,
        close_trigger_icon: LucideIcon::ChevronDown,
        full_width: false,
        without_elevation: false,
        split: true,
        trigger_radius_override: None,
        open_submenu: None,
        active_path: None,
        highlight: None,
        enabled: !sample.state.disabled,
        focus: ControlFocusState::default(),
        state: sample.state,
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, split_button_template_handlers(items.len()), window, cx))
        .into_any_element()
}

fn split_button_template_handlers(item_count: usize) -> PopupMenuTemplateHandlers {
    PopupMenuTemplateHandlers {
        trigger_bounds: Box::new(input_noop_bounds),
        action_click: Box::new(input_noop_click),
        trigger_click: Box::new(input_noop_click),
        trigger_hover: Box::new(input_noop_hover),
        trigger_mouse_down: Box::new(input_noop_mouse_down),
        trigger_mouse_up: Box::new(input_noop_mouse_up),
        trigger_mouse_up_out: Box::new(input_noop_mouse_up),
        root_mouse_down_out: Box::new(input_noop_mouse_down),
        item_hovers: (0..item_count).map(|_| Box::new(input_noop_hover) as _).collect(),
        submenu_hovers: Vec::new(),
        item_clicks: (0..item_count).map(|_| Box::new(input_noop_click) as _).collect(),
    }
}

fn split_button_items() -> [MenuItem; 3] {
    [
        MenuItem::new("save-as").label("Save as").icon(LucideIcon::Save),
        MenuItem::new("duplicate").label("Duplicate").icon(LucideIcon::Copy),
        MenuItem::new("export").label("Export").icon(LucideIcon::Download),
    ]
}
