//! Bare sidebar navigation. Width and visibility belong to the host.
//!
//! ```no_run
//! use gpui::{Context, Entity};
//! use gpui_luma::controls::sidebar::SidebarBuilder;
//! use gpui_luma_look_shadcn::{Frame, ShadcnLook, Sidebar};
//!
//! fn standalone<M: 'static>(
//!     look: &ShadcnLook, content: SidebarBuilder, cx: &mut Context<M>,
//! ) -> Entity<gpui_luma::controls::frame::FrameControl> {
//!     let navigation = Sidebar::new("nav").look(look).sidebar(content).spawn(cx);
//!     Frame::sidebar("nav-frame").look(look).child(navigation).spawn(cx)
//! }
//! ```

use gpui::{Context, Entity, SharedString};
use gpui_luma::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use gpui_luma::controls::sidebar::{
    SidebarBuilder, SidebarContentBuilder, SidebarControlBuilder, SidebarFooterBuilder, SidebarGroupBuilder,
    SidebarHeaderBuilder, SidebarMenuBuilder, SidebarMenuItemBuilder, SidebarMenuSubBuilder, SidebarPresentation,
};

use crate::look::{ShadcnLook, resolve_look_from};

/// Builder in the guise of a sidebar control: Shadcn templates plus SDK options, until `.spawn(cx)`.
pub struct Sidebar {
    look: Option<ShadcnLook>,
    builder: SidebarControlBuilder,
}

impl Sidebar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { look: None, builder: gpui_luma::controls::sidebar::SidebarControl::new(id) }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn presentation(mut self, presentation: SidebarPresentation) -> Self {
        self.builder = self.builder.presentation(presentation);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.builder = self.builder.animated(animated);
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected_id(selected_id);
        self
    }

    pub fn sidebar(mut self, sidebar: SidebarBuilder) -> Self {
        self.builder = self.builder.sidebar(sidebar);
        self
    }

    pub fn overlay_scrollbar(mut self, overlay: bool) -> Self {
        self.builder = self.builder.overlay_scrollbar(overlay);
        self
    }

    pub fn scrollbar_placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.builder = self.builder.scrollbar_placement(placement);
        self
    }

    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.builder = self.builder.scrollbar_visibility(visibility);
        self
    }

    pub fn auto_hide_scrollbar(mut self, auto_hide: bool) -> Self {
        self.builder = self.builder.auto_hide_scrollbar(auto_hide);
        self
    }

    pub fn auto_hide_scrollbar_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.builder = self.builder.auto_hide_scrollbar_activate(activate);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<gpui_luma::controls::sidebar::SidebarControl> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        let theme = crate::tooltip_theme(&look);
        let entity = self
            .builder
            .panel_template(look.sidebar_panel_template())
            .scrollbar_template(look.scrollbar_template())
            .spawn(cx);
        entity.update(cx, |sidebar, cx| sidebar.set_tooltip_theme(theme, cx));
        entity
    }

    pub fn panel(id: impl Into<SharedString>) -> SidebarBuilder {
        gpui_luma::controls::sidebar::sidebar(id)
    }

    pub fn header() -> SidebarHeaderBuilder {
        gpui_luma::controls::sidebar::sidebar_header()
    }

    pub fn content() -> SidebarContentBuilder {
        gpui_luma::controls::sidebar::sidebar_content()
    }

    pub fn group() -> SidebarGroupBuilder {
        gpui_luma::controls::sidebar::sidebar_group()
    }

    pub fn menu(id: impl Into<SharedString>) -> SidebarMenuBuilder {
        gpui_luma::controls::sidebar::sidebar_menu(id)
    }

    pub fn menu_item(id: impl Into<SharedString>, label: impl Into<SharedString>) -> SidebarMenuItemBuilder {
        gpui_luma::controls::sidebar::sidebar_menu_item(id, label)
    }

    pub fn menu_sub() -> SidebarMenuSubBuilder {
        gpui_luma::controls::sidebar::sidebar_menu_sub()
    }

    pub fn footer() -> SidebarFooterBuilder {
        gpui_luma::controls::sidebar::sidebar_footer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_helpers_construct() {
        let _ = Sidebar::header().title("Properties");
        let _ = Sidebar::menu("pinned");
        let _ = Sidebar::menu_item("a", "A");
    }
}
