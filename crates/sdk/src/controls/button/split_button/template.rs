use std::sync::{Arc, OnceLock};
use gpui::{App, Div, Stateful, Window};
use crate::controls::popup_menu::{PopupMenuTemplate, ThemedPopupMenuTemplate, default_popup_menu_theme};
use super::{SplitButtonRenderModel, SplitButtonTemplateHandlers, SplitButtonTheme};

/// Specialization boundary. Geometry, menu placement and face paint are shared
/// with PopupMenu; implementations receive independent states in `split_state`.
pub trait SplitButtonTemplate: Send + Sync {
    fn render(
        &self,
        model: &SplitButtonRenderModel<'_>,
        handlers: SplitButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

pub struct ThemedSplitButtonTemplate {
    shared: ThemedPopupMenuTemplate,
}
impl ThemedSplitButtonTemplate {
    pub fn new(theme: Arc<dyn SplitButtonTheme>) -> Self {
        Self { shared: ThemedPopupMenuTemplate::new(theme) }
    }
}
impl SplitButtonTemplate for ThemedSplitButtonTemplate {
    fn render(
        &self,
        model: &SplitButtonRenderModel<'_>,
        handlers: SplitButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.shared.render(model, handlers, window, cx)
    }
}

pub fn default_split_button_template() -> Arc<dyn SplitButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SplitButtonTemplate>> = OnceLock::new();
    TEMPLATE
        .get_or_init(|| Arc::new(ThemedSplitButtonTemplate::new(default_popup_menu_theme())))
        .clone()
}

pub(super) struct EngineTemplate(pub Arc<dyn SplitButtonTemplate>);
impl PopupMenuTemplate for EngineTemplate {
    fn render(
        &self,
        model: &SplitButtonRenderModel<'_>,
        handlers: SplitButtonTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        self.0.render(model, handlers, window, cx)
    }
}
