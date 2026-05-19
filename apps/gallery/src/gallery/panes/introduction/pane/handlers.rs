use gpui::{Context, SharedString, Subscription};
use gpui_luma::controls::choice_group::ChoiceGroupEvent;
use gpui_luma::controls::combobox::ComboBoxEvent;
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::popup_menu::PopupMenuEvent;
use gpui_luma::controls::radio_group::RadioGroupEvent;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma::controls::textfield::TextFieldEvent;

use crate::gallery::control::GalleryApp;

use super::{IntroButton, IntroField, IntroductionPane, WorkspaceDensity};
use crate::gallery::panes::shared::notify_entity;

impl IntroductionPane {
    // ===== Wiring =====

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        self.subscribe_payment_panel(cx, subscriptions);
        self.subscribe_workspace_panel(cx, subscriptions);
        self.subscribe_system_panel(cx, subscriptions);
    }

    fn subscribe_payment_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.payment.submit_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Submit, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.cancel_button, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_button_event(IntroButton::Cancel, event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.payment.name_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Name, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.email_field, |app, _, event: &TextFieldEvent, cx| {
            app.panes.introduction.handle_textfield_event(IntroField::Email, event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.payment.payment_combobox, |app, _, event: &ComboBoxEvent, cx| {
            app.panes.introduction.handle_payment_combobox_event(event, cx);
        }));

        subscriptions.push(cx.subscribe(&self.payment.same_as_shipping_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_same_as_shipping_event(event, cx);
        }));
    }

    fn subscribe_workspace_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.workspace.workspace_popup_menu, |app, _, event: &PopupMenuEvent, cx| {
            app.panes.introduction.handle_popup_menu_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_layout_choice_group,
            |app, _, event: &ChoiceGroupEvent, cx| {
                app.panes.introduction.handle_workspace_layout_event(event, cx);
            },
        ));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_icon_demo_choice_group,
            |app, _, event: &ChoiceGroupEvent, cx| {
                app.panes.introduction.handle_workspace_icon_demo_event(event, cx);
            },
        ));
        subscriptions.push(cx.subscribe(
            &self.workspace.workspace_density_radio_group,
            |app, _, event: &RadioGroupEvent<WorkspaceDensity>, cx| {
                app.panes.introduction.handle_workspace_density_event(event, cx);
            },
        ));
    }

    fn subscribe_system_panel(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.system.terms_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_terms_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.social_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_social_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.referral_checkbox, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_referral_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.two_factor_switch, |app, _, event: &ButtonEvent, cx| {
            app.panes.introduction.handle_two_factor_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.system.budget_slider, |app, _, event: &SliderEvent, cx| {
            app.panes.introduction.handle_budget_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        self.notify_payment_panel(cx);
        self.notify_workspace_panel(cx);
        self.notify_system_panel(cx);
    }

    fn notify_payment_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.payment.submit_button, cx);
        notify_entity(&self.payment.cancel_button, cx);
        notify_entity(&self.payment.name_field, cx);
        notify_entity(&self.payment.email_field, cx);
        notify_entity(&self.payment.payment_combobox, cx);
        notify_entity(&self.payment.same_as_shipping_checkbox, cx);
        notify_entity(&self.payment.payment_method_radio, cx);
    }

    fn notify_workspace_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.workspace.workspace_popup_menu, cx);
        notify_entity(&self.workspace.workspace_layout_choice_group, cx);
        notify_entity(&self.workspace.workspace_icon_demo_choice_group, cx);
        notify_entity(&self.workspace.workspace_density_radio_group, cx);
    }

    fn notify_system_panel(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.system.terms_checkbox, cx);
        notify_entity(&self.system.social_checkbox, cx);
        notify_entity(&self.system.referral_checkbox, cx);
        notify_entity(&self.system.two_factor_switch, cx);
        notify_entity(&self.system.budget_slider, cx);
        notify_entity(&self.system.completion_progress, cx);
    }

    // ===== Event handling =====

    fn handle_button_event(&mut self, button: IntroButton, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match button {
            IntroButton::Submit => {
                self.clicks_submit += 1;
                self.last_event = SharedString::from("Button::Submit");
            }
            IntroButton::Cancel => {
                self.clicks_cancel += 1;
                self.last_event = SharedString::from("Button::Cancel");
            }
        }

        cx.notify();
    }

    fn handle_popup_menu_event(&mut self, event: &PopupMenuEvent, cx: &mut Context<GalleryApp>) {
        match event {
            PopupMenuEvent::Select { label, .. } => {
                self.workspace_action = label.clone();
                self.last_event = SharedString::from("PopupMenu::Select");
            }
        }

        cx.notify();
    }

    fn handle_workspace_layout_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected_ids, selected, .. } => {
                self.workspace_layout = if *selected {
                    label.clone()
                } else {
                    SharedString::from("None")
                };
                self.last_event = SharedString::from("ChoiceGroup::Layout");

                let next_selected_ids = selected_ids.clone();
                self.workspace
                    .workspace_layout_choice_group
                    .update(cx, move |group, cx| group.set_managed_selected_ids(next_selected_ids.clone(), cx));
            }
        }

        cx.notify();
    }

    fn handle_workspace_icon_demo_event(&mut self, event: &ChoiceGroupEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ChoiceGroupEvent::Change { label, selected_ids, selected, .. } => {
                self.workspace_icon_demo = if *selected {
                    label.clone()
                } else {
                    SharedString::from("None")
                };
                self.last_event = SharedString::from("ChoiceGroup::IconDemo");

                let next_selected_ids = selected_ids.clone();
                self.workspace
                    .workspace_icon_demo_choice_group
                    .update(cx, move |group, cx| group.set_managed_selected_ids(next_selected_ids.clone(), cx));
            }
        }

        cx.notify();
    }

    fn handle_workspace_density_event(
        &mut self,
        event: &RadioGroupEvent<WorkspaceDensity>,
        cx: &mut Context<GalleryApp>,
    ) {
        match event {
            RadioGroupEvent::Change { selected_value, .. } => {
                self.workspace_density = selected_value
                    .as_ref()
                    .map_or_else(|| SharedString::from("None"), |density| SharedString::from(density.label()));
                self.last_event = SharedString::from("RadioGroup::Density");
            }
        }

        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_textfield_event(&mut self, field: IntroField, event: &TextFieldEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => {
                let value: SharedString = value.clone().into();
                match field {
                    IntroField::Name => self.name_value = value,
                    IntroField::Email => self.email_value = value,
                }
                self.last_event = SharedString::from("TextField::Change");
                self.recompute_completion(cx);
            }
            TextFieldEvent::Focus => {}
            TextFieldEvent::Blur => {}
        }

        cx.notify();
    }

    fn handle_payment_combobox_event(&mut self, event: &ComboBoxEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ComboBoxEvent::Select | ComboBoxEvent::Complete => {
                self.payment_selection_set = true;
                self.last_event = SharedString::from("ComboBox::Select");
            }
            ComboBoxEvent::Clear => {
                self.payment_selection_set = false;
                self.last_event = SharedString::from("ComboBox::Clear");
            }
            ComboBoxEvent::Change => {
                self.last_event = SharedString::from("ComboBox::Change");
            }
        }

        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_same_as_shipping_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.same_as_shipping = !self.same_as_shipping;
        self.payment.same_as_shipping_checkbox.update(cx, |b, cx| b.set_data(self.same_as_shipping, cx));
        self.last_event = SharedString::from("Checkbox::SameAsShipping");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_terms_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.accepted_terms = !self.accepted_terms;
        self.system.terms_checkbox.update(cx, |b, cx| b.set_data(self.accepted_terms, cx));
        self.last_event = SharedString::from("Checkbox::Terms");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_social_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.social_source = !self.social_source;
        self.system.social_checkbox.update(cx, |b, cx| b.set_data(self.social_source, cx));
        self.last_event = SharedString::from("Checkbox::Social");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_referral_event(&mut self, _event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        self.referral_source = !self.referral_source;
        self.system.referral_checkbox.update(cx, |b, cx| b.set_data(self.referral_source, cx));
        self.last_event = SharedString::from("Checkbox::Referral");
        self.recompute_completion(cx);
        cx.notify();
    }

    fn handle_two_factor_event(&mut self, event: &ButtonEvent, cx: &mut Context<GalleryApp>) {
        match event {
            ButtonEvent::Click => {
                self.system.two_factor_switch.update(cx, |button, cx| {
                    let new_on = !*button.data();
                    button.set_data(new_on, cx);
                });
                self.two_factor_enabled = !self.two_factor_enabled;
                self.last_event = SharedString::from("Switch::TwoFactor");
                self.recompute_completion(cx);
                cx.notify();
            }
        }
    }

    fn handle_budget_event(&mut self, event: &SliderEvent, cx: &mut Context<GalleryApp>) {
        let SliderEvent::Change { value } = event;
        self.budget = *value;
        self.last_event = SharedString::from("Slider::Budget");
        self.recompute_completion(cx);
        cx.notify();
    }

    // ===== Derived state =====

    fn recompute_completion(&mut self, cx: &mut Context<GalleryApp>) {
        let mut score = 0.0f32;
        let mut max_score = 0.0f32;

        max_score += 1.0;
        if !self.name_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if !self.email_value.is_empty() {
            score += 1.0;
        }

        max_score += 1.0;
        if self.payment_selection_set {
            score += 1.0;
        }

        max_score += 1.0;
        if self.accepted_terms {
            score += 1.0;
        }

        max_score += 1.0;
        if self.two_factor_enabled {
            score += 1.0;
        }

        max_score += 1.0;
        if self.social_source || self.referral_source {
            score += 1.0;
        }

        max_score += 1.0;
        if !self.workspace_density.is_empty() && self.workspace_density.as_ref() != "None" {
            score += 1.0;
        }

        max_score += 1.0;
        score += self.budget / 100.0;

        self.completion = if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        };

        let completion = self.completion;
        self.system
            .completion_progress
            .update(cx, move |progress, cx| progress.set_value(completion as f64, cx));
    }
}
