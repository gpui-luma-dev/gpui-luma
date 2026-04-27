use super::defaults::ProtoButtonDefaults;
use super::resolved_style::{
    ProtoButtonResolvedOptionalValue, ProtoButtonResolvedStyle, ProtoButtonResolvedValue, ProtoButtonValueSource,
};
use super::{ProtoButtonNullableOverride, ProtoButtonStatefulOverride, ProtoButtonTemplateParams, ProtoButtonVisualState};

/// Pure deterministic resolver for ProtoButton.
///
/// Precedence for stateful fields:
/// 1) state-specific override
/// 2) base override
/// 3) defaults source value
///
/// Nullable semantics:
/// - Inherit => keep defaults source value
/// - Set(T) => force value
/// - Clear => force `None`
pub fn resolve_proto_button_style(
    defaults: &ProtoButtonDefaults,
    params: &ProtoButtonTemplateParams,
    visual_state: ProtoButtonVisualState,
) -> ProtoButtonResolvedStyle {
    ProtoButtonResolvedStyle {
        background: resolve_stateful_with_source(defaults.background, &params.background, visual_state),
        foreground: resolve_stateful_with_source(defaults.foreground, &params.foreground, visual_state),
        border: resolve_stateful_with_source(defaults.border, &params.border, visual_state),
        focus_ring: resolve_nullable_with_source(defaults.focus_ring, &params.focus_ring),

        radius: resolve_optional_override(defaults.radius, params.radius),
        padding_x: resolve_optional_override(defaults.padding_x, params.padding_x),
        padding_y: resolve_optional_override(defaults.padding_y, params.padding_y),
        gap: resolve_optional_override(defaults.gap, params.gap),
        height: resolve_optional_override(defaults.height, params.height),

        typography_size: resolve_optional_override(defaults.typography_size, params.typography_size),
        typography_line_height: resolve_optional_override(
            defaults.typography_line_height,
            params.typography_line_height,
        ),
        typography_weight: resolve_optional_override(defaults.typography_weight, params.typography_weight),

        // These are explicit template knobs today, so they resolve from params.
        disabled_opacity: ProtoButtonResolvedValue::new(params.disabled_opacity, ProtoButtonValueSource::BaseOverride),
        pointer_cursor_when_enabled: ProtoButtonResolvedValue::new(
            params.pointer_cursor_when_enabled,
            ProtoButtonValueSource::BaseOverride,
        ),
    }
}

pub fn resolve_stateful_with_source<T: Copy>(
    default_value: T,
    overrides: &ProtoButtonStatefulOverride<T>,
    visual_state: ProtoButtonVisualState,
) -> ProtoButtonResolvedValue<T> {
    if let Some(value) = overrides.for_state(visual_state).copied() {
        return ProtoButtonResolvedValue::new(value, ProtoButtonValueSource::StateOverride(visual_state));
    }

    if let Some(value) = overrides.base {
        return ProtoButtonResolvedValue::new(value, ProtoButtonValueSource::BaseOverride);
    }

    ProtoButtonResolvedValue::new(default_value, ProtoButtonValueSource::Theme)
}

pub fn resolve_nullable_with_source<T: Copy>(
    inherited: Option<T>,
    override_value: &ProtoButtonNullableOverride<T>,
) -> ProtoButtonResolvedOptionalValue<T> {
    match override_value {
        ProtoButtonNullableOverride::Inherit => {
            ProtoButtonResolvedOptionalValue::new(inherited, ProtoButtonValueSource::Theme)
        }
        ProtoButtonNullableOverride::Set(value) => {
            ProtoButtonResolvedOptionalValue::new(Some(*value), ProtoButtonValueSource::ExplicitSet)
        }
        ProtoButtonNullableOverride::Clear => {
            ProtoButtonResolvedOptionalValue::new(None, ProtoButtonValueSource::ExplicitClear)
        }
    }
}

fn resolve_optional_override<T: Copy>(default_value: T, override_value: Option<T>) -> ProtoButtonResolvedValue<T> {
    match override_value {
        Some(value) => ProtoButtonResolvedValue::new(value, ProtoButtonValueSource::BaseOverride),
        None => ProtoButtonResolvedValue::new(default_value, ProtoButtonValueSource::Theme),
    }
}

#[cfg(test)]
mod tests {
    use gpui::{FontWeight, Hsla};

    use super::*;
    use crate::theme::{ButtonVariant, ControlSize};

    fn hsla(h: f32, s: f32, l: f32) -> Hsla {
        Hsla { h, s, l, a: 1.0 }
    }

    fn defaults() -> ProtoButtonDefaults {
        ProtoButtonDefaults {
            background: hsla(0.10, 0.20, 0.30),
            foreground: hsla(0.90, 0.10, 0.95),
            border: hsla(0.11, 0.15, 0.50),
            focus_ring: Some(hsla(0.62, 0.70, 0.55)),
            radius: 8.0,
            padding_x: 12.0,
            padding_y: 8.0,
            gap: 6.0,
            height: 36.0,
            typography_size: 13.0,
            typography_line_height: 18.0,
            typography_weight: FontWeight::MEDIUM,
            disabled_opacity: 0.56,
            pointer_cursor_when_enabled: true,
        }
    }

    fn params() -> ProtoButtonTemplateParams {
        ProtoButtonTemplateParams { variant: ButtonVariant::Standard, size: ControlSize::Md, ..Default::default() }
    }

    #[test]
    fn state_precedence_is_state_then_base_then_theme() {
        let mut p = params();
        p.background.base = Some(hsla(0.2, 0.3, 0.4));
        p.background.hovered = Some(hsla(0.7, 0.5, 0.3));

        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Hovered);
        assert_eq!(resolved.background.source, ProtoButtonValueSource::StateOverride(ProtoButtonVisualState::Hovered));
        assert_eq!(resolved.background.value, hsla(0.7, 0.5, 0.3));

        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Pressed);
        assert_eq!(resolved.background.source, ProtoButtonValueSource::BaseOverride);
        assert_eq!(resolved.background.value, hsla(0.2, 0.3, 0.4));

        let p = params();
        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Pressed);
        assert_eq!(resolved.background.source, ProtoButtonValueSource::Theme);
        assert_eq!(resolved.background.value, defaults().background);
    }

    #[test]
    fn nullable_semantics_are_deterministic() {
        let mut p = params();

        p.focus_ring = ProtoButtonNullableOverride::Inherit;
        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Focused);
        assert_eq!(resolved.focus_ring.source, ProtoButtonValueSource::Theme);
        assert_eq!(resolved.focus_ring.value, defaults().focus_ring);

        p.focus_ring = ProtoButtonNullableOverride::Set(hsla(0.01, 0.9, 0.5));
        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Focused);
        assert_eq!(resolved.focus_ring.source, ProtoButtonValueSource::ExplicitSet);
        assert_eq!(resolved.focus_ring.value, Some(hsla(0.01, 0.9, 0.5)));

        p.focus_ring = ProtoButtonNullableOverride::Clear;
        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Focused);
        assert_eq!(resolved.focus_ring.source, ProtoButtonValueSource::ExplicitClear);
        assert_eq!(resolved.focus_ring.value, None);
    }

    #[test]
    fn structural_optional_overrides_fallback_to_theme() {
        let mut p = params();
        p.radius = Some(14.0);

        let resolved = resolve_proto_button_style(&defaults(), &p, ProtoButtonVisualState::Default);
        assert_eq!(resolved.radius.value, 14.0);
        assert_eq!(resolved.radius.source, ProtoButtonValueSource::BaseOverride);
        assert_eq!(resolved.padding_x.value, defaults().padding_x);
        assert_eq!(resolved.padding_x.source, ProtoButtonValueSource::Theme);
    }
}
