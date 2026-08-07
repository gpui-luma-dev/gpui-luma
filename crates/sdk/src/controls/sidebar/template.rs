use std::sync::Arc;

use gpui::{AnyElement, Div, IntoElement, SharedString, Stateful, div, prelude::*, px};

use super::model::SidebarPaneRender;
use super::theme::{SidebarCollapsible, SidebarVariant};

/// Render snapshot for the sidebar shell (panel + inset).
pub struct SidebarRenderModel<'a> {
    pub id: &'a SharedString,
    pub open: bool,
    pub collapsible: SidebarCollapsible,
    pub variant: SidebarVariant,
    pub enabled: bool,
    pub has_inset: bool,
}

pub trait SidebarTemplate: Send + Sync {
    fn render(&self, model: SidebarRenderModel<'_>, panel: AnyElement, inset: Option<AnyElement>) -> Stateful<Div>;
}

pub struct DefaultSidebarTemplate;

impl SidebarTemplate for DefaultSidebarTemplate {
    fn render(&self, model: SidebarRenderModel<'_>, panel: AnyElement, inset: Option<AnyElement>) -> Stateful<Div> {
        let show_panel = match model.collapsible {
            SidebarCollapsible::Offcanvas => model.open,
            SidebarCollapsible::None => true,
            SidebarCollapsible::Icon | SidebarCollapsible::Responsive => true,
        };

        match inset {
            // Panel-only hosts (e.g. studio preview) must fill parent bounds like a bare
            // SidebarPanelEngine — avoid a shrink-wrapped flex_none chrome wrapper.
            None => {
                let mut root = div()
                    .id(SharedString::from(format!("{}-shell", model.id)))
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0));
                if show_panel {
                    root = root
                        .child(div().id(SharedString::from(format!("{}-panel", model.id))).size_full().child(panel));
                }
                root
            }
            Some(inset) => {
                let mut root = div()
                    .id(SharedString::from(format!("{}-shell", model.id)))
                    .flex()
                    .flex_row()
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0));

                if show_panel {
                    root = root.child(
                        div().id(SharedString::from(format!("{}-panel", model.id))).flex_none().h_full().child(panel),
                    );
                }

                root.child(
                    div()
                        .id(SharedString::from(format!("{}-inset", model.id)))
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(0.0))
                        .min_h(px(0.0))
                        .h_full()
                        .child(inset),
                )
            }
        }
    }
}

pub fn default_sidebar_template() -> Arc<dyn SidebarTemplate> {
    Arc::new(DefaultSidebarTemplate)
}

pub fn render_inset_column(
    header: Option<&SidebarPaneRender>,
    content: Option<&SidebarPaneRender>,
    footer: Option<&SidebarPaneRender>,
) -> AnyElement {
    let mut column = div().flex().flex_col().size_full().min_w(px(0.0)).min_h(px(0.0));

    if let Some(header) = header {
        column = column.child(div().flex_none().w_full().child(header()));
    }
    if let Some(content) = content {
        column = column.child(div().flex_1().min_h(px(0.0)).w_full().child(content()));
    }
    if let Some(footer) = footer {
        column = column.child(div().flex_none().w_full().child(footer()));
    }

    column.into_any_element()
}
