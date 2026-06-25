use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Window, div, px, prelude::*};
use gpui_luma::DockPanel;
use gpui_luma::controls::overlay_window::OverlayWindowRenderModel;

use super::model::DialogRenderModel;

pub(in crate::gallery) trait DialogTemplate: Send + Sync {
    fn render(
        &self,
        model: &DialogRenderModel,
        overlay_model: &OverlayWindowRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}

pub(in crate::gallery) struct DefaultDialogTemplate;

pub(in crate::gallery) fn default_dialog_template() -> Arc<dyn DialogTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn DialogTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultDialogTemplate)).clone()
}

impl DialogTemplate for DefaultDialogTemplate {
    fn render(
        &self,
        model: &DialogRenderModel,
        overlay_model: &OverlayWindowRenderModel<'_>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let mut content = div().w_full().min_w_0().flex().flex_col().gap(px(14.0));

        if model.title.is_some() || model.header_end.is_some() {
            let mut header = DockPanel::new().last_child_fill(true);

            if let Some(header_end) = &model.header_end {
                header = header.right(header_end(overlay_model, window, cx));
            }

            if let Some(title) = &model.title {
                header = header.fill(div().w_full().min_w_0().text_xl().child(title.clone()));
            } else {
                header = header.fill(div().w_full().min_w_0());
            }

            content = content.child(header);
        }

        if let Some(body) = &model.body {
            content = content.child(div().w_full().min_w_0().child(body(overlay_model, window, cx)));
        }

        if let Some(footer) = &model.footer {
            content = content.child(div().w_full().min_w_0().pt(px(4.0)).child(footer(overlay_model, window, cx)));
        }

        content.into_any_element()
    }
}
