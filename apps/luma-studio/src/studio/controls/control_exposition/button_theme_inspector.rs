use std::sync::Arc;

use gpui::Context;
use gpui_luma_look_shadcn::ShadcnLook;

use super::button_inspector_adapter::{ButtonInspectorAdapter, BUTTON_INSPECTOR_SPEC};
use super::theme_inspector::ThemeInspector;

pub type ButtonThemeInspector = ThemeInspector;

impl ThemeInspector {
    pub fn for_embedded_pane(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        ThemeInspector::for_embedded_pane_with_spec(look, &BUTTON_INSPECTOR_SPEC, ButtonInspectorAdapter::shared(), cx)
    }
}
