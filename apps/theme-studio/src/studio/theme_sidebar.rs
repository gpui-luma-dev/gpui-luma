use std::collections::{HashMap, HashSet};

use gpui::{App, Context, Entity, Render, Subscription, Window, div, prelude::*, px};

use gpui_luma::controls::accordion::{AccordionContent, AccordionControl, AccordionItem, AccordionTrigger};

use gpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use gpui_luma::controls::slider::{Slider, SliderEvent};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::controls::textfield::{
    TextField, TextFieldAppearance, TextFieldAppearanceOverride, TextFieldBuilder, TextFieldEvent,
};
use gpui_luma::theme::ControlSize;
use gpui_luma::{hstack, vstack};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnLookControlExt, ShadcnTextSize};

use super::content_tabs::theme_studio_tabs_navigation_template;
use super::token_color_row::token_color_row;

use gpui::Hsla;

use crate::studio::app::ThemeStudioApp;
use crate::studio::export::{catalog_color_for_token, token_css_name};
use crate::studio::overrides::{
    RADIUS_REM_MAX, RADIUS_REM_MIN, SHADOW_BLUR_MAX, SHADOW_BLUR_MIN, SHADOW_OFFSET_X_MAX, SHADOW_OFFSET_X_MIN,
    SHADOW_OFFSET_Y_MAX, SHADOW_OFFSET_Y_MIN, SHADOW_OPACITY_MAX, SHADOW_OPACITY_MIN, SHADOW_SPREAD_MAX,
    SHADOW_SPREAD_MIN, SPACING_REM_MAX, SPACING_REM_MIN, StudioOverrides, ThemeShadowOverride, clamp_radius_rem,
    clamp_shadow_blur, clamp_shadow_offset_x, clamp_shadow_offset_y, clamp_shadow_opacity, clamp_shadow_spread,
    clamp_spacing_rem, resolved_shadow_override,
};
use crate::studio::panels::{format_hex_color, parse_hex_color};
use crate::theme::available_themes;

const TOKEN_CATEGORIES: &[(&str, &[(&str, &str)])] = &[
    ("BASE", &[("background", "Background"), ("foreground", "Foreground")]),
    ("PRIMARY", &[("primary", "Background"), ("primary-foreground", "Foreground")]),
    ("SECONDARY", &[("secondary", "Background"), ("secondary-foreground", "Foreground")]),
    ("ACCENT", &[("accent", "Background"), ("accent-foreground", "Foreground")]),
    ("CARD", &[("card", "Background"), ("card-foreground", "Foreground")]),
    ("POPOVER", &[("popover", "Background"), ("popover-foreground", "Foreground")]),
    ("MUTED", &[("muted", "Background"), ("muted-foreground", "Foreground")]),
    ("DESTRUCTIVE", &[("destructive", "Background"), ("destructive-foreground", "Foreground")]),
    ("BORDER & INPUT", &[("border", "Border"), ("input", "Input"), ("ring", "Ring")]),
    (
        "CHART",
        &[
            ("chart-1", "Chart 1"),
            ("chart-2", "Chart 2"),
            ("chart-3", "Chart 3"),
            ("chart-4", "Chart 4"),
            ("chart-5", "Chart 5"),
        ],
    ),
    (
        "SIDEBAR",
        &[
            ("sidebar", "Background"),
            ("sidebar-foreground", "Foreground"),
            ("sidebar-primary", "Primary"),
            ("sidebar-primary-foreground", "Primary Foreground"),
            ("sidebar-accent", "Accent"),
            ("sidebar-accent-foreground", "Accent Foreground"),
            ("sidebar-border", "Border"),
            ("sidebar-ring", "Ring"),
        ],
    ),
];

const OTHER_CATEGORIES: &[&str] = &["HSL ADJUSTMENTS", "RADIUS", "SPACING", "SHADOW"];
const METRIC_STEP_REM: f32 = 0.01;
const METRIC_FIELD_WIDTH: f32 = 88.0;
const SHADOW_COLOR_SWATCH_SIZE: f32 = 28.0;
const SHADOW_COLOR_FIELD_WIDTH: f32 = 250.0;
const SHADOW_SECTION_GAP: f32 = 4.0;
const SHADOW_ROW_PADDING_TOP: f32 = 0.0;
const SHADOW_ROW_PADDING_BOTTOM: f32 = 0.0;
const DEFAULT_RADIUS_REM: f32 = 0.5;
const DEFAULT_SPACING_REM: f32 = 0.25;
const REM_IN_PX: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum SidebarTab {
    #[default]
    Colors,
    Typography,
    Other,
}

impl SidebarTab {
    fn from_id(id: &str) -> Option<Self> {
        match id {
            "colors" => Some(Self::Colors),
            "typography" => Some(Self::Typography),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

/// Theme token editing state owned by the sidebar (selectors, fields, overrides).
pub struct ThemeSidebarViewModel {
    pub look: std::sync::Arc<ShadcnLook>,
    pub global_overrides: HashMap<String, Hsla>,
    pub token_fields: HashMap<String, TextField>,
    pub radius_field: TextField,
    pub spacing_field: TextField,
    pub radius_slider: Slider,
    pub spacing_slider: Slider,
    pub shadow_override: ThemeShadowOverride,
    pub shadow_color_field: TextField,
    pub shadow_opacity_field: TextField,
    pub shadow_blur_field: TextField,
    pub shadow_spread_field: TextField,
    pub shadow_offset_x_field: TextField,
    pub shadow_offset_y_field: TextField,
    pub shadow_opacity_slider: Slider,
    pub shadow_blur_slider: Slider,
    pub shadow_spread_slider: Slider,
    pub shadow_offset_x_slider: Slider,
    pub shadow_offset_y_slider: Slider,
}

pub struct ThemeSidebar {
    vm: ThemeSidebarViewModel,
    theme_selector: Entity<Selector>,
    tabs: Entity<TabsNavigation>,
    active_tab: SidebarTab,
    token_accordion: Entity<AccordionControl>,
    other_accordion: Entity<AccordionControl>,
    _subscriptions: Vec<Subscription>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<ThemeStudioApp>,
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let theme_items = theme_selector_items();
        let theme_selector = look
            .selector("theme-studio-theme-selector")
            .label("Theme")
            .items(theme_items)
            .selected_id(active_theme_id.clone())
            .spawn(cx);

        let tabs = look
            .tabs_navigation("theme-studio-sidebar-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(theme_studio_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("colors").label("Colors"),
                TabsNavigationItem::new("typography").label("Typography"),
                TabsNavigationItem::new("other").label("Other"),
            ])
            .active("colors")
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |sidebar, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = SidebarTab::from_id(tab_id.as_ref()) {
                sidebar.active_tab = tab;
                cx.notify();
            }
        }));

        let mut token_fields = HashMap::new();

        for (_, tokens) in TOKEN_CATEGORIES {
            for (token, _) in *tokens {
                let initial = token_hex_value(&look, &global_overrides, token);

                let field = look
                    .textfield(format!("theme-studio-token-{token}"))
                    .value(initial)
                    .full_width(true)
                    .token_style()
                    .spawn(cx);

                token_fields.insert(token.to_string(), field);
            }
        }

        let radius_rem = effective_radius_rem(&look, overrides);
        let spacing_rem = effective_spacing_rem(&look, overrides);
        let radius_field = build_metric_field(&look, "radius", radius_rem, cx);
        let spacing_field = build_metric_field(&look, "spacing", spacing_rem, cx);
        let radius_slider = build_metric_slider(&look, "radius", RADIUS_REM_MIN, RADIUS_REM_MAX, radius_rem, cx);
        let spacing_slider = build_metric_slider(&look, "spacing", SPACING_REM_MIN, SPACING_REM_MAX, spacing_rem, cx);

        let shadow = resolved_shadow_override(&look, overrides);
        let shadow_color_field = build_shadow_color_field(&look, &shadow, cx);
        let shadow_opacity_field = build_shadow_number_field(&look, "shadow-opacity", shadow.opacity(), cx);
        let shadow_blur_field = build_shadow_number_field(&look, "shadow-blur", shadow.blur_px, cx);
        let shadow_spread_field = build_shadow_number_field(&look, "shadow-spread", shadow.spread_px, cx);
        let shadow_offset_x_field = build_shadow_number_field(&look, "shadow-offset-x", shadow.offset_x_px, cx);
        let shadow_offset_y_field = build_shadow_number_field(&look, "shadow-offset-y", shadow.offset_y_px, cx);
        let shadow_opacity_slider =
            build_shadow_slider(&look, "shadow-opacity", SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX, shadow.opacity(), cx);
        let shadow_blur_slider =
            build_shadow_slider(&look, "shadow-blur", SHADOW_BLUR_MIN, SHADOW_BLUR_MAX, shadow.blur_px, cx);
        let shadow_spread_slider =
            build_shadow_slider(&look, "shadow-spread", SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX, shadow.spread_px, cx);
        let shadow_offset_x_slider = build_shadow_slider(
            &look,
            "shadow-offset-x",
            SHADOW_OFFSET_X_MIN,
            SHADOW_OFFSET_X_MAX,
            shadow.offset_x_px,
            cx,
        );
        let shadow_offset_y_slider = build_shadow_slider(
            &look,
            "shadow-offset-y",
            SHADOW_OFFSET_Y_MIN,
            SHADOW_OFFSET_Y_MAX,
            shadow.offset_y_px,
            cx,
        );

        let token_accordion = Self::build_token_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);
        let other_accordion = Self::build_other_accordion(cx.entity(), look.clone(), &HashSet::new(), cx);

        Self {
            vm: ThemeSidebarViewModel {
                look,
                global_overrides,
                token_fields,
                radius_field,
                spacing_field,
                radius_slider,
                spacing_slider,
                shadow_override: shadow.clone(),
                shadow_color_field,
                shadow_opacity_field,
                shadow_blur_field,
                shadow_spread_field,
                shadow_offset_x_field,
                shadow_offset_y_field,
                shadow_opacity_slider,
                shadow_blur_slider,
                shadow_spread_slider,
                shadow_offset_x_slider,
                shadow_offset_y_slider,
            },
            theme_selector,
            tabs,
            active_tab: SidebarTab::Colors,
            token_accordion,
            other_accordion,
            _subscriptions: subscriptions,
        }
    }

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
                        let Some(color) = parse_hex_color(value) else {
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

    fn expanded_token_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(
            &self.token_accordion,
            TOKEN_CATEGORIES.iter().map(|(category, _)| *category),
            "token",
            cx,
        )
    }

    fn expanded_other_category_ids(&self, cx: &App) -> HashSet<String> {
        expanded_category_ids(&self.other_accordion, OTHER_CATEGORIES.iter().copied(), "other", cx)
    }

    fn build_token_accordion(
        sidebar: Entity<Self>,
        look: std::sync::Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("theme-studio-token-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for (category, tokens) in TOKEN_CATEGORIES {
            let items = *tokens;
            let sidebar = sidebar.clone();
            let id = category_item_id("token", category);
            let expanded = if expanded_categories.is_empty() {
                *category == "BASE"
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| {
                        category_token_content(sidebar.read(cx), items).into_any_element()
                    }),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }

    fn build_other_accordion(
        sidebar: Entity<Self>,
        look: std::sync::Arc<ShadcnLook>,
        expanded_categories: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> Entity<AccordionControl> {
        let mut accordion_builder = look
            .accordion("theme-studio-other-accordion")
            .multiple()
            .item_dividers(false)
            .trigger_min_height(28.0)
            .trigger_padding_y(4.0)
            .content_padding_top(0.0)
            .content_padding_bottom(4.0)
            .template(look.accordion_template());

        for category in OTHER_CATEGORIES {
            let sidebar = sidebar.clone();
            let id = category_item_id("other", category);
            let expanded = if expanded_categories.is_empty() {
                true
            } else {
                expanded_categories.contains(&id)
            };
            accordion_builder = accordion_builder.item(
                AccordionItem::new(
                    id,
                    AccordionTrigger::new(*category),
                    AccordionContent::custom(move |_window, cx| {
                        other_category_content(sidebar.read(cx), category).into_any_element()
                    }),
                )
                .expanded(expanded),
            );
        }

        accordion_builder.spawn(cx)
    }
}

/// System monospace face for hex values. Theme CSS `font-mono` families (e.g. Fira Code) are not
/// registered with GPUI unless explicitly loaded, so token fields use a native face per platform.
fn token_field_mono_font() -> gpui::SharedString {
    #[cfg(target_os = "macos")]
    {
        "Menlo".into()
    }
    #[cfg(target_os = "windows")]
    {
        return "Consolas".into();
    }
    #[cfg(target_os = "linux")]
    {
        return "DejaVu Sans Mono".into();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "monospace".into()
    }
}

const TOKEN_FIELD_FONT_SIZE: f32 = 12.0;
const TOKEN_FIELD_LINE_HEIGHT: f32 = 16.0;

fn apply_token_field_appearance(mut appearance: TextFieldAppearance) -> TextFieldAppearance {
    appearance.font_family = token_field_mono_font();
    appearance.typography.size = TOKEN_FIELD_FONT_SIZE;
    appearance.typography.line_height = TOKEN_FIELD_LINE_HEIGHT;
    appearance.padding_y = 2.0;
    appearance.min_height = 22.0;
    appearance
}

fn token_field_appearance_override_arc() -> TextFieldAppearanceOverride {
    std::sync::Arc::new(apply_token_field_appearance)
}

pub trait TokenFieldBuilderExt {
    fn token_style(self) -> Self;
}

impl TokenFieldBuilderExt for TextFieldBuilder {
    fn token_style(self) -> Self {
        self.appearance_override(apply_token_field_appearance)
    }
}

fn build_metric_field(
    look: &std::sync::Arc<ShadcnLook>,
    id: &str,
    value_rem: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    look.textfield(format!("theme-studio-{id}-field"))
        .value(format_metric_rem(value_rem))
        .full_width(true)
        .token_style()
        .spawn(cx)
}

fn build_metric_slider(
    look: &std::sync::Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    look.slider(format!("theme-studio-{id}-slider"))
        .range(min..max)
        .step(METRIC_STEP_REM)
        .value(value)
        .spawn(cx)
}

fn category_item_id(prefix: &str, category: &str) -> String {
    format!("{prefix}-{}", category.to_lowercase().replace(' ', "-").replace('&', "and"))
}

fn expanded_category_ids<'a>(
    accordion: &Entity<AccordionControl>,
    categories: impl IntoIterator<Item = &'a str>,
    prefix: &str,
    cx: &App,
) -> HashSet<String> {
    let accordion = accordion.read(cx);
    categories
        .into_iter()
        .filter_map(|category| {
            let id = category_item_id(prefix, category);
            accordion.is_expanded(&id.clone().into()).then_some(id)
        })
        .collect()
}

fn category_token_content(sidebar: &ThemeSidebar, tokens: &[(&str, &str)]) -> impl IntoElement {
    let theme = &sidebar.vm.look;
    let overrides = &sidebar.vm.global_overrides;
    let chrome = theme.chrome();
    let row_label_typography = theme.mode_tokens().typography.text.label;

    let mut rows = vstack! {
        gap=6;
    };

    for (token, label) in tokens {
        let Some(field) = sidebar.vm.token_fields.get(*token) else {
            continue;
        };
        let color = effective_token_color(theme, overrides, token);
        let label = label.to_string();
        let field = field.clone();

        rows = vstack! {
            gap=6;
            rows,
            token_color_row(label, color, field, &chrome, &row_label_typography),
        };
    }

    rows
}

fn other_category_content(sidebar: &ThemeSidebar, category: &str) -> impl IntoElement {
    match category {
        "RADIUS" => metric_category_content(
            sidebar,
            "Radius",
            sidebar.vm.radius_slider.clone(),
            sidebar.vm.radius_field.clone(),
        )
        .into_any_element(),
        "SPACING" => metric_category_content(
            sidebar,
            "Spacing",
            sidebar.vm.spacing_slider.clone(),
            sidebar.vm.spacing_field.clone(),
        )
        .into_any_element(),
        "SHADOW" => shadow_category_content(sidebar).into_any_element(),
        _ => category_placeholder_content(sidebar, category).into_any_element(),
    }
}

fn category_placeholder_content(sidebar: &ThemeSidebar, category: &str) -> impl IntoElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();

    div()
        .w_full()
        .pt(px(2.0))
        .pb(px(6.0))
        .text_xs()
        .text_color(chrome.muted_text)
        .child(format!("{category} controls coming soon."))
}

fn metric_category_content(sidebar: &ThemeSidebar, label: &str, slider: Slider, field: TextField) -> impl IntoElement {
    slider_field_row(sidebar, label, slider, field, "rem")
}

fn shadow_category_content(sidebar: &ThemeSidebar) -> impl IntoElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let swatch_color = Hsla { a: 1.0, ..sidebar.vm.shadow_override.color };

    vstack! {
        gap=SHADOW_SECTION_GAP;
        hstack! {
            gap=10 align=center;
            div()
                .size(px(SHADOW_COLOR_SWATCH_SIZE))
                .flex_shrink_0()
                .rounded(px(8.0))
                .bg(swatch_color)
                .border_1()
                .border_color(chrome.border),
            div()
                .w(px(SHADOW_COLOR_FIELD_WIDTH))
                .max_w_full()
                .child(sidebar.vm.shadow_color_field.clone()),
        },
        slider_field_row_compact(sidebar, "Opacity", sidebar.vm.shadow_opacity_slider.clone(), sidebar.vm.shadow_opacity_field.clone(), ""),
        slider_field_row_compact(sidebar, "Blur", sidebar.vm.shadow_blur_slider.clone(), sidebar.vm.shadow_blur_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Spread", sidebar.vm.shadow_spread_slider.clone(), sidebar.vm.shadow_spread_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Offset X", sidebar.vm.shadow_offset_x_slider.clone(), sidebar.vm.shadow_offset_x_field.clone(), "px"),
        slider_field_row_compact(sidebar, "Offset Y", sidebar.vm.shadow_offset_y_slider.clone(), sidebar.vm.shadow_offset_y_field.clone(), "px"),
    }
    .w_full()
}

fn slider_field_row(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
) -> impl IntoElement {
    slider_field_row_with_padding(sidebar, label, slider, field, unit, 6.0, 8.0)
}

fn slider_field_row_compact(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
) -> impl IntoElement {
    slider_field_row_with_padding(
        sidebar,
        label,
        slider,
        field,
        unit,
        SHADOW_ROW_PADDING_TOP,
        SHADOW_ROW_PADDING_BOTTOM,
    )
}

fn slider_field_row_with_padding(
    sidebar: &ThemeSidebar,
    label: &str,
    slider: Slider,
    field: TextField,
    unit: &'static str,
    padding_top: f32,
    padding_bottom: f32,
) -> impl IntoElement {
    let theme = &sidebar.vm.look;
    let chrome = theme.chrome();
    let row_label_typography = theme.typography_scale(ShadcnTextSize::Xs);
    let unit_style = theme.typography_scale(ShadcnTextSize::Sm);
    let label = label.to_string();

    hstack! {
        gap=10 align=center;
        div()
            .typography_style(row_label_typography)
            .text_color(chrome.body_text)
            .child(label),
        div()
            .flex_1()
            .min_w(px(0.0))
            .child(slider),
        div()
            .w(px(METRIC_FIELD_WIDTH))
            .child(field),
        div()
            .typography_style(unit_style)
            .text_color(chrome.muted_text)
            .child(unit),
    }
    .w_full()
    .min_w(px(0.0))
    .pt(px(padding_top))
    .pb(px(padding_bottom))
}

fn parse_metric_rem(value: &str, min: f32, max: f32) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(min, max))
}

fn format_metric_rem(value: f32) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let mut text = format!("{rounded:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn effective_radius_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.radius_rem().unwrap_or_else(|| {
        look.parse_pixel_token("radius").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_RADIUS_REM)
    })
}

fn effective_spacing_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.spacing_rem().unwrap_or_else(|| {
        look.parse_pixel_token("spacing").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_SPACING_REM)
    })
}

fn build_shadow_color_field(
    look: &std::sync::Arc<ShadcnLook>,
    shadow: &ThemeShadowOverride,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    look.textfield("theme-studio-shadow-color-field")
        .value(format_shadow_color_input(shadow.color))
        .full_width(true)
        .token_style()
        .spawn(cx)
}

fn build_shadow_number_field(
    look: &std::sync::Arc<ShadcnLook>,
    id: &str,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> TextField {
    look.textfield(format!("theme-studio-{id}-field"))
        .value(format_shadow_number(value))
        .full_width(true)
        .token_style()
        .spawn(cx)
}

fn build_shadow_slider(
    look: &std::sync::Arc<ShadcnLook>,
    id: &str,
    min: f32,
    max: f32,
    value: f32,
    cx: &mut Context<ThemeSidebar>,
) -> Slider {
    look.slider(format!("theme-studio-{id}-slider"))
        .range(min..max)
        .step(METRIC_STEP_REM)
        .value(value)
        .spawn(cx)
}

fn format_shadow_color_input(color: Hsla) -> String {
    format!("hsl({} {}% {}%)", (color.h * 360.0).round(), (color.s * 100.0).round(), (color.l * 100.0).round(),)
}

fn format_shadow_number(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let mut text = format!("{rounded:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn parse_shadow_color_input(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }

    let inner = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("hsl(") && trimmed.ends_with(')') {
        &trimmed[4..trimmed.len() - 1]
    } else if trimmed.len() >= 6 && trimmed[..5].eq_ignore_ascii_case("hsla(") && trimmed.ends_with(')') {
        &trimmed[5..trimmed.len() - 1]
    } else {
        return None;
    };

    let normalized = inner.replace(',', " ");
    let (channels, alpha) = match normalized.split_once('/') {
        Some((channels, alpha)) => (channels.trim(), Some(alpha.trim())),
        None => (normalized.trim(), None),
    };
    let mut parts = channels.split_whitespace();
    let hue = parts.next()?.parse::<f32>().ok()? / 360.0;
    let saturation = parts.next()?.trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let lightness = parts.next()?.trim_end_matches('%').parse::<f32>().ok()? / 100.0;
    let alpha = alpha.and_then(|value| value.parse::<f32>().ok()).unwrap_or(1.0);

    Some(Hsla {
        h: hue.rem_euclid(1.0),
        s: saturation.clamp(0.0, 1.0),
        l: lightness.clamp(0.0, 1.0),
        a: alpha.clamp(0.0, 1.0),
    })
}

fn token_hex_value(look: &ShadcnLook, global_overrides: &HashMap<String, Hsla>, token: &str) -> String {
    format_hex_color(effective_token_color(look, global_overrides, token))
}

fn effective_token_color(look: &ShadcnLook, global_overrides: &HashMap<String, Hsla>, token: &str) -> Hsla {
    token_color_with_fallback(look, global_overrides, token, gpui::hsla(0.0, 0.0, 0.5, 1.0))
}

/// Resolves a theme token when present; otherwise uses `fallback` (e.g. chrome defaults).
fn token_color_with_fallback(
    look: &ShadcnLook,
    global_overrides: &HashMap<String, Hsla>,
    token: &str,
    fallback: Hsla,
) -> Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| catalog_color_for_token(look, token))
        .unwrap_or(fallback)
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.vm.look.chrome();
        let has_catalog = self.vm.look.has_css_catalog();
        let sidebar_bg =
            token_color_with_fallback(&self.vm.look, &self.vm.global_overrides, "sidebar", chrome.panel_background);

        let colors_body = {
            let mut body = div().flex().flex_col().w_full().gap(px(10.0));

            if !has_catalog {
                body = body.child(
                    div()
                        .text_xs()
                        .text_color(chrome.muted_text)
                        .child("Native default theme: pick a tweakcn theme above for catalog-backed swatches."),
                );
            }

            body.child(div().w_full().child(self.token_accordion.clone()))
        };

        let other_body = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(10.0))
            .child(div().w_full().child(self.other_accordion.clone()));

        let tab_body = match self.active_tab {
            SidebarTab::Colors => colors_body.into_any_element(),
            SidebarTab::Typography => div().w_full().into_any_element(),
            SidebarTab::Other => other_body.into_any_element(),
        };

        div()
            .id("theme-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("theme-studio-sidebar-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(self.theme_selector.clone()),
            )
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(
                div()
                    .id("theme-studio-sidebar-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_y_scroll()
                    .p(px(12.0))
                    .child(tab_body),
            )
    }
}

fn theme_selector_items() -> Vec<SelectorItem> {
    let mut items = vec![SelectorItem::new("default").label("Default")];
    for theme in available_themes() {
        items.push(SelectorItem::new(theme.id).label(theme.display_name()));
    }
    items
}
