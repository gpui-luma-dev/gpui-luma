//! Native Switch sample inspection. Reads the very render model and theme sent to the SDK template.
use std::sync::Arc;
use gpui::{App, Context, Div, Entity, IntoElement, Render, Stateful, ScrollHandle, Window, div, prelude::*, px};
use gpui_luma::controls::{
    button::{ButtonRenderModel, ButtonTemplate},
    switch::{SwitchData, SwitchTheme, ThemedSwitchTemplate},
};
use gpui_luma::infra::template::{Modifier, TemplateWithModifiers};
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{
    inspector::{InspectPropertyRow, InspectorRowStyle, render_inspector_row},
    native::NativeTheme,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Selection {
    on: bool,
    state: InteractionState,
    size: ControlSize,
    scale: f32,
    mode: ThemeMode,
}

pub struct SwitchInspector {
    theme: NativeTheme,
    selection: Option<Selection>,
    pub rows: Vec<InspectPropertyRow>,
    pub scroll: ScrollHandle,
}
impl SwitchInspector {
    pub fn new(theme: NativeTheme) -> Self {
        Self { theme, selection: None, rows: Vec::new(), scroll: ScrollHandle::new() }
    }
    pub fn set_theme(&mut self, theme: NativeTheme, cx: &mut Context<Self>) {
        self.theme = theme;
        self.selection = None;
        cx.notify();
    }
    fn inspect(
        &mut self,
        theme: &dyn SwitchTheme,
        model: &ButtonRenderModel<SwitchData>,
        scale: f32,
        mode: ThemeMode,
        cx: &mut Context<Self>,
    ) {
        let selection = Selection { on: model.data.checked, state: model.state, size: model.size, scale, mode };
        if self.selection == Some(selection) {
            return;
        }
        self.selection = Some(selection);
        // The SDK template draws focus separately and interpolates between settled endpoints.
        let state = InteractionState { focused: false, ..model.state };
        let palette = theme.resolve(selection.on, state, model.size);
        let geometry = theme.scale(model.size, scale);
        let focus_palette = theme.resolve(selection.on, InteractionState { focused: true, ..model.state }, model.size);
        let row = |label: &str, value: String, source: &str| InspectPropertyRow::new(label, value, source);
        self.rows = vec![
            row("switch.value", format!("{}", selection.on), "Switch render model"),
            row("switch.state", format!("{:?}", model.state), "Switch render model"),
            row("switch.size", format!("{:?}", model.size), "SDK default (no instance override)"),
            row("switch.mode", format!("{mode:?}"), "Native SDK ThemeTokens"),
            row(
                "switch.track.background",
                format!("{:?}", palette.track_background),
                "DefaultSwitchTheme::resolve; settled endpoint",
            ),
            row(
                "switch.track.border",
                format!("{:?}", palette.track_border),
                "DefaultSwitchTheme::resolve; focus drawn separately",
            ),
            row(
                "switch.thumb.background",
                format!("{:?}", palette.thumb_background),
                "DefaultSwitchTheme::resolve; settled endpoint",
            ),
            row("switch.thumb.border", format!("{:?}", palette.thumb_border), "DefaultSwitchTheme::resolve"),
            row("switch.label.color", format!("{:?}", palette.label_color), "DefaultSwitchTheme::resolve"),
            row("switch.track.width", format!("{}px", geometry.track_width), "SwitchTheme::scale / SDK stylesheet"),
            row("switch.track.height", format!("{}px", geometry.track_height), "SwitchTheme::scale / SDK stylesheet"),
            row("switch.thumb.size", format!("{}px", geometry.thumb_size), "SwitchTheme::scale / SDK stylesheet"),
            row(
                "switch.track.padding",
                format!("{}px", geometry.track_padding),
                "SwitchTheme::scale / SDK stylesheet",
            ),
            row("switch.track.radius", format!("{}px", geometry.track_radius), "SwitchTheme::scale / SDK stylesheet"),
            row(
                "switch.disabled.opacity",
                format!("{}", geometry.disabled_opacity),
                "SwitchTheme::scale / SDK stylesheet",
            ),
            row(
                "switch.focus.visible",
                format!("{}", model.state.focused && !model.state.disabled),
                "SDK Switch template; separate focus paint",
            ),
            row(
                "switch.focus.color",
                format!("{:?}", focus_palette.track_border),
                "DefaultSwitchTheme::resolve (focused)",
            ),
            row(
                "switch.focus.width",
                format!("{}px", theme.metrics().focus.width),
                "SwitchTheme::metrics().focus.width",
            ),
            row("switch.focus.gap", format!("{}px", theme.metrics().focus.gap), "SwitchTheme::metrics().focus.gap"),
            row("switch.label.font", palette.label_font_family.to_string(), "DefaultSwitchTheme::resolve"),
            row("switch.gap", format!("{}px", geometry.gap), "SwitchTheme::scale / SDK stylesheet"),
            row("switch.label.typography", format!("{:?}", palette.label_typography), "DefaultSwitchTheme::resolve"),
            row("switch.thumb.shadow", format!("{:?}", palette.thumb_shadow), "DefaultSwitchTheme::resolve"),
        ];
        cx.notify();
    }
}
impl Render for SwitchInspector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let p = &self.theme.tokens.palette;
        div().id("switch-inspector").size_full().overflow_y_scroll().track_scroll(&self.scroll).p(px(16.0))
            .bg(p.surface.panel.background).text_color(p.app.foreground)
            .child(div().text_size(px(18.0)).child("Theme Inspector · SDK Switch"))
            .child(div().py(px(8.0)).child("Native default variant; live size, value and interaction state. Colors report settled endpoints during motion."))
            .children(self.rows.iter().enumerate().map(|(index, row)| render_inspector_row(
                InspectorRowStyle {
                    border: p.border.default,
                    muted_text: p.app.muted_foreground,
                    caption: self.theme.tokens.typography.text.caption,
                    mono_font: self.theme.tokens.typography.font.mono.family.clone(),
                    wrap_source: true,
                },
                div().child(row.label.clone()).into_any_element(),
                div().child(row.value.clone()).into_any_element(),
                row.source.clone(), 200.0, index > 0,
            )))
    }
}

/// Delegates painting unchanged to the SDK and passes its actual model to shared inspection.
pub struct InspectedSwitchTemplate {
    theme: Arc<dyn SwitchTheme>,
    template: ThemedSwitchTemplate,
    inspector: Entity<SwitchInspector>,
    mode: ThemeMode,
}
impl InspectedSwitchTemplate {
    pub fn new(theme: &NativeTheme, inspector: Entity<SwitchInspector>) -> Self {
        let switch_theme = theme.switch_theme();
        Self {
            template: ThemedSwitchTemplate::new(switch_theme.clone()),
            theme: switch_theme,
            inspector,
            mode: theme.mode,
        }
    }
}
impl TemplateWithModifiers<ButtonRenderModel<SwitchData>> for InspectedSwitchTemplate {
    fn modifiers(&self) -> &[Modifier<ButtonRenderModel<SwitchData>>] {
        &[]
    }
}
impl ButtonTemplate<SwitchData> for InspectedSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<SwitchData>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        self.inspector.update(cx, |inspector, cx| {
            inspector.inspect(self.theme.as_ref(), model, window.scale_factor(), self.mode, cx)
        });
        self.template.render(model, window, cx)
    }
}
