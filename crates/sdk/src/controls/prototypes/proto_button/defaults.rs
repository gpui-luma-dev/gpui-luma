use std::sync::Arc;

use gpui::{FontWeight, Hsla};

use crate::theme::{
    ButtonFamilyAppearance, ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, ControlSize, InteractionState,
};

/// Immutable baseline style values used by the ProtoButton resolver before template overrides are applied.
#[derive(Clone, Copy, Debug)]
pub struct ProtoButtonDefaults {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub focus_ring: Option<Hsla>,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
    pub typography_size: f32,
    pub typography_line_height: f32,
    pub typography_weight: FontWeight,
    pub disabled_opacity: f32,
    pub pointer_cursor_when_enabled: bool,
}

impl From<ButtonFamilyAppearance> for ProtoButtonDefaults {
    fn from(value: ButtonFamilyAppearance) -> Self {
        Self {
            background: value.background,
            foreground: value.foreground,
            border: value.border,
            focus_ring: value.focus_ring,
            radius: value.radius,
            padding_x: value.padding_x,
            padding_y: value.padding_y,
            gap: value.gap,
            height: value.height,
            typography_size: value.typography.size,
            typography_line_height: value.typography.line_height,
            typography_weight: value.typography.weight,
            disabled_opacity: 0.56,
            pointer_cursor_when_enabled: true,
        }
    }
}

/// Input used when requesting defaults for a specific render pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtoButtonDefaultsRequest {
    pub variant: ButtonVariant,
    pub role: ButtonFamilyRole,
    pub size: ControlSize,
    pub state: InteractionState,
}

impl ProtoButtonDefaultsRequest {
    pub fn text(variant: ButtonVariant, size: ControlSize, state: InteractionState) -> Self {
        Self { variant, role: ButtonFamilyRole::Text, size, state }
    }
}

/// Defaults provider abstraction for ProtoButton.
///
/// This intentionally decouples the prototype resolver from any specific theme/appearance implementation.
/// Different sources can be swapped in without changing resolver or rendering logic.
pub trait ProtoButtonDefaultsSource: Send + Sync {
    fn resolve_defaults(&self, request: ProtoButtonDefaultsRequest) -> ProtoButtonDefaults;
}

/// Adapter that pulls defaults from an existing `ButtonFamilyTheme`.
pub struct ThemeProtoButtonDefaultsSource {
    theme: Arc<dyn ButtonFamilyTheme>,
}

impl ThemeProtoButtonDefaultsSource {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme }
    }

    pub fn theme(&self) -> &Arc<dyn ButtonFamilyTheme> {
        &self.theme
    }
}

impl ProtoButtonDefaultsSource for ThemeProtoButtonDefaultsSource {
    fn resolve_defaults(&self, request: ProtoButtonDefaultsRequest) -> ProtoButtonDefaults {
        self.theme.resolve(request.variant, request.role, request.size, request.state).into()
    }
}
