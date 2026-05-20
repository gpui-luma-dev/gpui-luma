mod handlers;

use gpui::{AnyElement, Context, Entity, FontWeight, SharedString, div, prelude::*, px};
use gpui_luma::controls::checkbox::Checkbox;
use gpui_luma::controls::choice_group::ChoiceGroup;
use gpui_luma::controls::combobox::ComboBox;
use gpui_luma::controls::command::button::Button;
use gpui_luma::controls::popup_menu::PopupMenu;
use gpui_luma::controls::progress::Progress;
use gpui_luma::controls::radio_button;
use gpui_luma::controls::radio_group::{RadioGroup, RadioGroupItem};
use gpui_luma::controls::slider::Slider;
use gpui_luma::controls::switch::Switch;
use gpui_luma::controls::textfield::TextField;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::gallery_pane_with_description;
use super::cards::{build_payment_panel, build_system_panel, build_workspace_panel, render_cards_row};

const INTRO_DESCRIPTION: &str = concat!(
    "A control-dense landing page built with real interactive gpui-luma controls. ",
    "Use it as a compositional reference for form-heavy product screens."
);

const INTRO_HEADING: &str = "Foundation Controls for GPUI";
const INTRO_SUBHEADING: &str = "A real, interactive introduction screen that combines command, input, choice, and feedback controls in one composition.";

#[derive(Clone)]
pub(super) struct PaymentPanel {
    pub(super) submit_button: Entity<Button>,
    pub(super) cancel_button: Entity<Button>,
    pub(super) name_field: TextField,
    pub(super) email_field: TextField,
    pub(super) payment_combobox: ComboBox,
    pub(super) same_as_shipping_checkbox: Checkbox,
    pub(super) payment_method_radio: radio_button::RadioButton,
}

#[derive(Clone)]
pub(super) struct WorkspacePanel {
    pub(super) workspace_popup_menu: Entity<PopupMenu>,
    pub(super) workspace_layout_choice_group: ChoiceGroup,
    pub(super) workspace_icon_demo_choice_group: ChoiceGroup,
    pub(super) workspace_density_radio_group: RadioGroup<RadioGroupItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WorkspaceDensity {
    Compact,
    Balanced,
    Comfortable,
}

impl WorkspaceDensity {
    pub(super) fn label(&self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Balanced => "Balanced",
            Self::Comfortable => "Comfortable",
        }
    }

    pub(super) fn from_id(id: &str) -> Option<Self> {
        match id {
            "compact" => Some(Self::Compact),
            "balanced" => Some(Self::Balanced),
            "comfortable" => Some(Self::Comfortable),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(super) struct SystemPanel {
    pub(super) terms_checkbox: Checkbox,
    pub(super) social_checkbox: Checkbox,
    pub(super) referral_checkbox: Checkbox,
    pub(super) two_factor_switch: Switch,
    pub(super) budget_slider: Slider,
    pub(super) completion_progress: Progress,
}

#[derive(Clone)]
pub(in crate::gallery) struct IntroductionPane {
    pub(super) payment: PaymentPanel,
    pub(super) workspace: WorkspacePanel,
    pub(super) system: SystemPanel,

    name_value: SharedString,
    email_value: SharedString,
    payment_selection_set: bool,

    same_as_shipping: bool,
    accepted_terms: bool,
    social_source: bool,
    referral_source: bool,
    two_factor_enabled: bool,

    pub(super) workspace_layout: SharedString,
    pub(super) workspace_density: SharedString,
    pub(super) workspace_icon_demo: SharedString,
    pub(super) workspace_action: SharedString,

    pub(super) budget: f32,
    pub(super) completion: f32,
    clicks_submit: usize,
    clicks_cancel: usize,
    last_event: SharedString,
}

#[derive(Clone, Copy)]
enum IntroButton {
    Submit,
    Cancel,
}

#[derive(Clone, Copy)]
enum IntroField {
    Name,
    Email,
}

impl IntroductionPane {
    // ===== Construction =====

    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        let payment = build_payment_panel(cx, theme);
        let workspace = build_workspace_panel(cx, theme);
        let system = build_system_panel(cx, theme);

        Self {
            payment,
            workspace,
            system,

            name_value: SharedString::default(),
            email_value: SharedString::default(),
            payment_selection_set: false,

            same_as_shipping: true,
            accepted_terms: false,
            social_source: true,
            referral_source: false,
            two_factor_enabled: false,

            workspace_layout: SharedString::from("Grid"),
            workspace_density: SharedString::from("Balanced"),
            workspace_icon_demo: SharedString::from("Left"),
            workspace_action: SharedString::from("None"),

            budget: 40.0,
            completion: 30.0,
            clicks_submit: 0,
            clicks_cancel: 0,
            last_event: SharedString::from("Ready"),
        }
    }

    // ===== Render =====

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_description(
            "Introduction",
            Some(INTRO_DESCRIPTION),
            div()
                .w_full()
                .pt(px(32.0))
                .flex()
                .justify_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(1180.0))
                        .flex()
                        .flex_col()
                        .gap(px(0.0))
                        .child(self.render_intro_header(chrome.title_text, chrome.body_text))
                        .gap(px(16.0))
                        .child(
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap(px(16.0))
                                .child(self.render_panel_row(theme))
                                .child(self.render_status_line(chrome.muted_text)),
                        ),
                )
                .into_any_element(),
            theme,
        )
    }

    fn render_intro_header(&self, title_color: gpui::Hsla, body_color: gpui::Hsla) -> AnyElement {
        div()
            .w_full()
            .max_w(px(860.0))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(32.0))
            .child(
                div()
                    .text_size(px(44.0))
                    .line_height(px(52.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(title_color)
                    .child(INTRO_HEADING),
            )
            .child(div().text_size(px(17.0)).line_height(px(26.0)).text_color(body_color).child(INTRO_SUBHEADING))
            .into_any_element()
    }

    fn render_panel_row(&self, theme: &GalleryThemePack) -> AnyElement {
        render_cards_row(self, theme)
    }

    fn render_status_line(&self, muted_text: gpui::Hsla) -> AnyElement {
        div()
            .pt(px(2.0))
            .text_size(px(11.0))
            .line_height(px(16.0))
            .text_color(muted_text)
            .child(format!(
                "Clicks: submit={}, cancel={} | Last event: {}",
                self.clicks_submit, self.clicks_cancel, self.last_event
            ))
            .into_any_element()
    }
}
