//! Cross-crate regression tests: compare public inspection output to rendered looks.
use luma::controls::button_family::ButtonFamilyRole;
use luma::theme::{ControlSize, InteractionState, ThemeMode};
use luma_look_shadcn::{ColorSource, MetricSource, ShadcnButtonStyle, ShadcnLook, ShadcnModeTokens};
use luma_look_shadcn::paint;
use crate::*;

fn states() -> impl Iterator<Item = InteractionState> {
    (0..16).map(|bits| InteractionState {
        hovered: bits & 1 != 0,
        pressed: bits & 2 != 0,
        focused: bits & 4 != 0,
        disabled: bits & 8 != 0,
        ..Default::default()
    })
}

fn styles() -> [ShadcnButtonStyle; 5] {
    use ShadcnButtonStyle::*;
    [Primary, Secondary, Outline, Ghost, ContentOnly]
}

#[test]
fn choice_colors_match_runtime_for_every_state_style_and_fallback() {
    for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
        for missing in [None, Some("ring"), Some("border"), Some("primary")] {
            let mut mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), theme_mode).unwrap();
            if let Some(token) = missing {
                mode.catalog.tokens.remove(token);
            }
            for style in styles() {
                for state in states() {
                    for selected in [false, true] {
                        let actual = inspect_checkbox_color_palette(&mode, theme_mode, style, selected, state);
                        let painted = paint::checkbox_look(&mode, style, selected, state, ControlSize::Md);
                        assert_eq!(actual.indicator_background.value, painted.indicator_background);
                        assert_eq!(actual.indicator_border.value, painted.indicator_border);
                        assert_eq!(actual.checkmark_color.value, painted.checkmark_color);
                        assert_eq!(actual.label_color.value, painted.label_color);

                        let actual = inspect_radio_button_color_palette(&mode, theme_mode, style, selected, state);
                        let painted = paint::radio_button_look(&mode, style, selected, state, ControlSize::Md);
                        assert_eq!(actual.indicator_background.value, painted.indicator_background);
                        assert_eq!(actual.indicator_border.value, painted.indicator_border);
                        assert_eq!(actual.dot_color.value, painted.dot_color);
                        assert_eq!(actual.label_color.value, painted.label_color);

                        let actual = inspect_switch_color_palette(&mode, theme_mode, style, selected, state);
                        let painted = paint::switch_look(&mode, theme_mode, style, selected, state, ControlSize::Md);
                        assert_eq!(actual.track_background.value, painted.track_background);
                        assert_eq!(actual.track_border.value, painted.track_border);
                        assert_eq!(actual.thumb_background.value, painted.thumb_background);
                        assert_eq!(actual.thumb_border.value, painted.thumb_border);
                        assert_eq!(actual.label_color.value, painted.label_color);
                    }
                }
            }
        }
    }
}

#[test]
fn focused_checkbox_provenance_and_hover_policy_match_paint() {
    let mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), ThemeMode::Light).unwrap();
    let inspect =
        |state| inspect_checkbox_color_palette(&mode, ThemeMode::Light, ShadcnButtonStyle::Primary, true, state);
    let focused = inspect(InteractionState { focused: true, ..Default::default() });
    assert!(matches!(focused.indicator_border.source, ColorSource::CssVar { ref token } if token == "ring"));
    assert_eq!(focused.indicator_border.value, mode.catalog.color("ring").unwrap());
    let hovered = inspect(InteractionState { hovered: true, ..Default::default() });
    assert_eq!(hovered.indicator_background.value, inspect(InteractionState::default()).indicator_background.value);
}

#[test]
fn button_and_toggle_tables_match_runtime_including_focus_and_typography() {
    for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
        let mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), theme_mode).unwrap();
        for style in styles() {
            for role in [
                ButtonFamilyRole::Text,
                ButtonFamilyRole::Icon,
                ButtonFamilyRole::Toggle { selected: false },
                ButtonFamilyRole::Toggle { selected: true },
            ] {
                for state in states() {
                    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                        let actual = inspect_button_color_palette(&mode, theme_mode, style, role, state);
                        let painted = paint::button_look(&mode, theme_mode, style, role, size, state);
                        assert_eq!(actual.background.value, painted.background);
                        assert_eq!(actual.foreground.value, painted.foreground);
                        assert_eq!(
                            actual.border.value,
                            luma::controls::button_family::button_family_effective_border(painted.border)
                        );
                        let metrics = inspect_button_metrics(&mode, theme_mode, style, role, size, state);
                        assert_eq!(metrics.height.value_px, painted.height);
                        assert_eq!(metrics.padding_x.value_px, painted.padding_x);
                        assert_eq!(metrics.icon_size.value_px, painted.icon_size);
                        let typography = inspect_button_typography_for_size(&mode, theme_mode, size, role);
                        assert_eq!(typography.font_size.value, painted.typography.size.to_string());
                        assert_eq!(typography.line_height.value, painted.typography.line_height.to_string());
                    }
                }
            }
        }
    }
}

#[test]
fn accordion_tables_cover_all_scale_fields_and_sources() {
    use luma::controls::accordion::AccordionScale;
    for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
        for spacing in [None, Some("0.3rem"), Some("invalid")] {
            let mut catalog = crate::test_support::sample_catalog();
            if let Some(spacing) = spacing {
                catalog.tokens.insert("spacing".into(), spacing.into());
            }
            catalog.tokens.insert("radius".into(), "0.625rem".into());
            let mode = ShadcnModeTokens::from_catalog(catalog, theme_mode).unwrap();
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for scale_factor in [1.0, 1.25, 1.5, 2.0] {
                    let actual = inspect_accordion_metrics_at_scale(&mode, theme_mode, size, scale_factor);
                    let painted = AccordionScale::compute(size, &mode.metrics, scale_factor);
                    assert_eq!(actual.trigger_height.value_px, painted.trigger_height);
                    assert_eq!(actual.padding_x.value_px, painted.padding_x);
                    assert_eq!(actual.padding_y.value_px, painted.padding_y);
                    assert_eq!(actual.content_padding_y.value_px, painted.content_padding_y);
                    assert_eq!(actual.radius.value_px, painted.radius);
                    assert_eq!(actual.item_gap.value_px, painted.item_gap);
                    assert_eq!(actual.inner_gap.value_px, painted.inner_gap);
                    assert_eq!(actual.icon_size.value_px, painted.icon_size);
                    assert_eq!(actual.chevron_size.value_px, painted.chevron_size);
                    match spacing {
                        Some("0.3rem") => assert!(matches!(actual.padding_x.source, MetricSource::Derived { .. })),
                        _ => assert!(matches!(actual.padding_x.source, MetricSource::Scaffold { .. })),
                    }
                    assert!(matches!(actual.radius.source, MetricSource::Derived { .. }));
                }
                for state in states() {
                    let actual = inspect_accordion_trigger_color_palette(&mode, theme_mode, state);
                    let painted = paint::accordion_trigger_palette(&mode, theme_mode, state, size);
                    assert_eq!(actual.background.map(|v| v.value), painted.background);
                    assert_eq!(actual.foreground.value, painted.foreground);
                    assert_eq!(actual.border_color.value, painted.border_color);
                    assert_eq!(actual.icon_color.value, painted.icon_color);
                    assert_eq!(actual.chevron_color.value, painted.chevron_color);
                    let table = luma_look_shadcn::tables::typography::resolve_control_typography(&mode, size, false);
                    assert_eq!(table.style.size, painted.typography.size);
                    assert_eq!(table.style.line_height, painted.typography.line_height);
                }
            }
        }
    }
}

#[test]
fn sidebar_metrics_and_focus_match_runtime() {
    let look = ShadcnLook::from_css_str(luma_look_shadcn::FALLBACK_CSS).unwrap();
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        let actual = ShadcnInspect::new(&look).inspect_sidebar_metrics(size);
        let section = paint::sidebar_section_look(&look);
        assert_eq!(actual.section_height.value_px, section.height);
        for state in states() {
            for selected in [false, true] {
                let painted = paint::sidebar_item_look(&look, selected, state, size);
                assert_eq!(actual.item_height.value_px, painted.height);
                assert_eq!(actual.item_padding_x.value_px, painted.padding_x);
                assert_eq!(actual.item_gap.value_px, painted.gap);
                assert_eq!(actual.item_radius.value_px, painted.radius);
                assert_eq!(actual.item_icon_size.value_px, painted.icon_size);
                let colors = ShadcnInspect::new(&look).inspect_sidebar_item_color_palette(selected, state);
                assert_eq!(colors.focus_border.map(|v| v.value), painted.focus_border);
                assert_eq!(colors.background.map(|v| v.value), painted.background);
            }
        }
    }
}

#[test]
fn slider_inspection_matches_paint() {
    for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
        let mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), theme_mode).unwrap();
        for state in states() {
            let actual = inspect_slider_color_palette(&mode, theme_mode, state);
            let painted =
                paint::slider_look(&mode, theme_mode, ShadcnButtonStyle::Primary, ControlSize::Md, None, state);
            assert_eq!(actual.track_background.value, painted.track_background);
            assert_eq!(actual.fill_background.value, painted.fill_background);
            assert_eq!(actual.thumb_background.value, painted.thumb_background);
            assert_eq!(actual.thumb_border.value, painted.thumb_border);
            let metrics = inspect_slider_metrics(&mode, theme_mode);
            assert_eq!(metrics.width.value_px, painted.width);
            assert_eq!(metrics.height.value_px, painted.height);
            assert_eq!(metrics.track_height.value_px, painted.track_height);
            assert_eq!(metrics.thumb_size.value_px, painted.thumb_size);
        }
    }
}

#[test]
fn textfield_hover_focus_invalid_and_missing_tokens_match_runtime() {
    use luma::controls::textfield::TextFieldState;
    use luma_look_shadcn::ShadcnTextFieldStyle;
    for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
        for missing in [None, Some("ring"), Some("input")] {
            let mut mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), theme_mode).unwrap();
            if let Some(token) = missing {
                mode.catalog.tokens.remove(token);
            }
            for style in [
                ShadcnTextFieldStyle::Input,
                ShadcnTextFieldStyle::Outline,
                ShadcnTextFieldStyle::Surface,
                ShadcnTextFieldStyle::Primary,
            ] {
                for bits in 0..32 {
                    let state = TextFieldState {
                        hovered: bits & 1 != 0,
                        focused: bits & 2 != 0,
                        focus_visible: bits & 4 != 0,
                        invalid: bits & 8 != 0,
                        ..Default::default()
                    };
                    let enabled = bits & 16 == 0;
                    let actual = inspect_textfield_color_palette(&mode, theme_mode, style, state, enabled);
                    let painted = paint::textfield_palette(&mode, theme_mode, style, state, enabled);
                    assert_eq!(actual.background.value, painted.background);
                    assert_eq!(actual.border.value, painted.border);
                    assert_eq!(actual.foreground.value, painted.foreground);
                }
            }
        }
    }
}

#[test]
fn incomplete_accordion_and_context_menu_palettes_share_safe_fallbacks() {
    let mut mode = ShadcnModeTokens::from_catalog(crate::test_support::sample_catalog(), ThemeMode::Light).unwrap();
    mode.catalog.tokens.clear();
    for state in states() {
        let actual = inspect_accordion_trigger_color_palette(&mode, ThemeMode::Light, state);
        let painted = paint::accordion_trigger_palette(&mode, ThemeMode::Light, state, ControlSize::Md);
        assert_eq!(actual.foreground.value, painted.foreground);
        assert_eq!(actual.border_color.value, painted.border_color);
        assert_eq!(actual.background.map(|v| v.value), painted.background);
        let actual = inspect_context_menu_color_palette(&mode, ThemeMode::Light, state, ControlSize::Md);
        let painted = paint::context_menu_look(&mode, ThemeMode::Light, state);
        assert_eq!(actual.target_background.value, painted.target_background);
        assert_eq!(actual.target_foreground.value, painted.target_foreground);
        assert_eq!(actual.target_border.value, painted.target_border);
    }
    for expanded in [false, true] {
        let actual = inspect_accordion_content_color_palette(&mode, ThemeMode::Light, expanded);
        let painted = paint::accordion_content_palette(&mode, ThemeMode::Light, expanded);
        assert_eq!(actual.foreground.value, painted.foreground);
        assert_eq!(actual.background.map(|v| v.value), painted.background);
    }
}
