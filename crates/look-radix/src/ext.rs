//! Look-local factory helpers. Apps use these instead of editing the SDK.

use std::sync::Arc;

use gpui::SharedString;
use luma::controls::button::{Button, ButtonBuilder, ButtonTemplate, DefaultButtonTemplate};
use luma::controls::checkbox::{self, CheckboxBuilder, CheckboxData, ThemedCheckboxTemplate};
use luma::controls::overlay_window::{self, OverlayWindowBuilder, ThemedOverlayWindowTemplate};
use luma::controls::popup_menu::{PopupMenu, PopupMenuBuilder, PopupMenuTemplate, ThemedPopupMenuTemplate};
use luma::controls::radio_button::{self, RadioButtonBuilder, RadioButtonData, ThemedRadioButtonTemplate};
use luma::controls::slider::{self, SliderBuilder, SliderTemplate, ThemedSliderTemplate};
use luma::controls::switch::{self, SwitchBuilder, SwitchData, ThemedSwitchTemplate};
use luma::controls::tabs::{Tabs, TabsBuilder, ThemedTabsTemplate};
use luma::controls::textarea::{TextArea, TextAreaBuilder, TextAreaTemplate, ThemedTextAreaTemplate};
use luma::controls::textfield::{self, TextFieldBuilder, TextFieldTemplate, ThemedTextFieldTemplate};
use luma::controls::toggle::{self, ToggleBuilder};

use crate::button::{
    RadixButtonPaint, RadixButtonVariant, button_family_theme_with, classic_button_template,
    classic_button_template_with,
};
use crate::button_layout::RadixRadius;
use crate::checkbox::{RadixCheckboxSize, RadixCheckboxVariant, checkbox_theme_for, checkbox_theme_with};
use crate::look::RadixLook;
use crate::overlay_window::overlay_window_theme;
use crate::popup_menu::{RadixPopupMenuVariant, popup_menu_theme};
use crate::radio::{RadixRadioSize, RadixRadioVariant, radio_theme_for, radio_theme_with};
use crate::slider::{RadixSliderVariant, slider_theme_with};
use crate::switch::{RadixSwitchSize, RadixSwitchVariant, switch_theme_for, switch_theme_with};
use crate::tabs::tabs_theme;
use crate::textarea::textarea_theme_with;
use crate::textfield::{RadixTextFieldVariant, textfield_theme_with};
use crate::toggle::{page_toggle_template, toggle_template};
use crate::tone::RadixTone;

/// Constructs SDK controls pre-bound to [`RadixLook`] variants.
pub trait RadixLookControlExt {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn classic_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn solid_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn soft_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn surface_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    /// Ghost button without a focus ring, for icon-only chrome.
    fn quiet_ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()>;
    /// Bare button template for previews that force state / paint axes.
    fn button_template(&self, variant: RadixButtonVariant, paint: RadixButtonPaint) -> Arc<dyn ButtonTemplate<()>>;
    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder;
    fn textfield_variant(&self, id: impl Into<SharedString>, variant: RadixTextFieldVariant) -> TextFieldBuilder;
    fn textfield_template(&self, variant: RadixTextFieldVariant) -> Arc<dyn TextFieldTemplate>;
    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder;
    fn textarea_variant(&self, id: impl Into<SharedString>, variant: RadixTextFieldVariant) -> TextAreaBuilder;
    fn textarea_template(&self, variant: RadixTextFieldVariant) -> Arc<dyn TextAreaTemplate>;
    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder;
    fn slider_variant(&self, id: impl Into<SharedString>, variant: RadixSliderVariant) -> SliderBuilder;
    fn slider_template(&self, variant: RadixSliderVariant) -> Arc<dyn SliderTemplate>;
    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder;
    fn checkbox_variant(&self, id: impl Into<SharedString>, variant: RadixCheckboxVariant) -> CheckboxBuilder;
    /// Bare template for previews that render forced states instead of live controls.
    fn checkbox_template(&self, variant: RadixCheckboxVariant) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn checkbox_template_with(
        &self,
        variant: RadixCheckboxVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn checkbox_template_for(
        &self,
        variant: RadixCheckboxVariant,
        paint: RadixButtonPaint,
        size: RadixCheckboxSize,
        radius: RadixRadius,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>>;
    fn radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder;
    fn radio_variant(&self, id: impl Into<SharedString>, variant: RadixRadioVariant) -> RadioButtonBuilder;
    fn radio_template(&self, variant: RadixRadioVariant) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn radio_template_with(
        &self,
        variant: RadixRadioVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn radio_template_for(
        &self,
        variant: RadixRadioVariant,
        paint: RadixButtonPaint,
        size: RadixRadioSize,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>>;
    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder;
    fn switch_variant(&self, id: impl Into<SharedString>, variant: RadixSwitchVariant) -> SwitchBuilder;
    fn switch_template(&self, variant: RadixSwitchVariant) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn switch_template_with(
        &self,
        variant: RadixSwitchVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn switch_template_for(
        &self,
        variant: RadixSwitchVariant,
        paint: RadixButtonPaint,
        size: RadixSwitchSize,
        radius: RadixRadius,
    ) -> Arc<dyn ButtonTemplate<SwitchData>>;
    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn page_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder;
    fn tabs(&self, id: impl Into<SharedString>) -> TabsBuilder;
    fn popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder;
    fn popup_menu_variant(&self, id: impl Into<SharedString>, variant: RadixPopupMenuVariant) -> PopupMenuBuilder;
    fn solid_popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder;
    /// Bare template for previews that render forced states instead of live controls.
    fn popup_menu_template(&self, variant: RadixPopupMenuVariant, tone: RadixTone) -> Arc<dyn PopupMenuTemplate>;
    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder;
}

impl RadixLookControlExt for Arc<RadixLook> {
    fn button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        self.solid_button(id)
    }

    fn classic_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(classic_button_template(Arc::clone(self)))
    }

    fn solid_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::Solid, RadixButtonPaint::accent()))
    }

    fn surface_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::Surface, RadixButtonPaint::accent()))
    }

    fn soft_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::Soft, RadixButtonPaint::accent()))
    }

    fn outline_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::Outline, RadixButtonPaint::accent()))
    }

    fn ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::Ghost, RadixButtonPaint::accent()))
    }

    fn quiet_ghost_button(&self, id: impl Into<SharedString>) -> ButtonBuilder<()> {
        Button::new(id).template(self.button_template(RadixButtonVariant::GhostQuiet, RadixButtonPaint::accent()))
    }

    fn button_template(&self, variant: RadixButtonVariant, paint: RadixButtonPaint) -> Arc<dyn ButtonTemplate<()>> {
        if matches!(variant, RadixButtonVariant::Classic) {
            return classic_button_template_with(Arc::clone(self), paint);
        }
        Arc::new(DefaultButtonTemplate::new(button_family_theme_with(Arc::clone(self), variant, paint)))
    }

    fn textfield(&self, id: impl Into<SharedString>) -> TextFieldBuilder {
        self.textfield_variant(id, RadixTextFieldVariant::default())
    }

    fn textfield_variant(&self, id: impl Into<SharedString>, variant: RadixTextFieldVariant) -> TextFieldBuilder {
        textfield::new(id).template(self.textfield_template(variant))
    }

    fn textfield_template(&self, variant: RadixTextFieldVariant) -> Arc<dyn TextFieldTemplate> {
        Arc::new(ThemedTextFieldTemplate::new(textfield_theme_with(Arc::clone(self), variant)))
    }

    fn textarea(&self, id: impl Into<SharedString>) -> TextAreaBuilder {
        self.textarea_variant(id, RadixTextFieldVariant::default())
    }

    fn textarea_variant(&self, id: impl Into<SharedString>, variant: RadixTextFieldVariant) -> TextAreaBuilder {
        TextArea::new(id)
            .template(self.textarea_template(variant))
            .theme(textarea_theme_with(Arc::clone(self), variant))
    }

    fn textarea_template(&self, variant: RadixTextFieldVariant) -> Arc<dyn TextAreaTemplate> {
        Arc::new(ThemedTextAreaTemplate::new(textarea_theme_with(Arc::clone(self), variant)))
    }

    fn slider(&self, id: impl Into<SharedString>) -> SliderBuilder {
        self.slider_variant(id, RadixSliderVariant::default())
    }

    fn slider_variant(&self, id: impl Into<SharedString>, variant: RadixSliderVariant) -> SliderBuilder {
        slider::new(id).template(self.slider_template(variant))
    }

    fn slider_template(&self, variant: RadixSliderVariant) -> Arc<dyn SliderTemplate> {
        Arc::new(ThemedSliderTemplate::new(slider_theme_with(Arc::clone(self), variant)))
    }

    fn checkbox(&self, id: impl Into<SharedString>) -> CheckboxBuilder {
        self.checkbox_variant(id, RadixCheckboxVariant::default())
    }

    fn checkbox_variant(&self, id: impl Into<SharedString>, variant: RadixCheckboxVariant) -> CheckboxBuilder {
        checkbox::new(id).template(self.checkbox_template(variant))
    }

    fn checkbox_template(&self, variant: RadixCheckboxVariant) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        self.checkbox_template_with(variant, RadixButtonPaint::accent())
    }

    fn checkbox_template_with(
        &self,
        variant: RadixCheckboxVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_with(Arc::clone(self), variant, paint)))
    }

    fn checkbox_template_for(
        &self,
        variant: RadixCheckboxVariant,
        paint: RadixButtonPaint,
        size: RadixCheckboxSize,
        radius: RadixRadius,
    ) -> Arc<dyn ButtonTemplate<CheckboxData>> {
        Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_for(Arc::clone(self), variant, paint, size, radius)))
    }

    fn radio(&self, id: impl Into<SharedString>) -> RadioButtonBuilder {
        self.radio_variant(id, RadixRadioVariant::default())
    }

    fn radio_variant(&self, id: impl Into<SharedString>, variant: RadixRadioVariant) -> RadioButtonBuilder {
        radio_button::new(id).template(self.radio_template(variant))
    }

    fn radio_template(&self, variant: RadixRadioVariant) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        self.radio_template_with(variant, RadixButtonPaint::accent())
    }

    fn radio_template_with(
        &self,
        variant: RadixRadioVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        Arc::new(ThemedRadioButtonTemplate::new(radio_theme_with(Arc::clone(self), variant, paint)))
    }

    fn radio_template_for(
        &self,
        variant: RadixRadioVariant,
        paint: RadixButtonPaint,
        size: RadixRadioSize,
    ) -> Arc<dyn ButtonTemplate<RadioButtonData>> {
        Arc::new(ThemedRadioButtonTemplate::new(radio_theme_for(Arc::clone(self), variant, paint, size)))
    }

    fn switch(&self, id: impl Into<SharedString>) -> SwitchBuilder {
        self.switch_variant(id, RadixSwitchVariant::default())
    }

    fn switch_variant(&self, id: impl Into<SharedString>, variant: RadixSwitchVariant) -> SwitchBuilder {
        switch::new(id).template(self.switch_template(variant))
    }

    fn switch_template(&self, variant: RadixSwitchVariant) -> Arc<dyn ButtonTemplate<SwitchData>> {
        self.switch_template_with(variant, RadixButtonPaint::accent())
    }

    fn switch_template_with(
        &self,
        variant: RadixSwitchVariant,
        paint: RadixButtonPaint,
    ) -> Arc<dyn ButtonTemplate<SwitchData>> {
        Arc::new(ThemedSwitchTemplate::new(switch_theme_with(Arc::clone(self), variant, paint)))
    }

    fn switch_template_for(
        &self,
        variant: RadixSwitchVariant,
        paint: RadixButtonPaint,
        size: RadixSwitchSize,
        radius: RadixRadius,
    ) -> Arc<dyn ButtonTemplate<SwitchData>> {
        Arc::new(ThemedSwitchTemplate::new(switch_theme_for(Arc::clone(self), variant, paint, size, radius)))
    }

    fn toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(toggle_template(Arc::clone(self)))
    }

    fn page_toggle(&self, id: impl Into<SharedString>) -> ToggleBuilder {
        toggle::new(id).template(page_toggle_template(Arc::clone(self)))
    }

    fn tabs(&self, id: impl Into<SharedString>) -> TabsBuilder {
        Tabs::new(id).template(Arc::new(ThemedTabsTemplate::new(tabs_theme(Arc::clone(self)))))
    }

    fn popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder {
        self.popup_menu_variant(id, RadixPopupMenuVariant::default())
    }

    fn popup_menu_variant(&self, id: impl Into<SharedString>, variant: RadixPopupMenuVariant) -> PopupMenuBuilder {
        PopupMenu::new(id).template(self.popup_menu_template(variant, RadixTone::default()))
    }

    fn popup_menu_template(&self, variant: RadixPopupMenuVariant, tone: RadixTone) -> Arc<dyn PopupMenuTemplate> {
        Arc::new(ThemedPopupMenuTemplate::new(popup_menu_theme(Arc::clone(self), variant, tone)))
    }

    fn solid_popup_menu(&self, id: impl Into<SharedString>) -> PopupMenuBuilder {
        self.popup_menu(id).trigger_style(luma::controls::popup_menu::PopupMenuTriggerStyle::Primary)
    }

    fn overlay_window(&self, id: impl Into<SharedString>) -> OverlayWindowBuilder {
        overlay_window::overlay_window(id)
            .template(Arc::new(ThemedOverlayWindowTemplate::new(overlay_window_theme(Arc::clone(self)))))
    }
}
