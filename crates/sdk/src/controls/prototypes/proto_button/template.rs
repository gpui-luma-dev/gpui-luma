use std::sync::{Arc, OnceLock, RwLock};

use gpui::{App, Div, FontWeight, Hsla, Stateful, Window, div, px, prelude::*};

use super::{
    ProtoButtonDefaultsRequest, ProtoButtonDefaultsSource, ProtoButtonRenderModel, ProtoButtonResolvedStyle,
    ThemeProtoButtonDefaultsSource, resolve_proto_button_style,
};
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyTheme, ButtonVariant, ControlSize, InteractionState, default_button_family_theme};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtoButtonVisualState {
    Default,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

impl From<InteractionState> for ProtoButtonVisualState {
    fn from(state: InteractionState) -> Self {
        if state.disabled {
            Self::Disabled
        } else if state.pressed {
            Self::Pressed
        } else if state.hovered {
            Self::Hovered
        } else if state.focused {
            Self::Focused
        } else {
            Self::Default
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProtoButtonStatefulOverride<T> {
    pub base: Option<T>,
    pub hovered: Option<T>,
    pub pressed: Option<T>,
    pub focused: Option<T>,
    pub disabled: Option<T>,
}

impl<T> ProtoButtonStatefulOverride<T> {
    pub fn for_state(&self, state: ProtoButtonVisualState) -> Option<&T> {
        match state {
            ProtoButtonVisualState::Default => None,
            ProtoButtonVisualState::Hovered => self.hovered.as_ref(),
            ProtoButtonVisualState::Pressed => self.pressed.as_ref(),
            ProtoButtonVisualState::Focused => self.focused.as_ref(),
            ProtoButtonVisualState::Disabled => self.disabled.as_ref(),
        }
    }

    pub fn resolve(&self, state: ProtoButtonVisualState) -> Option<&T> {
        self.for_state(state).or(self.base.as_ref())
    }
}

#[derive(Clone, Debug)]
pub enum ProtoButtonNullableOverride<T> {
    Inherit,
    Set(T),
    Clear,
}

impl<T> ProtoButtonNullableOverride<T> {
    pub fn resolve<'a>(&'a self, inherited: Option<&'a T>) -> Option<&'a T> {
        match self {
            Self::Inherit => inherited,
            Self::Set(value) => Some(value),
            Self::Clear => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProtoButtonTemplateParams {
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled_opacity: f32,
    pub pointer_cursor_when_enabled: bool,
    pub background: ProtoButtonStatefulOverride<Hsla>,
    pub foreground: ProtoButtonStatefulOverride<Hsla>,
    pub border: ProtoButtonStatefulOverride<Hsla>,
    pub focus_ring: ProtoButtonNullableOverride<Hsla>,
    pub radius: Option<f32>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub gap: Option<f32>,
    pub height: Option<f32>,
    pub typography_size: Option<f32>,
    pub typography_line_height: Option<f32>,
    pub typography_weight: Option<FontWeight>,
}

impl Default for ProtoButtonTemplateParams {
    fn default() -> Self {
        Self {
            variant: ButtonVariant::Standard,
            size: ControlSize::Md,
            disabled_opacity: 0.56,
            pointer_cursor_when_enabled: true,
            background: ProtoButtonStatefulOverride::default(),
            foreground: ProtoButtonStatefulOverride::default(),
            border: ProtoButtonStatefulOverride::default(),
            focus_ring: ProtoButtonNullableOverride::Inherit,
            radius: None,
            padding_x: None,
            padding_y: None,
            gap: None,
            height: None,
            typography_size: None,
            typography_line_height: None,
            typography_weight: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtoButtonTemplateParamField {
    Variant,
    Size,
    DisabledOpacity,
    PointerCursorWhenEnabled,
    BackgroundBase,
    BackgroundHovered,
    BackgroundPressed,
    BackgroundFocused,
    BackgroundDisabled,
    ForegroundBase,
    ForegroundHovered,
    ForegroundPressed,
    ForegroundFocused,
    ForegroundDisabled,
    BorderBase,
    BorderHovered,
    BorderPressed,
    BorderFocused,
    BorderDisabled,
    FocusRing,
    Radius,
    PaddingX,
    PaddingY,
    Gap,
    Height,
    TypographySize,
    TypographyLineHeight,
    TypographyWeight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtoButtonTemplateParamType {
    Color,
    Pixels,
    Number,
    Bool,
    Enum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtoButtonTemplateParamUsage {
    pub name: &'static str,
    pub description: &'static str,
    pub states: &'static [&'static str],
    pub param_type: ProtoButtonTemplateParamType,
    pub param_fields: &'static [ProtoButtonTemplateParamField],
    pub default_source: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtoButtonTemplateUsage {
    pub component: &'static str,
    pub parameters: &'static [ProtoButtonTemplateParamUsage],
}

pub fn proto_button_template_usage() -> &'static ProtoButtonTemplateUsage {
    &PROTO_BUTTON_TEMPLATE_USAGE
}

pub const PROTO_BUTTON_TEMPLATE_USAGE: ProtoButtonTemplateUsage = ProtoButtonTemplateUsage {
    component: "ProtoButton Template",
    parameters: &[
        ProtoButtonTemplateParamUsage {
            name: "variant",
            description: "Base variant resolved from theme before applying parameter overrides.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &[ProtoButtonTemplateParamField::Variant],
            default_source: "ButtonVariant::Prominent",
        },
        ProtoButtonTemplateParamUsage {
            name: "size",
            description: "Base size resolved from theme before applying parameter overrides.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &[ProtoButtonTemplateParamField::Size],
            default_source: "ControlSize::Md",
        },
        ProtoButtonTemplateParamUsage {
            name: "disabled opacity",
            description: "Opacity applied when the control is disabled.",
            states: &["disabled"],
            param_type: ProtoButtonTemplateParamType::Number,
            param_fields: &[ProtoButtonTemplateParamField::DisabledOpacity],
            default_source: "0.56",
        },
        ProtoButtonTemplateParamUsage {
            name: "pointer cursor when enabled",
            description: "Whether to set pointer cursor when enabled.",
            states: &["default", "hovered", "pressed", "focused"],
            param_type: ProtoButtonTemplateParamType::Bool,
            param_fields: &[ProtoButtonTemplateParamField::PointerCursorWhenEnabled],
            default_source: "true",
        },
        ProtoButtonTemplateParamUsage {
            name: "background",
            description: "Background override with base + per-state values applied after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &[
                ProtoButtonTemplateParamField::BackgroundBase,
                ProtoButtonTemplateParamField::BackgroundHovered,
                ProtoButtonTemplateParamField::BackgroundPressed,
                ProtoButtonTemplateParamField::BackgroundFocused,
                ProtoButtonTemplateParamField::BackgroundDisabled,
            ],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "foreground",
            description: "Foreground override with base + per-state values applied after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &[
                ProtoButtonTemplateParamField::ForegroundBase,
                ProtoButtonTemplateParamField::ForegroundHovered,
                ProtoButtonTemplateParamField::ForegroundPressed,
                ProtoButtonTemplateParamField::ForegroundFocused,
                ProtoButtonTemplateParamField::ForegroundDisabled,
            ],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "border",
            description: "Border color override with base + per-state values applied after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &[
                ProtoButtonTemplateParamField::BorderBase,
                ProtoButtonTemplateParamField::BorderHovered,
                ProtoButtonTemplateParamField::BorderPressed,
                ProtoButtonTemplateParamField::BorderFocused,
                ProtoButtonTemplateParamField::BorderDisabled,
            ],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "focus ring",
            description: "Focus ring override semantics: Inherit uses theme, Set(color) forces a color, Clear removes the ring.",
            states: &["focused"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &[ProtoButtonTemplateParamField::FocusRing],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "radius",
            description: "Corner radius override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::Radius],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "padding x",
            description: "Horizontal padding override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::PaddingX],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "padding y",
            description: "Vertical padding override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::PaddingY],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "gap",
            description: "Content gap override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::Gap],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "height",
            description: "Control height override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::Height],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography size",
            description: "Typography size override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::TypographySize],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography line height",
            description: "Typography line height override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &[ProtoButtonTemplateParamField::TypographyLineHeight],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography weight",
            description: "Typography weight override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &[ProtoButtonTemplateParamField::TypographyWeight],
            default_source: "theme",
        },
    ],
};

pub trait ProtoButtonTemplate: Send + Sync {
    fn render(&self, model: &ProtoButtonRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;

    fn read_params(&self) -> Option<ProtoButtonTemplateParams> {
        None
    }

    fn write_params(&self, _params: ProtoButtonTemplateParams) -> bool {
        false
    }
}

pub struct ThemedProtoButtonTemplate {
    defaults_source: Arc<dyn ProtoButtonDefaultsSource>,
    params: RwLock<ProtoButtonTemplateParams>,
}

impl ThemedProtoButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self::with_defaults_source(Arc::new(ThemeProtoButtonDefaultsSource::new(theme)))
    }

    pub fn with_defaults_source(defaults_source: Arc<dyn ProtoButtonDefaultsSource>) -> Self {
        Self { defaults_source, params: RwLock::new(ProtoButtonTemplateParams::default()) }
    }

    pub fn with_params(self, params: ProtoButtonTemplateParams) -> Self {
        Self { defaults_source: self.defaults_source, params: RwLock::new(params) }
    }

    fn resolve_style(
        &self,
        params: &ProtoButtonTemplateParams,
        model: &ProtoButtonRenderModel<'_>,
    ) -> ProtoButtonResolvedStyle {
        let visual_state = ProtoButtonVisualState::from(model.state);
        let defaults = self.defaults_source.resolve_defaults(ProtoButtonDefaultsRequest::text(
            params.variant,
            params.size,
            model.state,
        ));

        resolve_proto_button_style(&defaults, params, visual_state)
    }

    pub fn params(&self) -> ProtoButtonTemplateParams {
        self.params.read().expect("ProtoButton template params lock poisoned").clone()
    }

    pub fn set_params(&self, params: ProtoButtonTemplateParams) {
        *self.params.write().expect("ProtoButton template params lock poisoned") = params;
    }

    pub fn update_params(&self, update: impl FnOnce(&mut ProtoButtonTemplateParams)) {
        let mut params = self.params.write().expect("ProtoButton template params lock poisoned");
        update(&mut params);
    }
}

impl Default for ThemedProtoButtonTemplate {
    fn default() -> Self {
        Self::new(default_button_family_theme())
    }
}

pub fn default_proto_button_template() -> Arc<dyn ProtoButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ProtoButtonTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(ThemedProtoButtonTemplate::default())).clone()
}

impl ProtoButtonTemplate for ThemedProtoButtonTemplate {
    fn render(&self, model: &ProtoButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let params = self.params();
        let style = self.resolve_style(&params, model);

        let control = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(style.gap.value))
            .px(px(style.padding_x.value))
            .py(px(style.padding_y.value))
            .h(px(style.height.value))
            .bg(style.background.value)
            .text_color(style.foreground.value)
            .border_1()
            .border_color(style.border.value)
            .rounded(px(style.radius.value))
            .text_size(px(style.typography_size.value))
            .line_height(px(style.typography_line_height.value))
            .font_weight(style.typography_weight.value)
            .child(model.label.clone());

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, style.focus_ring.value, style.radius.value);

        if model.state.disabled {
            root = root.opacity(style.disabled_opacity.value);
        } else if style.pointer_cursor_when_enabled.value {
            root = root.cursor_pointer();
        }

        root
    }

    fn read_params(&self) -> Option<ProtoButtonTemplateParams> {
        Some(self.params())
    }

    fn write_params(&self, params: ProtoButtonTemplateParams) -> bool {
        self.set_params(params);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProtoButtonNullableOverride, ProtoButtonRenderModel, ProtoButtonStatefulOverride, ProtoButtonTemplateParams,
        ProtoButtonVisualState, ThemedProtoButtonTemplate, proto_button_template_usage,
    };
    use crate::controls::prototypes::proto_button::ProtoButtonSize;
    use crate::theme::{ButtonFamilyRole, ButtonVariant, InteractionState, default_button_family_theme};

    #[test]
    fn template_usage_metadata_is_well_formed() {
        let usage = proto_button_template_usage();

        assert_eq!(usage.component, "ProtoButton Template");
        assert!(!usage.parameters.is_empty());

        for parameter in usage.parameters {
            assert!(!parameter.name.is_empty());
            assert!(!parameter.description.is_empty());
            assert!(!parameter.states.is_empty());
            assert!(!parameter.param_fields.is_empty());
            assert!(!parameter.default_source.is_empty());
        }
    }

    #[test]
    fn params_support_runtime_read_write_with_interior_mutability() {
        let template = ThemedProtoButtonTemplate::new(default_button_family_theme());
        let initial = template.params();

        assert_eq!(initial.variant, ButtonVariant::Prominent);

        template.update_params(|params| {
            params.variant = ButtonVariant::Standard;
            params.disabled_opacity = 0.72;
            params.radius = Some(10.0);
        });

        let updated = template.params();
        assert_eq!(updated.variant, ButtonVariant::Standard);
        assert_eq!(updated.disabled_opacity, 0.72);
        assert_eq!(updated.radius, Some(10.0));
    }

    #[test]
    fn stateful_override_resolve_prefers_state_over_base() {
        let overrides = ProtoButtonStatefulOverride {
            base: Some(10),
            hovered: Some(20),
            pressed: Some(30),
            focused: Some(40),
            disabled: Some(50),
        };

        assert_eq!(overrides.resolve(ProtoButtonVisualState::Default), Some(&10));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Hovered), Some(&20));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Pressed), Some(&30));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Focused), Some(&40));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Disabled), Some(&50));
    }

    #[test]
    fn stateful_override_resolve_falls_back_to_base_when_state_missing() {
        let overrides = ProtoButtonStatefulOverride { base: Some(7), hovered: Some(9), ..Default::default() };

        assert_eq!(overrides.resolve(ProtoButtonVisualState::Default), Some(&7));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Hovered), Some(&9));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Pressed), Some(&7));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Focused), Some(&7));
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Disabled), Some(&7));
    }

    #[test]
    fn stateful_override_resolve_returns_none_when_no_values_exist() {
        let overrides = ProtoButtonStatefulOverride::<i32>::default();

        assert_eq!(overrides.resolve(ProtoButtonVisualState::Default), None);
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Hovered), None);
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Pressed), None);
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Focused), None);
        assert_eq!(overrides.resolve(ProtoButtonVisualState::Disabled), None);
    }

    #[test]
    fn nullable_override_inherit_uses_inherited_value() {
        let override_value = ProtoButtonNullableOverride::<i32>::Inherit;
        let inherited = 42;

        assert_eq!(override_value.resolve(Some(&inherited)).copied(), Some(42));
        assert_eq!(override_value.resolve(None), None);
    }

    #[test]
    fn nullable_override_set_wins_over_inherited_value() {
        let override_value = ProtoButtonNullableOverride::Set(7);
        let inherited = 42;

        assert_eq!(override_value.resolve(Some(&inherited)).copied(), Some(7));
        assert_eq!(override_value.resolve(None).copied(), Some(7));
    }

    #[test]
    fn nullable_override_clear_removes_value() {
        let override_value = ProtoButtonNullableOverride::<i32>::Clear;
        let inherited = 42;

        assert_eq!(override_value.resolve(Some(&inherited)), None);
        assert_eq!(override_value.resolve(None), None);
    }

    #[test]
    fn resolve_appearance_prefers_state_override_over_base_and_theme() {
        let template = ThemedProtoButtonTemplate::new(default_button_family_theme());
        let id = gpui::SharedString::from("proto-button-test-state-precedence");
        let label = gpui::SharedString::from("Proto");
        let model = ProtoButtonRenderModel {
            id: &id,
            label: &label,
            size: ProtoButtonSize::Md,
            state: InteractionState { hovered: true, ..InteractionState::default() },
        };

        let mut params = ProtoButtonTemplateParams::default();
        let base_background = gpui::Hsla { h: 0.10, s: 0.30, l: 0.40, a: 1.0 };
        let hovered_background = gpui::Hsla { h: 0.60, s: 0.70, l: 0.35, a: 1.0 };
        params.background.base = Some(base_background);
        params.background.hovered = Some(hovered_background);

        let style = template.resolve_style(&params, &model);
        assert_eq!(style.background.value, hovered_background);
    }

    #[test]
    fn resolve_appearance_falls_back_to_base_override_when_state_override_missing() {
        let template = ThemedProtoButtonTemplate::new(default_button_family_theme());
        let id = gpui::SharedString::from("proto-button-test-base-fallback");
        let label = gpui::SharedString::from("Proto");
        let model = ProtoButtonRenderModel {
            id: &id,
            label: &label,
            size: ProtoButtonSize::Md,
            state: InteractionState { pressed: true, ..InteractionState::default() },
        };

        let mut params = ProtoButtonTemplateParams::default();
        let base_background = gpui::Hsla { h: 0.15, s: 0.55, l: 0.42, a: 1.0 };
        params.background.base = Some(base_background);

        let style = template.resolve_style(&params, &model);
        assert_eq!(style.background.value, base_background);
    }

    #[test]
    fn resolve_appearance_uses_theme_when_no_overrides_are_present() {
        let template = ThemedProtoButtonTemplate::new(default_button_family_theme());
        let id = gpui::SharedString::from("proto-button-test-theme-fallback");
        let label = gpui::SharedString::from("Proto");
        let model = ProtoButtonRenderModel {
            id: &id,
            label: &label,
            size: ProtoButtonSize::Md,
            state: InteractionState::default(),
        };

        let params = ProtoButtonTemplateParams::default();
        let style = template.resolve_style(&params, &model);

        let expected =
            default_button_family_theme().resolve(params.variant, ButtonFamilyRole::Text, params.size, model.state);
        assert_eq!(style.background.value, expected.background);
    }
}
