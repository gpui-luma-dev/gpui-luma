use std::sync::Arc;

use gpui::{AppContext, Bounds, Entity, Pixels, SharedString, div, prelude::*, px};
use lucide_svg_static::Icon as LucideIcon;

use super::{ControlFocusState, MenuPath, PopupMenu, PopupMenuTemplate, default_popup_menu_template};
use crate::theme::InteractionState;
use super::template::modified_popup_menu_template;
use crate::infra::menu_item::MenuItem;
use crate::controls::floating_menu::FloatingMenuHighlight;
use crate::infra::icon::DisclosureIcons;
use crate::motion::overlay_presence::OverlayPresence;
use crate::infra::presenter::{ControlPresenter, HasPresenter};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuPlacement {
    #[default]
    Smart,
    BelowStart,
    AboveStart,
    /// Opens to the right of the trigger, bottom edges aligned (sidebar account menus).
    RightEnd,
    CenteredOnTrigger,
}

/// Trigger chrome aligned with command button variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopupMenuTriggerStyle {
    Primary,
    Secondary,
    #[default]
    Outline,
    Ghost,
}

/// Owned snapshot passed to trigger face presenters.
#[derive(Clone, Debug)]
pub struct PopupMenuTriggerModel {
    pub id: SharedString,
    pub label: SharedString,
    pub open: bool,
    pub enabled: bool,
    pub state: InteractionState,
}

fn default_trigger_content() -> ControlPresenter<PopupMenuTriggerModel> {
    Arc::new(|model, _| div().flex_1().min_w(px(0.0)).truncate().child(model.label.clone()).into_any_element())
}

/// Presenter used by the icon-only face preset ([`PopupMenuBuilder::icon`] / [`PopupMenu::set_icon`](super::PopupMenu::set_icon)).
pub fn icon_content(icon: LucideIcon) -> ControlPresenter<PopupMenuTriggerModel> {
    Arc::new(move |_, _| crate::infra::icon::lucide_icon(icon, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0))
}

#[derive(Clone)]
pub struct PopupMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) content: ControlPresenter<PopupMenuTriggerModel>,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    pub(crate) tab_stop: bool,
    pub(crate) placement: PopupMenuPlacement,
    pub(crate) trigger_style: PopupMenuTriggerStyle,
    pub(crate) trigger_size: ControlSize,
    pub(crate) menu_size: ControlSize,
    /// Square icon-button chrome (no trailing end icon). Set by the icon face preset.
    pub(crate) icon_only: bool,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) end_icon: Option<LucideIcon>,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) full_width: bool,
    pub(crate) without_elevation: bool,
    pub(crate) split: bool,
    pub(crate) template: Arc<dyn PopupMenuTemplate>,
}

pub struct PopupMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub content: ControlPresenter<PopupMenuTriggerModel>,
    pub items: &'a [MenuItem],
    pub open: bool,
    pub disclosure_progress: f32,
    pub presence: OverlayPresence,
    pub submenu_transition: Option<(usize, f32)>,
    pub trigger_bounds: Option<Bounds<Pixels>>,
    pub placement: PopupMenuPlacement,
    pub trigger_style: PopupMenuTriggerStyle,
    pub trigger_size: ControlSize,
    pub menu_size: ControlSize,
    pub icon_only: bool,
    pub icon: Option<LucideIcon>,
    pub end_icon: Option<LucideIcon>,
    pub disclosure_icons: &'a DisclosureIcons,
    pub full_width: bool,
    pub without_elevation: bool,
    pub split: bool,
    pub trigger_radius_override: Option<f32>,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub highlight: Option<FloatingMenuHighlight>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: InteractionState,
}

pub struct PopupMenuBuilder {
    pub(crate) model: PopupMenuModel,
}

impl PopupMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: PopupMenuModel {
                label: id.clone(),
                id,
                content: default_trigger_content(),
                items: Vec::new(),
                enabled: true,
                tab_stop: true,
                placement: PopupMenuPlacement::Smart,
                trigger_style: PopupMenuTriggerStyle::default(),
                trigger_size: ControlSize::Md,
                menu_size: ControlSize::Md,
                icon_only: false,
                icon: None,
                end_icon: None,
                disclosure_icons: DisclosureIcons::new(LucideIcon::ChevronUp, LucideIcon::ChevronDown),
                full_width: false,
                without_elevation: false,
                split: false,
                template: default_popup_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self.model.content = default_trigger_content();
        self.model.icon_only = false;
        self.model.icon = None;
        self
    }

    /// Icon-only face preset: [`icon_content`] + square chrome (`icon_only`).
    ///
    /// Not a parallel field beside the presenter — use [`HasPresenter::content`] / [`PopupMenu::set_presenter`](super::PopupMenu::set_presenter)
    /// for a custom face. Mirrors [`PopupMenu::set_icon`](super::PopupMenu::set_icon).
    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.model.content = icon_content(icon);
        self.model.icon_only = true;
        self.model.icon = Some(icon);
        self
    }

    /// Trailing adornment on a labeled trigger (distinct from the icon face preset).
    pub fn end_icon(mut self, icon: LucideIcon) -> Self {
        self.model.end_icon = Some(icon);
        self
    }

    /// Sets the open and closed trigger icons, including SVG-path sources.
    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons = icons;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.model.full_width = full_width;
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    pub fn placement(mut self, placement: PopupMenuPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn trigger_style(mut self, style: PopupMenuTriggerStyle) -> Self {
        self.model.trigger_style = style;
        self
    }

    pub fn ghost(self) -> Self {
        self.trigger_style(PopupMenuTriggerStyle::Ghost)
    }

    /// Uses the trigger as the secondary face of a single-focus split button.
    pub fn split(mut self) -> Self {
        self.model.split = true;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.trigger_size = size;
        self
    }

    pub fn menu_size(mut self, size: ControlSize) -> Self {
        self.model.menu_size = size;
        self
    }

    pub fn without_elevation(mut self) -> Self {
        self.model.without_elevation = true;
        self
    }

    pub fn template(mut self, template: Arc<dyn PopupMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &PopupMenuRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_popup_menu_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<PopupMenu> {
        cx.new(|cx| PopupMenu::from_builder(self, cx))
    }
}

impl HasPresenter<PopupMenuTriggerModel> for PopupMenuBuilder {
    fn set_presenter(&mut self, content: ControlPresenter<PopupMenuTriggerModel>) {
        self.model.content = content;
        self.model.icon_only = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_popup_menu_template();
        let builder = PopupMenuBuilder::new("popup-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
