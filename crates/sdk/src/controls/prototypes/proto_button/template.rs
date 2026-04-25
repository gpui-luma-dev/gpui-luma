use std::sync::{Arc, OnceLock, RwLock};

use gpui::{App, Div, FontWeight, Hsla, Stateful, Window, div, px, prelude::*};

use super::ProtoButtonRenderModel;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, ControlSize, default_button_family_theme};

#[derive(Clone, Debug)]
pub struct ProtoButtonTemplateParams {
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled_opacity: f32,
    pub pointer_cursor_when_enabled: bool,
    pub background: Option<Hsla>,
    pub foreground: Option<Hsla>,
    pub border: Option<Hsla>,
    pub focus_ring: Option<Hsla>,
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
            variant: ButtonVariant::Primary,
            size: ControlSize::Md,
            disabled_opacity: 0.56,
            pointer_cursor_when_enabled: true,
            background: None,
            foreground: None,
            border: None,
            focus_ring: None,
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
    pub param_fields: &'static [&'static str],
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
            param_fields: &["ProtoButtonTemplateParams.variant"],
            default_source: "ButtonVariant::Primary",
        },
        ProtoButtonTemplateParamUsage {
            name: "size",
            description: "Base size resolved from theme before applying parameter overrides.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &["ProtoButtonTemplateParams.size"],
            default_source: "ControlSize::Md",
        },
        ProtoButtonTemplateParamUsage {
            name: "disabled opacity",
            description: "Opacity applied when the control is disabled.",
            states: &["disabled"],
            param_type: ProtoButtonTemplateParamType::Number,
            param_fields: &["ProtoButtonTemplateParams.disabled_opacity"],
            default_source: "0.56",
        },
        ProtoButtonTemplateParamUsage {
            name: "pointer cursor when enabled",
            description: "Whether to set pointer cursor when enabled.",
            states: &["default", "hovered", "pressed", "focused"],
            param_type: ProtoButtonTemplateParamType::Bool,
            param_fields: &["ProtoButtonTemplateParams.pointer_cursor_when_enabled"],
            default_source: "true",
        },
        ProtoButtonTemplateParamUsage {
            name: "background",
            description: "Background override applied to all states after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &["ProtoButtonTemplateParams.background"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "foreground",
            description: "Foreground override applied to all states after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &["ProtoButtonTemplateParams.foreground"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "border",
            description: "Border color override applied to all states after theme resolution.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &["ProtoButtonTemplateParams.border"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "focus ring",
            description: "Focus ring override applied when focused.",
            states: &["focused"],
            param_type: ProtoButtonTemplateParamType::Color,
            param_fields: &["ProtoButtonTemplateParams.focus_ring"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "radius",
            description: "Corner radius override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.radius"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "padding x",
            description: "Horizontal padding override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.padding_x"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "padding y",
            description: "Vertical padding override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.padding_y"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "gap",
            description: "Content gap override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.gap"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "height",
            description: "Control height override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.height"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography size",
            description: "Typography size override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.typography_size"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography line height",
            description: "Typography line height override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Pixels,
            param_fields: &["ProtoButtonTemplateParams.typography_line_height"],
            default_source: "theme",
        },
        ProtoButtonTemplateParamUsage {
            name: "typography weight",
            description: "Typography weight override.",
            states: &["default", "hovered", "pressed", "focused", "disabled"],
            param_type: ProtoButtonTemplateParamType::Enum,
            param_fields: &["ProtoButtonTemplateParams.typography_weight"],
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
    theme: Arc<dyn ButtonFamilyTheme>,
    params: RwLock<ProtoButtonTemplateParams>,
}

impl ThemedProtoButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme, params: RwLock::new(ProtoButtonTemplateParams::default()) }
    }

    pub fn with_params(self, params: ProtoButtonTemplateParams) -> Self {
        Self { theme: self.theme, params: RwLock::new(params) }
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
        let mut appearance = self.theme.resolve(params.variant, ButtonFamilyRole::Text, params.size, model.state);

        if let Some(value) = params.background {
            appearance.background = value;
        }
        if let Some(value) = params.foreground {
            appearance.foreground = value;
        }
        if let Some(value) = params.border {
            appearance.border = value;
        }
        if let Some(value) = params.focus_ring {
            appearance.focus_ring = Some(value);
        }
        if let Some(value) = params.radius {
            appearance.radius = value;
        }
        if let Some(value) = params.padding_x {
            appearance.padding_x = value;
        }
        if let Some(value) = params.padding_y {
            appearance.padding_y = value;
        }
        if let Some(value) = params.gap {
            appearance.gap = value;
        }
        if let Some(value) = params.height {
            appearance.height = value;
        }
        if let Some(value) = params.typography_size {
            appearance.typography.size = value;
        }
        if let Some(value) = params.typography_line_height {
            appearance.typography.line_height = value;
        }
        if let Some(value) = params.typography_weight {
            appearance.typography.weight = value;
        }

        let control = div()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .h(px(appearance.height))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .child(model.label.clone());

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

        if model.state.disabled {
            root = root.opacity(params.disabled_opacity);
        } else if params.pointer_cursor_when_enabled {
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
    use super::{ThemedProtoButtonTemplate, proto_button_template_usage};
    use crate::theme::{ButtonVariant, default_button_family_theme};

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

        assert_eq!(initial.variant, ButtonVariant::Primary);

        template.update_params(|params| {
            params.variant = ButtonVariant::Destructive;
            params.disabled_opacity = 0.72;
            params.radius = Some(10.0);
        });

        let updated = template.params();
        assert_eq!(updated.variant, ButtonVariant::Destructive);
        assert_eq!(updated.disabled_opacity, 0.72);
        assert_eq!(updated.radius, Some(10.0));
    }
}
