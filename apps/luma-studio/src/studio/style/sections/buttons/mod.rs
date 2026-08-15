use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate, default_button_template};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::split_button::SplitButton;
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnLookControlExt};

use crate::studio::style::shared::button_matrix::{
    BUTTON_STYLE_VARIANTS, BUTTON_TABLE_STATE_COLUMN_WIDTH, SIZE_PREVIEW_STYLE, button_look_for_style,
    render_button_size_radius_matrix, render_icon_button_state_header_cell, shadcn_style_id,
};
use crate::studio::style::shared::samples::{ButtonStateSample, ButtonTemplateVariant};
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

pub(crate) fn render_button_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let template: Arc<dyn ButtonTemplate<()>> = default_button_template();
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
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    let transparent_border = gpui::hsla(0.0, 0.0, 0.0, 0.0);

    section_shell_with_width(
        960.0,
        "Buttons",
        "State and variant matrix.",
        chrome.title_text,
        chrome.muted_text,
        transparent_border,
        transparent_border,
        render_button_preview_tabbed_content(
            look.as_ref(),
            &template,
            &samples,
            preview_tabs,
            active_tab,
            transparent_border,
            window,
            cx,
        ),
    )
}

fn render_button_preview_tabbed_content(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    preview_tabs: Entity<TabsNavigation>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if active_tab.as_ref() == "sizes" {
        render_button_size_matrix(look, template, window, cx)
    } else {
        render_button_template_matrix(look, template, samples, window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

pub(crate) fn render_split_button_prototype_section(
    look: Arc<ShadcnLook>,
    preview: Entity<SplitButtonPreview>,
) -> AnyElement {
    let chrome = look.chrome();
    section_shell_with_width(
        960.0,
        "Split Button Prototype",
        "Issue #17 prototype: an action face joined to an adjacent menu trigger.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div().w_full().flex().justify_center().child(preview).into_any_element(),
    )
}

pub(crate) struct SplitButtonPreview {
    look: Arc<ShadcnLook>,
    pairs: Vec<(Entity<SplitButton>, &'static str)>,
}

impl SplitButtonPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let variants = [
            ("Primary", look.primary_split_button("style-split-primary")),
            ("Secondary", look.secondary_split_button("style-split-secondary")),
            ("Outline", look.outline_split_button("style-split-outline")),
            ("Ghost", look.ghost_split_button("style-split-ghost")),
        ];
        let pairs = variants
            .into_iter()
            .map(|(label, builder)| {
                let control = builder
                    .label(label)
                    .items([
                        MenuItem::new("save-as").label("Save as"),
                        MenuItem::new("duplicate").label("Duplicate"),
                        MenuItem::new("export").label("Export"),
                    ])
                    .spawn(cx);
                (control, label)
            })
            .collect();
        Self { look, pairs }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for (control, _) in &self.pairs {
            control.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }
}

impl Render for SplitButtonPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(div().text_sm().text_color(self.look.chrome().muted_text).child("Split Button Prototype"))
            .child(
                div().flex().flex_wrap().items_center().gap(px(16.0)).children(
                    self.pairs.iter().map(|(control, label)| {
                        div().id(format!("style-split-preview-{label}")).child(control.clone())
                    }),
                ),
            )
    }
}

fn render_button_template_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    samples: &[ButtonStateSample],
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let variant = ButtonTemplateVariant::TextButton;
    let chrome = look.chrome();

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome).state_column_width(BUTTON_TABLE_STATE_COLUMN_WIDTH),
    )
    .column_headers(samples.iter().map(|sample| render_icon_button_state_header_cell(sample, chrome.muted_text)))
    .rows(BUTTON_STYLE_VARIANTS.iter().map(|row| {
        VariantStateTableRow {
            label: SharedString::from(row.label),
            description: SharedString::from(row.description),
            cells: samples
                .iter()
                .map(|sample| render_button_state_sample(template, look, row.style, variant, sample, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn render_button_size_matrix(
    look: &ShadcnLook,
    template: &Arc<dyn ButtonTemplate<()>>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_button_size_radius_matrix(look, template, SIZE_PREVIEW_STYLE, false, window, cx)
}

pub(crate) fn render_button_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    look: &ShadcnLook,
    style: ShadcnButtonStyle,
    variant: ButtonTemplateVariant,
    sample: &ButtonStateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-button-preview-{}-{}-{}",
        shadcn_style_id(style),
        variant.id(),
        sample.id
    ));
    let look = button_look_for_style(Arc::new(look.clone()), style);
    let model = ButtonRenderModel {
        id,
        data: (),
        content: variant.content(),
        role: if matches!(variant, ButtonTemplateVariant::IconButton) {
            ButtonFamilyRole::Icon
        } else {
            ButtonFamilyRole::Text
        },
        size: ButtonSize::Md,
        state: sample.state,
        round: variant.round(),
        radius_override: std::cell::Cell::new(None),
        elevation: style != ShadcnButtonStyle::ContentOnly,
        compact: false,
        look: Some(look),
        ..Default::default()
    };

    div()
        .w_full()
        .flex()
        .justify_center()
        .items_center()
        .child(template.render(&model, window, cx))
        .into_any_element()
}
