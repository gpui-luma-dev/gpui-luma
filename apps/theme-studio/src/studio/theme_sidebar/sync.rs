use gpui::Context;
use gpui_luma::theme::ControlSize;
use gpui_luma::controls::tabs_navigation::TabsNavigationWidthMode;

use super::controls::token_field_appearance_override_arc;
use super::model::TOKEN_CATEGORIES;
use super::parsing::{
    effective_radius_rem, effective_spacing_rem, format_metric_rem, format_shadow_color_input, format_shadow_number,
    token_hex_value,
};
use super::ThemeSidebar;
use crate::studio::content_tabs::theme_studio_tabs_navigation_template;
use crate::studio::overrides::{resolved_shadow_override, StudioOverrides};
use gpui_luma_look_shadcn::ShadcnLook;

impl ThemeSidebar {
    pub fn apply_theme_snapshot(
        &mut self,
        look: std::sync::Arc<ShadcnLook>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        let expanded_token_categories = self.expanded_token_category_ids(cx);
        let expanded_other_categories = self.expanded_other_category_ids(cx);
        self.vm.look = look;
        self.vm.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.vm.look.clone();
        self.sync_theme_selector_template(&theme, cx);
        self.sync_tabs_template(&theme, cx);
        self.sync_token_field_templates(&theme, cx);
        self.sync_metric_control_templates(&theme, cx);
        self.sync_shadow_control_templates(&theme, cx);
        self.sync_token_fields_from(theme.as_ref(), overrides, cx);
        self.sync_metric_controls_from(theme.as_ref(), overrides, cx);
        self.sync_shadow_controls_from(theme.as_ref(), overrides, cx);
        self.token_accordion = Self::build_token_accordion(cx.entity(), theme.clone(), &expanded_token_categories, cx);
        self.other_accordion = Self::build_other_accordion(cx.entity(), theme, &expanded_other_categories, cx);
        cx.notify();
    }

    fn sync_theme_selector_template(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_template(theme.selector_template(), cx);
        });
    }

    fn sync_tabs_template(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.tabs.update(cx, |tabs, cx| {
            tabs.set_size(ControlSize::Lg, cx);
            tabs.set_width_mode(TabsNavigationWidthMode::Uniform, cx);
            tabs.set_template(theme_studio_tabs_navigation_template(theme.clone(), ControlSize::Lg), cx);
        });
    }

    fn sync_token_field_templates(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in self.vm.token_fields.values() {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_appearance_override(Some(token_field_appearance_override_arc()), cx);
            });
        }
    }

    fn sync_metric_control_templates(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in [&self.vm.radius_field, &self.vm.spacing_field] {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_appearance_override(Some(token_field_appearance_override_arc()), cx);
            });
        }

        for slider in [&self.vm.radius_slider, &self.vm.spacing_slider] {
            slider.update(cx, |slider, cx| {
                slider.set_template(theme.slider_template(), cx);
            });
        }
    }

    fn sync_shadow_control_templates(&self, theme: &std::sync::Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for field in [
            &self.vm.shadow_color_field,
            &self.vm.shadow_opacity_field,
            &self.vm.shadow_blur_field,
            &self.vm.shadow_spread_field,
            &self.vm.shadow_offset_x_field,
            &self.vm.shadow_offset_y_field,
        ] {
            field.update(cx, |field, cx| {
                field.set_template(theme.textfield_template(), cx);
                field.set_appearance_override(Some(token_field_appearance_override_arc()), cx);
            });
        }

        for slider in [
            &self.vm.shadow_opacity_slider,
            &self.vm.shadow_blur_slider,
            &self.vm.shadow_spread_slider,
            &self.vm.shadow_offset_x_slider,
            &self.vm.shadow_offset_y_slider,
        ] {
            slider.update(cx, |slider, cx| {
                slider.set_template(theme.slider_template(), cx);
            });
        }
    }

    pub fn sync_global_overrides(&mut self, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        let expanded_token_categories = self.expanded_token_category_ids(cx);
        let expanded_other_categories = self.expanded_other_category_ids(cx);
        self.vm.global_overrides = overrides.global_color_overrides.clone();
        let theme = self.vm.look.clone();
        self.sync_token_fields_from(theme.as_ref(), overrides, cx);
        self.sync_metric_controls_from(theme.as_ref(), overrides, cx);
        self.sync_shadow_controls_from(theme.as_ref(), overrides, cx);
        self.token_accordion = Self::build_token_accordion(cx.entity(), theme.clone(), &expanded_token_categories, cx);
        self.other_accordion = Self::build_other_accordion(cx.entity(), theme, &expanded_other_categories, cx);
        cx.notify();
    }

    pub fn sync_theme_selector(&mut self, active_theme_id: impl Into<gpui::SharedString>, cx: &mut Context<Self>) {
        let active_theme_id = active_theme_id.into();
        self.theme_selector.update(cx, |selector, cx| {
            selector.set_selected_id(active_theme_id, cx);
        });
    }

    pub fn sync_token_fields_from(&mut self, look: &ShadcnLook, overrides: &StudioOverrides, cx: &mut Context<Self>) {
        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let Some(field) = self.vm.token_fields.get(*token) else {
                    continue;
                };
                let value = token_hex_value(look, &overrides.global_color_overrides, token);
                field.update(cx, |field, cx| field.set_value(value, cx));
            }
        }
    }

    pub fn sync_metric_controls_from(
        &mut self,
        look: &ShadcnLook,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        let radius_rem = effective_radius_rem(look, overrides);
        let spacing_rem = effective_spacing_rem(look, overrides);

        self.vm.radius_field.update(cx, |field, cx| field.set_value(format_metric_rem(radius_rem), cx));
        self.vm.spacing_field.update(cx, |field, cx| field.set_value(format_metric_rem(spacing_rem), cx));
        self.vm.radius_slider.update(cx, |slider, cx| slider.set_value(radius_rem, cx));
        self.vm.spacing_slider.update(cx, |slider, cx| slider.set_value(spacing_rem, cx));
    }

    pub fn sync_shadow_controls_from(
        &mut self,
        look: &ShadcnLook,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) {
        let shadow = resolved_shadow_override(look, overrides);
        self.vm.shadow_override = shadow.clone();

        self.vm
            .shadow_color_field
            .update(cx, |field, cx| field.set_value(format_shadow_color_input(shadow.color), cx));
        self.vm
            .shadow_opacity_field
            .update(cx, |field, cx| field.set_value(format_metric_rem(shadow.opacity()), cx));
        self.vm
            .shadow_blur_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.blur_px), cx));
        self.vm
            .shadow_spread_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.spread_px), cx));
        self.vm
            .shadow_offset_x_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_x_px), cx));
        self.vm
            .shadow_offset_y_field
            .update(cx, |field, cx| field.set_value(format_shadow_number(shadow.offset_y_px), cx));
        self.vm.shadow_opacity_slider.update(cx, |slider, cx| slider.set_value(shadow.opacity(), cx));
        self.vm.shadow_blur_slider.update(cx, |slider, cx| slider.set_value(shadow.blur_px, cx));
        self.vm.shadow_spread_slider.update(cx, |slider, cx| slider.set_value(shadow.spread_px, cx));
        self.vm.shadow_offset_x_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_x_px, cx));
        self.vm.shadow_offset_y_slider.update(cx, |slider, cx| slider.set_value(shadow.offset_y_px, cx));
    }
}
