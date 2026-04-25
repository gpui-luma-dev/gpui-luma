use gpui::{FontWeight, Hsla};

use super::ProtoButtonVisualState;

/// Provenance metadata describing where a resolved value came from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtoButtonValueSource {
    Theme,
    BaseOverride,
    StateOverride(ProtoButtonVisualState),
    ExplicitSet,
    ExplicitClear,
}

impl ProtoButtonValueSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Theme => "theme",
            Self::BaseOverride => "base override",
            Self::StateOverride(_) => "state override",
            Self::ExplicitSet => "set",
            Self::ExplicitClear => "clear",
        }
    }

    pub fn state_label(self) -> Option<&'static str> {
        match self {
            Self::StateOverride(ProtoButtonVisualState::Default) => Some("default"),
            Self::StateOverride(ProtoButtonVisualState::Hovered) => Some("hovered"),
            Self::StateOverride(ProtoButtonVisualState::Pressed) => Some("pressed"),
            Self::StateOverride(ProtoButtonVisualState::Focused) => Some("focused"),
            Self::StateOverride(ProtoButtonVisualState::Disabled) => Some("disabled"),
            _ => None,
        }
    }
}

/// Resolved non-null style value and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtoButtonResolvedValue<T> {
    pub value: T,
    pub source: ProtoButtonValueSource,
}

impl<T> ProtoButtonResolvedValue<T> {
    pub fn new(value: T, source: ProtoButtonValueSource) -> Self {
        Self { value, source }
    }
}

/// Resolved nullable style value and its provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtoButtonResolvedOptionalValue<T> {
    pub value: Option<T>,
    pub source: ProtoButtonValueSource,
}

impl<T> ProtoButtonResolvedOptionalValue<T> {
    pub fn new(value: Option<T>, source: ProtoButtonValueSource) -> Self {
        Self { value, source }
    }
}

/// Final deterministic style contract used for rendering ProtoButton.
///
/// This is intentionally separate from theme appearance models:
/// - defaults are provided by a defaults source
/// - params are merged via resolver precedence rules
/// - rendering consumes this resolved style only
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtoButtonResolvedStyle {
    pub background: ProtoButtonResolvedValue<Hsla>,
    pub foreground: ProtoButtonResolvedValue<Hsla>,
    pub border: ProtoButtonResolvedValue<Hsla>,
    pub focus_ring: ProtoButtonResolvedOptionalValue<Hsla>,

    pub radius: ProtoButtonResolvedValue<f32>,
    pub padding_x: ProtoButtonResolvedValue<f32>,
    pub padding_y: ProtoButtonResolvedValue<f32>,
    pub gap: ProtoButtonResolvedValue<f32>,
    pub height: ProtoButtonResolvedValue<f32>,

    pub typography_size: ProtoButtonResolvedValue<f32>,
    pub typography_line_height: ProtoButtonResolvedValue<f32>,
    pub typography_weight: ProtoButtonResolvedValue<FontWeight>,

    pub disabled_opacity: ProtoButtonResolvedValue<f32>,
    pub pointer_cursor_when_enabled: ProtoButtonResolvedValue<bool>,
}
