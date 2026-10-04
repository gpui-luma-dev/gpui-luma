//! Cross-family regression coverage for selected configuration and common geometry.
use std::collections::HashMap;
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};
use crate::{ShadcnLook, ShadcnTextFieldStyle, StylesheetConfig};

const OVERRIDES: &str = r#"
[common.textfield.geometry]
min_height=47
padding_x=7
padding_y=3
gap=4
icon_size=19
font_size=17
line_height=25
[common.textarea.geometry]
min_height=71
padding_x=9
padding_y=6
font_size=18
line_height=29
[common.progress.geometry]
size=83
stroke_width=3.5
track_height=9
thumb_size=22
[common.stepper.geometry]
step_badge_size=43
track_thickness=4
[common.scrollbar.geometry]
length_h=331
length_v=221
thickness=15
track_thickness=5
thumb_thickness=10
min_thumb_length=34
[common.floating_menu.geometry]
padding=5
min_width=245
item_height=31
item_padding_x=8
item_gap=6
item_icon_size=17
font_size=16
line_height=22
submenu_offset_x=5
[common.popup_menu.geometry]
height=51
padding_x=11
padding_y=4
gap=7
icon_size=21
font_size=17
line_height=25
[common.selector.geometry]
height=49
padding_x=9
padding_y=5
gap=8
icon_size=23
font_size=18
line_height=26
[common.context_menu.geometry]
min_width=231
padding_x=12
padding_y=8
font_size=19
line_height=26
[common.card.geometry]
padding=23
section_gap=11
header_gap=7
body_gap=12
[common.badge.geometry]
min_height=29
padding_x=13
padding_y=5
gap=7
icon_size=17
font_size=15
line_height=21
[common.toolbar.geometry]
padding_x=10
padding_y=6
gap=12
[common.sidebar.geometry]
section_height=24
item_height=34
item_padding_x=11
item_gap=10
item_icon_size=19
[common.control_group.geometry]
padding_x=12
padding_y=7
gap=10
[common.table.geometry]
padding_x=4
padding_y=7
[common.overlay_window.geometry]
padding=19
min_width=450
max_width=570
estimated_height=300
font_size=18
line_height=26
[common.tooltip.geometry]
padding=11
padding_y=6
max_width=345
font_size=15
line_height=22
[common.pager.geometry]
button_size=39
button_min_width=41
control_height=43
padding_x=10
padding_y=12
gap=8
group_gap=19
font_size=14
line_height=20
"#;

fn custom_look() -> ShadcnLook {
    // Leave per-size geometry empty so general overrides exercise every family.
    let mut config = crate::embedded_stylesheet().clone();
    config.common = StylesheetConfig::parse(OVERRIDES).unwrap().common;
    ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap()
}

#[test]
fn input_families_resolve_independent_dimensions_and_typography() {
    let look = custom_look();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
            for style in [
                ShadcnTextFieldStyle::Outline,
                ShadcnTextFieldStyle::Input,
                ShadcnTextFieldStyle::Primary,
                ShadcnTextFieldStyle::Surface,
            ] {
                for bits in 0..32 {
                    let state = gpui_luma::controls::textfield::TextFieldState {
                        hovered: bits & 1 != 0,
                        focused: bits & 2 != 0,
                        focus_visible: bits & 4 != 0,
                        invalid: bits & 8 != 0,
                        ..Default::default()
                    };
                    let enabled = bits & 16 == 0;
                    let field =
                        crate::controls::textfield::textfield_look(&tokens, mode, style, state, enabled, size, &scale);
                    let area = crate::controls::textarea::textarea_look(
                        &tokens,
                        mode,
                        style,
                        gpui_luma::controls::textarea::TextAreaState {
                            hovered: state.hovered,
                            focused: state.focused,
                            focus_visible: state.focus_visible,
                            invalid: state.invalid,
                            ..Default::default()
                        },
                        enabled,
                        size,
                        &scale,
                    );
                    assert_eq!(
                        (field.min_height, field.padding_x, field.padding_y, field.icon_size, field.gap),
                        (47.0, 7.0, 3.0, 19.0, 4.0)
                    );
                    assert_eq!((field.typography.size, field.typography.line_height), (17.0, 25.0));
                    assert_eq!((area.min_height, area.padding_x, area.padding_y), (71.0, 9.0, 6.0));
                    assert_eq!((area.typography.size, area.typography.line_height), (18.0, 29.0));
                    #[cfg(feature = "inspect")]
                    {
                        let inspect = crate::inspect::ShadcnInspect::new(&look);
                        let f = inspect.inspect_textfield_metrics(size);
                        let a = inspect.inspect_textarea_metrics(size);
                        assert_eq!(f.min_height.value_px, field.min_height);
                        assert_eq!(f.padding_x.value_px, field.padding_x);
                        assert_eq!(a.min_height.value_px, area.min_height);
                        assert_eq!(a.padding_y.value_px, area.padding_y);
                        assert!(format!("{:?}", a.min_height.source).contains("common.textarea"));
                        let colors = inspect.inspect_textfield_color_palette(style, state, enabled);
                        assert_eq!(colors.background.value, field.background);
                        assert_eq!(colors.border.value, field.border);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "inspect")]
#[test]
fn remaining_common_geometry_reaches_paint_and_inspection() {
    use gpui_luma::controls::{
        scrollbar::{ScrollbarOrientation, ScrollbarStyle},
        overlay_window::OverlayWindowMode,
        pager::PagerStyle,
        toolbar::ToolbarVariant,
        selector::SelectorTriggerStyle,
    };
    let look = custom_look();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        let inspect = crate::inspect::ShadcnInspect::new(&look);
        for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
            let progress = look.progress_theme().resolve(true, size);
            let p = inspect.inspect_progress_metrics_for_size(size);
            assert_eq!((progress.size, progress.track_height, progress.thumb_size), (83.0, 9.0, 22.0));
            assert_eq!(p.track_height.value_px, progress.track_height);
            assert!(format!("{:?}", p.size.source).contains("common.progress"));
            let stepper = look.stepper_theme().resolve(true, size);
            assert_eq!((stepper.step_badge_size, stepper.track_thickness), (43.0, 4.0));
            assert_eq!(inspect.inspect_stepper_metrics_for_size(size).step_badge_size.value_px, 43.0);
            for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                let scrollbar = crate::paint::scrollbar_look(
                    &tokens,
                    InteractionState::default(),
                    orientation,
                    size,
                    ScrollbarStyle::Ghost,
                );
                let s = inspect.inspect_scrollbar_metrics_for_size(orientation, ScrollbarStyle::Ghost, size);
                assert_eq!(
                    (scrollbar.thickness, scrollbar.thumb_thickness, scrollbar.min_thumb_length),
                    (15.0, 10.0, 34.0)
                );
                assert_eq!(s.length.value_px, scrollbar.length);
                assert!(format!("{:?}", s.length.source).contains("common.scrollbar"));
            }
            let menu = crate::paint::floating_menu_look(&tokens, mode, size);
            let m = inspect.inspect_floating_menu_metrics(size);
            assert_eq!((menu.min_width, menu.item_height, menu.item_icon_size), (245.0, 31.0, 17.0));
            assert_eq!((menu.item_typography.size, menu.item_typography.line_height), (16.0, 22.0));
            assert_eq!(m.item_icon_size.value_px, menu.item_icon_size);
            assert!(format!("{:?}", m.item_icon_size.source).contains("common.floating_menu"));
            let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
            let selector = crate::controls::selector::selector_look(
                &tokens,
                mode,
                SelectorTriggerStyle::Outline,
                InteractionState::default(),
                size,
                &scale,
                false,
            );
            let s = inspect.inspect_selector_metrics(size);
            assert_eq!(
                (selector.trigger_height, selector.trigger_padding_x, selector.trigger_icon_size),
                (49.0, 9.0, 23.0)
            );
            assert_eq!(s.trigger.padding_x.value_px, selector.trigger_padding_x);
            let popup_scale = crate::controls::popup_menu::popup_menu_trigger_scale(
                &tokens,
                mode,
                size,
                InteractionState::default(),
                1.0,
            );
            let popup = crate::controls::popup_menu::popup_menu_palette(
                &tokens,
                mode,
                gpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline,
                gpui_luma::controls::popup_menu::PopupMenuTriggerMetrics { size, ..Default::default() },
                InteractionState::default(),
            );
            assert_eq!((popup_scale.height, popup_scale.padding_x, popup_scale.icon_size), (51.0, 11.0, 21.0));
            assert_eq!((popup.trigger_typography.size, popup.trigger_typography.line_height), (17.0, 25.0));
            let p = inspect
                .inspect_popup_menu_metrics(gpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline, size);
            assert_eq!(p.trigger.height.value_px, popup_scale.height);
            assert!(format!("{:?}", p.trigger.icon_size.source).contains("common.popup_menu"));
            let card = crate::controls::card::card_look(&look, size);
            assert_eq!((card.padding, card.section_gap, card.header_gap, card.body_gap), (23.0, 11.0, 7.0, 12.0));
            assert_eq!(inspect.inspect_card_metrics(size).padding.value_px, 23.0);
            let toolbar = look.toolbar_theme().resolve(true, size, ToolbarVariant::Outline);
            assert_eq!((toolbar.padding_x, toolbar.padding_y, toolbar.gap), (10.0, 6.0, 12.0));
            assert_eq!(inspect.inspect_toolbar_metrics(size).gap.value_px, 12.0);
            let sidebar = crate::paint::sidebar_item_look(&look, false, InteractionState::default(), size);
            assert_eq!((sidebar.height, sidebar.padding_x, sidebar.icon_size, sidebar.gap), (34.0, 11.0, 19.0, 10.0));
            assert_eq!(inspect.inspect_sidebar_metrics(size).item_height.value_px, 34.0);
            let overlay = look.overlay_window_theme().resolve(size, OverlayWindowMode::Modal);
            assert_eq!(
                (overlay.padding, overlay.min_width, overlay.max_width, overlay.estimated_height),
                (19.0, 450.0, 570.0, 300.0)
            );
            assert_eq!(
                inspect.inspect_overlay_window_metrics(size, OverlayWindowMode::Modal).max_width.value_px,
                570.0
            );
        }
        let context = crate::paint::context_menu_look(&tokens, mode, InteractionState::default());
        assert_eq!((context.target_min_width, context.target_padding_x), (231.0, 12.0));
        let group = crate::paint::control_group_list_look(&tokens, true);
        assert_eq!((group.padding_x, group.padding_y, group.gap), (12.0, 7.0, 10.0));
        let table = crate::paint::table_look(&tokens, true, false, ControlSize::Md);
        assert_eq!((table.padding_x, table.padding_y), (4.0, 7.0));
        let badge = crate::badge_look(&look, crate::BadgeVariant::Default, crate::ShadcnSize::Md);
        assert_eq!((badge.min_height, badge.padding_x, badge.gap, badge.icon_size), (29.0, 13.0, 7.0, 17.0));
        for style in [PagerStyle::Minimal, PagerStyle::MinimalEdge, PagerStyle::Numeric] {
            let pager = crate::paint::pager_look(&look, true, style);
            assert_eq!((pager.button_size, pager.button_min_width, pager.group_gap), (39.0, 41.0, 19.0));
            assert_eq!(inspect.inspect_pager_metrics(style).button_size.value_px, 39.0);
        }
        let tooltip = crate::tooltip_theme(&look).resolve();
        assert_eq!(
            (tooltip.padding, tooltip.padding_y, tooltip.max_width, tooltip.text_size),
            (11.0, 6.0, 345.0, 15.0)
        );
    }
}

#[test]
fn selected_configuration_survives_color_copies_token_edits_and_live_replacement() {
    let look = custom_look();
    let copy = look
        .with_color_overrides(&HashMap::from([(
            "primary".into(),
            gpui_luma::color::ColorValue::srgb(0.0, 0.0, 0.0, 1.0),
        )]))
        .unwrap();
    let progress = copy.progress_theme();
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 83.0);
    copy.apply_token_overrides(&HashMap::from([("radius".into(), "10px".into())])).unwrap();
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 83.0);
    let stored_tokens = copy.mode_tokens();
    let independent = crate::test_support::fallback_look();
    copy.replace_theme(&independent);
    assert_eq!(progress.resolve(true, ControlSize::Md).size, 64.0);
    assert_eq!(crate::paint::progress_look(&stored_tokens, true, ControlSize::Md).size, 83.0);
    assert_eq!(look.progress_theme().resolve(true, ControlSize::Md).size, 83.0);
}

#[test]
fn migrated_literal_families_keep_their_bundled_size_baselines() {
    use gpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};
    let look = crate::test_support::fallback_look();
    for (size, diameter, track, thumb, badge) in [
        (ControlSize::Sm, 48.0, 4.0, 12.0, 24.0),
        (ControlSize::Md, 64.0, 6.0, 16.0, 32.0),
        (ControlSize::Lg, 80.0, 8.0, 20.0, 40.0),
    ] {
        let progress = look.progress_theme().resolve(true, size);
        assert_eq!((progress.size, progress.track_height, progress.thumb_size), (diameter, track, thumb));
        assert_eq!(look.stepper_theme().resolve(true, size).step_badge_size, badge);
        let scrollbar = crate::paint::scrollbar_look(
            &look.mode_tokens(),
            InteractionState::default(),
            ScrollbarOrientation::Horizontal,
            size,
            ScrollbarStyle::Ghost,
        );
        assert_eq!(scrollbar.length, 260.0);
    }
}

#[test]
fn sparse_common_geometry_preserves_legacy_fallback_and_explicit_zero() {
    let config = StylesheetConfig::parse(
        r#"
[progress.metrics.md]
size=91
stroke_width=7
track_height=8
thumb_size=19
[common.progress.geometry]
track_height=0
[common.progress.sizes.md]
size=0
"#,
    )
    .unwrap();
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    let progress = look.progress_theme().resolve(true, ControlSize::Md);
    assert_eq!(
        (progress.size, progress.stroke_width, progress.track_height, progress.thumb_size),
        (0.0, 7.0, 0.0, 19.0)
    );
    #[cfg(feature = "inspect")]
    {
        let p = crate::inspect::ShadcnInspect::new(&look).inspect_progress_metrics_for_size(ControlSize::Md);
        assert_eq!(p.size.value_px, 0.0);
        assert!(format!("{:?}", p.size.source).contains("common.progress.sizes.md.size"));
        assert_eq!(p.thumb_size.value_px, 19.0);
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selector_unset_geometry_retains_sdk_provenance_and_values() {
    let look = crate::test_support::fallback_look();
    for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
        let tokens = look.mode_tokens();
        let scale = StandardBoxScale::compute(size, &tokens.metrics, 1.0);
        let s = crate::inspect::ShadcnInspect::new(&look).inspect_selector_metrics(size);
        assert_eq!(s.trigger.padding_x.value_px, scale.padding_x);
        assert_eq!(s.trigger.radius.value_px, scale.radius);
        assert!(!format!("{:?}", s.trigger.padding_x.source).contains("button"));
        assert!(!format!("{:?}", s.trigger.height.source).contains("button"));
    }
}

#[cfg(feature = "inspect")]
#[test]
fn selected_color_rules_follow_the_same_configuration_as_geometry() {
    let mut config = crate::embedded_stylesheet().clone();
    for rule in &mut config.progress.color_rules {
        rule.track_color = "secondary".into();
        rule.progress_color = "secondary-foreground".into();
    }
    let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, config).unwrap();
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        look.set_mode(mode);
        let tokens = look.mode_tokens();
        for enabled in [false, true] {
            let paint = look.progress_theme().resolve(enabled, ControlSize::Md);
            let inspect = crate::inspect::ShadcnInspect::new(&look).inspect_progress_color_palette(enabled);
            assert_eq!(paint.track_color, tokens.catalog.color("secondary").unwrap());
            assert_eq!(paint.progress_color, tokens.catalog.color("secondary-foreground").unwrap());
            assert_eq!(inspect.track_color.value, paint.track_color);
            assert_eq!(inspect.progress_color.value, paint.progress_color);
        }
    }
}
