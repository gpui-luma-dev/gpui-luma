use std::sync::Arc;

use gpui::prelude::*;
use luma::controls::sidebar::{
    DefaultSidebarTemplate, SidebarPanelTemplate, SidebarRenderModel, SidebarTemplate, SidebarTheme,
    ThemedSidebarPanelTemplate,
};
use luma::theme::{ControlSize, InteractionState};

use super::floating_menu::floating_menu_theme;
use crate::controls::sidebar::{sidebar_branch_look, sidebar_container_look, sidebar_item_look, sidebar_section_look};
use crate::look::ShadcnLook;

struct ShadcnSidebarTheme {
    theme: ShadcnLook,
}

impl SidebarTheme for ShadcnSidebarTheme {
    fn resolve_container(&self) -> luma::controls::sidebar::SidebarContainerLook {
        let tokens = self.theme.mode_tokens();
        sidebar_container_look(tokens.as_ref())
    }

    fn resolve_section(&self) -> luma::controls::sidebar::SidebarSectionLook {
        sidebar_section_look(&self.theme)
    }

    fn resolve_branch(&self, state: InteractionState, size: ControlSize) -> luma::controls::sidebar::SidebarItemLook {
        sidebar_branch_look(&self.theme, state, size)
    }

    fn resolve_item(
        &self,
        selected: bool,
        state: InteractionState,
        size: ControlSize,
    ) -> luma::controls::sidebar::SidebarItemLook {
        sidebar_item_look(&self.theme, selected, state, size)
    }

    fn metrics(&self) -> luma::theme::MetricTokens {
        self.theme.mode_tokens().metrics
    }
}

pub fn sidebar_panel_template(theme: ShadcnLook) -> Arc<dyn SidebarPanelTemplate> {
    let menu_theme = floating_menu_theme(theme.clone());
    Arc::new(ThemedSidebarPanelTemplate::new_with_floating_menu_theme(
        Arc::new(ShadcnSidebarTheme { theme: theme.clone() }),
        menu_theme,
    ))
}

pub fn sidebar_template(theme: ShadcnLook) -> Arc<dyn SidebarTemplate> {
    Arc::new(ThemedSidebarTemplate { theme: theme.clone() })
}

struct ThemedSidebarTemplate {
    theme: ShadcnLook,
}

impl SidebarTemplate for ThemedSidebarTemplate {
    fn render(
        &self,
        model: SidebarRenderModel<'_>,
        panel: gpui::AnyElement,
        inset: Option<gpui::AnyElement>,
    ) -> gpui::Stateful<gpui::Div> {
        if inset.is_none() {
            return DefaultSidebarTemplate.render(model, panel, None);
        }

        let show_panel = match model.collapsible {
            luma::controls::sidebar::SidebarCollapsible::Offcanvas => model.open,
            luma::controls::sidebar::SidebarCollapsible::None
            | luma::controls::sidebar::SidebarCollapsible::Icon
            | luma::controls::sidebar::SidebarCollapsible::Responsive => true,
        };
        let mut root = gpui::div()
            .id(gpui::SharedString::from(format!("{}-shell", model.id)))
            .flex()
            .flex_row()
            .size_full()
            .min_w(gpui::px(0.0))
            .min_h(gpui::px(0.0));
        if show_panel {
            root = root.child(
                gpui::div()
                    .id(gpui::SharedString::from(format!("{}-panel", model.id)))
                    .flex_none()
                    .h_full()
                    .child(panel),
            );
        }
        if let Some(inset) = inset {
            let radius = self.theme.mode_tokens().metrics.radius.lg;
            root = root.child(
                gpui::div()
                    .id(gpui::SharedString::from(format!("{}-inset", model.id)))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(gpui::px(0.0))
                    .min_h(gpui::px(0.0))
                    .h_full()
                    .rounded(gpui::px(radius))
                    .overflow_hidden()
                    .bg(self.theme.chrome().content_background)
                    .child(inset),
            );
        }
        root
    }
}

pub fn sidebar_theme(theme: ShadcnLook) -> Arc<dyn SidebarTheme> {
    Arc::new(ShadcnSidebarTheme { theme: theme.clone() })
}
