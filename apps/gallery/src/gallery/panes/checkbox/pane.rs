use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Hsla, IntoElement, Subscription, div, prelude::*, rgb};
use gpui_luma::controls::checkbox::{Checkbox, CheckboxEvent, CheckboxTemplate, ThemedCheckboxTemplate};
use gpui_luma::theme::{CheckboxAppearance, CheckboxTheme, DefaultCheckboxTheme, InteractionState};

use crate::gallery::control::GalleryApp;

use super::super::shared::gallery_pane;

struct GalleryCheckboxTheme {
    base: DefaultCheckboxTheme,
    control_border: Option<Hsla>,
    control_background: Option<Hsla>,
    control_padding: Option<(f32, f32)>,
}

impl GalleryCheckboxTheme {
    fn new() -> Self {
        Self {
            base: DefaultCheckboxTheme::default(),
            control_border: None,
            control_background: None,
            control_padding: None,
        }
    }

    fn control_border(mut self, color: impl Into<Hsla>) -> Self {
        self.control_border = Some(color.into());
        self
    }

    fn control_background(mut self, color: impl Into<Hsla>) -> Self {
        self.control_background = Some(color.into());
        self
    }

    fn control_padding(mut self, x: f32, y: f32) -> Self {
        self.control_padding = Some((x, y));
        self
    }
}

impl CheckboxTheme for GalleryCheckboxTheme {
    fn resolve(&self, checked: bool, state: InteractionState) -> CheckboxAppearance {
        let mut appearance = self.base.resolve(checked, state);

        if !state.disabled {
            if let Some(control_border) = self.control_border {
                appearance.control_border = Some(control_border);
            }

            if let Some(control_background) = self.control_background {
                appearance.control_background = Some(control_background);
            }

            if let Some((x, y)) = self.control_padding {
                appearance.control_padding_x = x;
                appearance.control_padding_y = y;
            }
        }

        appearance
    }
}

#[derive(Clone)]
pub(in crate::gallery) struct CheckboxPane {
    default_checkbox: Entity<Checkbox>,
    border_checkbox: Entity<Checkbox>,
    filled_checkbox: Entity<Checkbox>,
    disabled_checkbox: Entity<Checkbox>,
    default_checked: bool,
    border_checked: bool,
    filled_checked: bool,
}

#[derive(Clone, Copy)]
enum CheckboxPresentation {
    Default,
    Border,
    BorderAndBackground,
}

impl CheckboxPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>) -> Self {
        Self {
            default_checkbox: Checkbox::new("checkbox-default").label("As-is").checked(true).spawn(cx),
            border_checkbox: Checkbox::new("checkbox-border")
                .label("Border")
                .template(border_checkbox_template())
                .spawn(cx),
            filled_checkbox: Checkbox::new("checkbox-border-background")
                .label("Border + fill")
                .checked(true)
                .template(filled_checkbox_template())
                .spawn(cx),
            disabled_checkbox: Checkbox::new("disabled-checkbox")
                .label("Disabled checkbox")
                .checked(true)
                .enabled(false)
                .spawn(cx),
            default_checked: true,
            border_checked: false,
            filled_checked: true,
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.default_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Default, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.border_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::Border, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.filled_checkbox, |app, _, event: &CheckboxEvent, cx| {
            app.panes.checkbox.handle_event(CheckboxPresentation::BorderAndBackground, event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self) -> AnyElement {
        gallery_pane(
            "Checkbox",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(self.default_checkbox.clone())
                        .child(self.border_checkbox.clone())
                        .child(self.filled_checkbox.clone())
                        .child(self.disabled_checkbox.clone()),
                )
                .child(div().text_color(rgb(0x334155)).child(format!(
                    "Checked: as-is={}, border={}, border + fill={}",
                    self.default_checked, self.border_checked, self.filled_checked
                )))
                .into_any_element(),
        )
    }

    fn handle_event(
        &mut self,
        presentation: CheckboxPresentation,
        event: &CheckboxEvent,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            CheckboxEvent::Change { checked } => {
                match presentation {
                    CheckboxPresentation::Default => {
                        self.default_checked = *checked;
                    }
                    CheckboxPresentation::Border => {
                        self.border_checked = *checked;
                    }
                    CheckboxPresentation::BorderAndBackground => {
                        self.filled_checked = *checked;
                    }
                }
                cx.notify();
            }
        }
    }
}

fn border_checkbox_template() -> Arc<dyn CheckboxTemplate> {
    checkbox_template(GalleryCheckboxTheme::new().control_border(rgb(0x2563eb)).control_padding(10.0, 6.0))
}

fn filled_checkbox_template() -> Arc<dyn CheckboxTemplate> {
    checkbox_template(
        GalleryCheckboxTheme::new()
            .control_border(rgb(0xbe185d))
            .control_background(rgb(0xfce7f3))
            .control_padding(10.0, 6.0),
    )
}

fn checkbox_template(theme: impl CheckboxTheme + 'static) -> Arc<dyn CheckboxTemplate> {
    Arc::new(ThemedCheckboxTemplate::new(Arc::new(theme)))
}
