use gpui::{Context, Entity, Subscription};
use gpui_luma::controls::selector::SelectorEvent;
use gpui_luma::controls::slider::SliderEvent;
use gpui_luma::controls::textfield::TextFieldEvent;

use super::model::TOKEN_CATEGORIES;
use super::parsing::{parse_metric_rem, parse_shadow_color_input};
use super::ThemeSidebar;
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::{
    RADIUS_REM_MAX, RADIUS_REM_MIN, SHADOW_BLUR_MAX, SHADOW_BLUR_MIN, SHADOW_OFFSET_X_MAX, SHADOW_OFFSET_X_MIN,
    SHADOW_OFFSET_Y_MAX, SHADOW_OFFSET_Y_MIN, SHADOW_OPACITY_MAX, SHADOW_OPACITY_MIN, SHADOW_SPREAD_MAX,
    SHADOW_SPREAD_MIN, clamp_radius_rem, clamp_shadow_blur, clamp_shadow_offset_x, clamp_shadow_offset_y,
    clamp_shadow_opacity, clamp_shadow_spread, clamp_spacing_rem, SPACING_REM_MAX, SPACING_REM_MIN,
};

impl ThemeSidebar {
    /// Wire selector and token field events on the app entity.
    ///
    /// Subscriptions must not be registered on `ThemeSidebar` itself: `cx.subscribe` re-enters
    /// the subscriber entity, and handlers call `theme_sidebar.update`, which panics.
    pub fn wire_subscriptions(
        sidebar: &Entity<Self>,
        cx: &mut Context<ThemeStudioApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        let theme_selector = sidebar.read(cx).theme_selector.clone();
        subscriptions.push(cx.subscribe(&theme_selector, |app, _, event: &SelectorEvent, cx| {
            let SelectorEvent::Change { item_id, .. } = event;
            app.change_theme(item_id.as_ref(), cx);
        }));

        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let field = sidebar.read(cx).vm.token_fields.get(*token).expect("token field").clone();
                let token_key = token.to_string();
                subscriptions.push(cx.subscribe(&field, move |app, _, event: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change { value } = event {
                        let Some(color) = crate::studio::panels::parse_hex_color(value) else {
                            return;
                        };
                        app.set_global_color(&token_key, color, cx);
                    }
                }));
            }
        }

        let radius_field = sidebar.read(cx).vm.radius_field.clone();
        subscriptions.push(cx.subscribe(&radius_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(rem) = parse_metric_rem(value, RADIUS_REM_MIN, RADIUS_REM_MAX).map(clamp_radius_rem) else {
                    return;
                };
                app.set_radius_rem(rem, cx);
            }
        }));

        let spacing_field = sidebar.read(cx).vm.spacing_field.clone();
        subscriptions.push(cx.subscribe(&spacing_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(rem) = parse_metric_rem(value, SPACING_REM_MIN, SPACING_REM_MAX).map(clamp_spacing_rem) else {
                    return;
                };
                app.set_spacing_rem(rem, cx);
            }
        }));

        let radius_slider = sidebar.read(cx).vm.radius_slider.clone();
        subscriptions.push(cx.subscribe(&radius_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_radius_rem(*value, cx);
        }));

        let spacing_slider = sidebar.read(cx).vm.spacing_slider.clone();
        subscriptions.push(cx.subscribe(&spacing_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_spacing_rem(*value, cx);
        }));

        let shadow_color_field = sidebar.read(cx).vm.shadow_color_field.clone();
        subscriptions.push(cx.subscribe(&shadow_color_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(color) = parse_shadow_color_input(value) else {
                    return;
                };
                app.set_shadow_color(color, cx);
            }
        }));

        let shadow_opacity_field = sidebar.read(cx).vm.shadow_opacity_field.clone();
        subscriptions.push(cx.subscribe(&shadow_opacity_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(opacity) =
                    parse_metric_rem(value, SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX).map(clamp_shadow_opacity)
                else {
                    return;
                };
                app.set_shadow_opacity(opacity, cx);
            }
        }));

        let shadow_blur_field = sidebar.read(cx).vm.shadow_blur_field.clone();
        subscriptions.push(cx.subscribe(&shadow_blur_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(blur) = parse_metric_rem(value, SHADOW_BLUR_MIN, SHADOW_BLUR_MAX).map(clamp_shadow_blur)
                else {
                    return;
                };
                app.set_shadow_blur(blur, cx);
            }
        }));

        let shadow_spread_field = sidebar.read(cx).vm.shadow_spread_field.clone();
        subscriptions.push(cx.subscribe(&shadow_spread_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(spread) =
                    parse_metric_rem(value, SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX).map(clamp_shadow_spread)
                else {
                    return;
                };
                app.set_shadow_spread(spread, cx);
            }
        }));

        let shadow_offset_x_field = sidebar.read(cx).vm.shadow_offset_x_field.clone();
        subscriptions.push(cx.subscribe(&shadow_offset_x_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(offset_x) =
                    parse_metric_rem(value, SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_X_MAX).map(clamp_shadow_offset_x)
                else {
                    return;
                };
                app.set_shadow_offset_x(offset_x, cx);
            }
        }));

        let shadow_offset_y_field = sidebar.read(cx).vm.shadow_offset_y_field.clone();
        subscriptions.push(cx.subscribe(&shadow_offset_y_field, |app, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } = event {
                let Some(offset_y) =
                    parse_metric_rem(value, SHADOW_OFFSET_Y_MIN, SHADOW_OFFSET_Y_MAX).map(clamp_shadow_offset_y)
                else {
                    return;
                };
                app.set_shadow_offset_y(offset_y, cx);
            }
        }));

        let shadow_opacity_slider = sidebar.read(cx).vm.shadow_opacity_slider.clone();
        subscriptions.push(cx.subscribe(&shadow_opacity_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_shadow_opacity(*value, cx);
        }));

        let shadow_blur_slider = sidebar.read(cx).vm.shadow_blur_slider.clone();
        subscriptions.push(cx.subscribe(&shadow_blur_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_shadow_blur(*value, cx);
        }));

        let shadow_spread_slider = sidebar.read(cx).vm.shadow_spread_slider.clone();
        subscriptions.push(cx.subscribe(&shadow_spread_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_shadow_spread(*value, cx);
        }));

        let shadow_offset_x_slider = sidebar.read(cx).vm.shadow_offset_x_slider.clone();
        subscriptions.push(cx.subscribe(&shadow_offset_x_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_shadow_offset_x(*value, cx);
        }));

        let shadow_offset_y_slider = sidebar.read(cx).vm.shadow_offset_y_slider.clone();
        subscriptions.push(cx.subscribe(&shadow_offset_y_slider, |app, _, event: &SliderEvent, cx| {
            let SliderEvent::Change { value } = event;
            app.set_shadow_offset_y(*value, cx);
        }));
    }
}
