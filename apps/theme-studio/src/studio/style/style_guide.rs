use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, IntoElement, KeyDownEvent, MouseButton, Pixels, Render, ScrollHandle,
    ScrollWheelEvent, Window, div, point, prelude::*, px,
};
use gpui_luma::controls::navigation_sidebar::{NavNode, NavigationSidebar};
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::{declare_form};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};
use lucide_icons::Icon as LucideIcon;

use crate::studio::style::sections;
use crate::studio::style::shared::callout::render_sparse_catalog_callout;
use crate::studio::style::sticky_section_heading::{
    StickySectionHeadingTracker, render_sticky_section_heading_lane, with_sticky_heading_tracker,
};

const TRACKER_HEIGHT: f32 = 280.0;
const TRACKER_INSET: f32 = 22.0;
const TRACKER_THUMB_HEIGHT: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StyleGuideSection {
    Sidebar,
    Feedback,
    Buttons,
    IconButtons,
    Checkbox,
    Radio,
    Switch,
    Toggle,
    Menus,
    Pager,
    Selectors,
    Tabs,
    Scrollbar,
    Slider,
    TextField,
    TextArea,
    Listbox,
    ListView,
    TreeView,
    Accordion,
    Typography,
}

impl StyleGuideSection {
    const ALL: [Self; 21] = [
        Self::Accordion,
        Self::Buttons,
        Self::Checkbox,
        Self::Feedback,
        Self::IconButtons,
        Self::ListView,
        Self::Listbox,
        Self::Menus,
        Self::Pager,
        Self::Radio,
        Self::Scrollbar,
        Self::Selectors,
        Self::Sidebar,
        Self::Slider,
        Self::Switch,
        Self::Tabs,
        Self::TextArea,
        Self::TextField,
        Self::Toggle,
        Self::TreeView,
        Self::Typography,
    ];
}

declare_form! {
    pub struct StyleGuidePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {
            scroll_handle: ScrollHandle = ScrollHandle::new(),
            sticky_heading_tracker: Rc<RefCell<StickySectionHeadingTracker>> =
                Rc::new(RefCell::new(StickySectionHeadingTracker::default())),
            last_scroll_offset: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            last_max_scroll: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            sidebar_preview: Option<Entity<NavigationSidebar>> = None,
            buttons_preview_tabs: Option<Entity<TabsNavigation>> = None,
            icon_buttons_preview_tabs: Option<Entity<TabsNavigation>> = None,
            checkbox_preview_tabs: Option<Entity<TabsNavigation>> = None,
            radio_preview_tabs: Option<Entity<TabsNavigation>> = None,
            switch_preview_tabs: Option<Entity<TabsNavigation>> = None,
            switch_customization_preview: Option<Entity<sections::switch::customization::SwitchCustomizationPreview>> =
                None,
            toggles_preview_tabs: Option<Entity<TabsNavigation>> = None,
            menus_preview_tabs: Option<Entity<TabsNavigation>> = None,
            pager_preview: Option<Entity<sections::pager::PagerPreview>> = None,
            selectors_preview_tabs: Option<Entity<TabsNavigation>> = None,
            scrollbar_preview_tabs: Option<Entity<TabsNavigation>> = None,
            textfield_preview_tabs: Option<Entity<TabsNavigation>> = None,
            slider_preview_tabs: Option<Entity<TabsNavigation>> = None,
            slider_customization_preview: Option<Entity<sections::slider::customization::SliderCustomizationPreview>> =
                None,
            listbox_preview_tabs: Option<Entity<TabsNavigation>> = None,
            listbox_preview: Option<Entity<sections::listbox::ListboxPreview>> = None,
            list_view_preview_tabs: Option<Entity<TabsNavigation>> = None,
            list_view_preview: Option<Entity<sections::list_view::ListViewPreview>> = None,
            tree_view_preview_tabs: Option<Entity<TabsNavigation>> = None,
            tree_view_preview: Option<Entity<sections::tree_view::TreeViewPreview>> = None,
            accordion_preview_tabs: Option<Entity<TabsNavigation>> = None,
            accordion_preview: Option<Entity<sections::accordion::AccordionPreview>> = None,
        }
    }
}

impl StyleGuidePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.sync_sidebar_preview(cx);
        self.sync_buttons_preview_tabs(cx);
        self.sync_icon_buttons_preview_tabs(cx);
        self.sync_checkbox_preview_tabs(cx);
        self.sync_radio_preview_tabs(cx);
        self.sync_switch_preview_tabs(cx);
        self.sync_switch_customization_preview(cx);
        self.sync_toggles_preview_tabs(cx);
        self.sync_menus_preview_tabs(cx);
        self.sync_pager_preview(cx);
        self.sync_selectors_preview_tabs(cx);
        self.sync_scrollbar_preview_tabs(cx);
        self.sync_textfield_preview_tabs(cx);
        self.sync_listbox_preview_tabs(cx);
        self.sync_listbox_preview(cx);
        self.sync_list_view_preview_tabs(cx);
        self.sync_list_view_preview(cx);
        self.sync_tree_view_preview_tabs(cx);
        self.sync_tree_view_preview(cx);
        self.sync_accordion_preview_tabs(cx);
        self.sync_accordion_preview(cx);
        self.sync_slider_preview_tabs(cx);
        self.sync_slider_customization_preview(cx);
        self.sticky_heading_tracker.borrow_mut().reset();
        cx.notify();
    }

    fn set_vertical_offset(&self, value: f32, cx: &mut Context<Self>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    fn scroll_vertical_by(&self, delta: Pixels, cx: &mut Context<Self>) -> bool {
        let current = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let target = (current + delta.as_f32()).clamp(0.0, max);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    fn sidebar_preview(&mut self, cx: &mut Context<Self>) -> Entity<NavigationSidebar> {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            return sidebar;
        }

        let sidebar = self
            .look
            .navigation_sidebar("theme-studio-style-guide-sidebar-preview")
            .title("Properties")
            .subtitle("Task workspace")
            .collapsible(true)
            .selected_id(INITIAL_SIDEBAR_SELECTION_ID)
            .items(style_guide_sidebar_nodes())
            .footer_nodes(style_guide_sidebar_footer_nodes())
            .spawn(cx);

        self.sidebar_preview = Some(sidebar.clone());
        sidebar
    }

    fn sync_sidebar_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            let look = self.look.clone();
            sidebar.update(cx, move |sidebar, cx| {
                sidebar.set_template(look.navigation_sidebar_template(), cx);
                sidebar.set_scrollbar_template(look.scrollbar_template(), cx);
                sidebar.set_items(style_guide_sidebar_nodes(), cx);
                sidebar.set_footer_nodes(style_guide_sidebar_footer_nodes(), cx);
                sidebar.set_selected_id(INITIAL_SIDEBAR_SELECTION_ID, cx);
            });
        }
    }

    fn sync_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-buttons-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-icon-buttons-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.icon_buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_toggles_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.toggles_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn toggles_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.toggles_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-toggles-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.toggles_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_checkbox_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.checkbox_preview_tabs.clone(), cx);
    }

    fn checkbox_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.checkbox_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-checkbox-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.checkbox_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_radio_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.radio_preview_tabs.clone(), cx);
    }

    fn radio_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.radio_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-radio-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.radio_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_switch_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.switch_preview_tabs.clone(), cx);
    }

    fn switch_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.switch_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-switch-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
                TabsNavigationItem::new("customization").label("Customization"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.switch_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn switch_customization_preview(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Entity<sections::switch::customization::SwitchCustomizationPreview> {
        if let Some(preview) = self.switch_customization_preview.clone() {
            return preview;
        }

        let preview =
            cx.new(|cx| sections::switch::customization::SwitchCustomizationPreview::new(cx, self.look.clone()));
        self.switch_customization_preview = Some(preview.clone());
        preview
    }

    fn sync_switch_customization_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.switch_customization_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_menus_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.menus_preview_tabs.clone(), cx);
    }

    fn menus_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.menus_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-menus-preview-tabs")
            .items([
                TabsNavigationItem::new("menu-trigger").label("Menu Trigger"),
                TabsNavigationItem::new("trigger-sizes").label("Trigger Sizes"),
                TabsNavigationItem::new("floating-menu").label("Floating Menu"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("menu-trigger")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.menus_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn pager_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::pager::PagerPreview> {
        if let Some(preview) = self.pager_preview.clone() {
            return preview;
        }

        let preview = cx.new(|cx| sections::pager::PagerPreview::new(cx, self.look.clone()));
        self.pager_preview = Some(preview.clone());
        preview
    }

    fn sync_pager_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.pager_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_scrollbar_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.scrollbar_preview_tabs.clone(), cx);
    }

    fn sync_textfield_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.textfield_preview_tabs.clone(), cx);
    }

    fn sync_selectors_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.selectors_preview_tabs.clone(), cx);
    }

    fn selectors_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.selectors_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-selectors-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.selectors_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn scrollbar_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.scrollbar_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-scrollbar-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.scrollbar_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn textfield_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.textfield_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-textfield-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.textfield_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_slider_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.slider_preview_tabs.clone(), cx);
    }

    fn slider_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.slider_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-slider-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
                TabsNavigationItem::new("customization").label("Customization"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.slider_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn slider_customization_preview(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Entity<sections::slider::customization::SliderCustomizationPreview> {
        if let Some(preview) = self.slider_customization_preview.clone() {
            return preview;
        }

        let preview =
            cx.new(|cx| sections::slider::customization::SliderCustomizationPreview::new(cx, self.look.clone()));
        self.slider_customization_preview = Some(preview.clone());
        preview
    }

    fn sync_slider_customization_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.slider_customization_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| {
                preview.sync_look(look, cx);
            });
        }
    }

    fn sync_preview_tabs(&self, tabs: Option<Entity<TabsNavigation>>, cx: &mut Context<Self>) {
        if let Some(tabs) = tabs {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn spawn_template_sizes_tabs(&self, id: &'static str, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        let tabs = self
            .look
            .tabs_navigation(id)
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        tabs
    }

    fn listbox_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.listbox_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("theme-studio-listbox-preview-tabs", cx);
        self.listbox_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_listbox_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.listbox_preview_tabs.clone(), cx);
    }

    fn listbox_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::listbox::ListboxPreview> {
        if let Some(preview) = self.listbox_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::listbox::ListboxPreview::new(cx, self.look.clone()));
        self.listbox_preview = Some(preview.clone());
        preview
    }

    fn sync_listbox_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.listbox_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn list_view_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.list_view_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("theme-studio-list-view-preview-tabs", cx);
        self.list_view_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_list_view_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.list_view_preview_tabs.clone(), cx);
    }

    fn list_view_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::list_view::ListViewPreview> {
        if let Some(preview) = self.list_view_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::list_view::ListViewPreview::new(cx, self.look.clone()));
        self.list_view_preview = Some(preview.clone());
        preview
    }

    fn sync_list_view_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.list_view_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn tree_view_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.tree_view_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("theme-studio-tree-view-preview-tabs", cx);
        self.tree_view_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_tree_view_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.tree_view_preview_tabs.clone(), cx);
    }

    fn tree_view_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::tree_view::TreeViewPreview> {
        if let Some(preview) = self.tree_view_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::tree_view::TreeViewPreview::new(cx, self.look.clone()));
        self.tree_view_preview = Some(preview.clone());
        preview
    }

    fn sync_tree_view_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.tree_view_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }

    fn accordion_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.accordion_preview_tabs.clone() {
            return tabs;
        }
        let tabs = self.spawn_template_sizes_tabs("theme-studio-accordion-preview-tabs", cx);
        self.accordion_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_accordion_preview_tabs(&mut self, cx: &mut Context<Self>) {
        self.sync_preview_tabs(self.accordion_preview_tabs.clone(), cx);
    }

    fn accordion_preview(&mut self, cx: &mut Context<Self>) -> Entity<sections::accordion::AccordionPreview> {
        if let Some(preview) = self.accordion_preview.clone() {
            return preview;
        }
        let preview = cx.new(|cx| sections::accordion::AccordionPreview::new(cx, self.look.clone()));
        self.accordion_preview = Some(preview.clone());
        preview
    }

    fn sync_accordion_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = self.accordion_preview.clone() {
            let look = self.look.clone();
            preview.update(cx, move |preview, cx| preview.sync_look(look, cx));
        }
    }
}

impl Render for StyleGuidePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.sidebar_preview(cx);
        let _ = self.buttons_preview_tabs(cx);
        let _ = self.icon_buttons_preview_tabs(cx);
        let _ = self.checkbox_preview_tabs(cx);
        let _ = self.radio_preview_tabs(cx);
        let _ = self.switch_preview_tabs(cx);
        let _ = self.switch_customization_preview(cx);
        let _ = self.toggles_preview_tabs(cx);
        let _ = self.menus_preview_tabs(cx);
        let _ = self.pager_preview(cx);
        let _ = self.selectors_preview_tabs(cx);
        let _ = self.scrollbar_preview_tabs(cx);
        let _ = self.textfield_preview_tabs(cx);
        let _ = self.slider_preview_tabs(cx);
        let _ = self.slider_customization_preview(cx);
        let _ = self.listbox_preview_tabs(cx);
        let _ = self.listbox_preview(cx);
        let _ = self.list_view_preview_tabs(cx);
        let _ = self.list_view_preview(cx);
        let _ = self.tree_view_preview_tabs(cx);
        let _ = self.tree_view_preview(cx);
        let _ = self.accordion_preview_tabs(cx);
        let _ = self.accordion_preview(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let scroll_progress = self.scroll_progress();
            let scroll_handle = self.scroll_handle.clone();
            let last_scroll_offset = self.last_scroll_offset.clone();
            let last_max_scroll = self.last_max_scroll.clone();
            let sticky_heading_tracker = self.sticky_heading_tracker.clone();
            let scroll_y = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
            let sticky_snapshot = sticky_heading_tracker.borrow().snapshot(scroll_y);

            with_sticky_heading_tracker(sticky_heading_tracker.clone(), || {
                div()
                    .id("theme-studio-typography")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .bg(chrome.content_background)
                    .px(px(28.0))
                    .pt(px(12.0))
                    .pb(px(28.0))
                    .child(
                        div()
                            .w_full()
                            .min_h(px(0.0))
                            .flex_1()
                            .flex()
                            .items_stretch()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h(px(0.0))
                                    .child(
                                        div()
                                            .id("style-guide-content")
                                            .size_full()
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(0.0))
                                            .track_scroll(&self.scroll_handle)
                                            .on_prepaint(move |_, window: &mut Window, cx: &mut App| {
                                                let offset = (-scroll_handle.offset().y.as_f32()).max(0.0);
                                                let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                                                let offset_changed = (last_scroll_offset.get() - offset).abs() > 0.5;
                                                let max_changed = (last_max_scroll.get() - max_scroll).abs() > 0.5;

                                                if offset_changed || max_changed {
                                                    last_scroll_offset.set(offset);
                                                    last_max_scroll.set(max_scroll);
                                                    cx.notify(window.current_view());
                                                }
                                            })
                                            .child(
                                                div().w_full().flex().justify_start().pb(px(12.0)).child(
                                                    div()
                                                        .w_full()
                                                        .max_w(px(980.0))
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(14.0))
                                                        .on_prepaint({
                                                            let sticky_heading_tracker = sticky_heading_tracker.clone();
                                                            move |bounds, window, cx| {
                                                                let changed = sticky_heading_tracker
                                                                    .borrow_mut()
                                                                    .set_content_origin_y(bounds.origin.y.as_f32());
                                                                if changed {
                                                                    cx.notify(window.current_view());
                                                                }
                                                            }
                                                        })
                                                        .when_some(
                                                            render_sparse_catalog_callout(self.look.as_ref()),
                                                            |panel, callout| panel.child(callout),
                                                        )
                                                        .children(StyleGuideSection::ALL.into_iter().map(|section| {
                                                            self.render_section_shell(section, window, cx)
                                                        })),
                                                ),
                                            ),
                                    )
                                    .child(render_sticky_section_heading_lane(
                                        sticky_snapshot,
                                        chrome.content_background,
                                        980.0,
                                    )),
                            )
                            .child(self.render_scroll_tracker(scroll_progress)),
                    )
            })
        })
    }
}

impl StyleGuidePanel {
    fn render_section_shell(
        &self,
        section: StyleGuideSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = self.render_section(section, window, cx);
        let shell = div().relative().w_full().child(content);

        if Self::section_allows_interaction(section) {
            return shell.on_scroll_wheel(cx.listener(Self::handle_section_overlay_scroll_wheel)).into_any_element();
        }

        shell
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_mouse_down(MouseButton::Right, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_scroll_wheel(cx.listener(Self::handle_section_overlay_scroll_wheel))
                    .on_key_down(cx.listener(Self::handle_section_overlay_key_down)),
            )
            .into_any_element()
    }

    fn section_allows_interaction(section: StyleGuideSection) -> bool {
        matches!(
            section,
            StyleGuideSection::Buttons
                | StyleGuideSection::IconButtons
                | StyleGuideSection::Checkbox
                | StyleGuideSection::Radio
                | StyleGuideSection::Switch
                | StyleGuideSection::Toggle
                | StyleGuideSection::Menus
                | StyleGuideSection::Pager
                | StyleGuideSection::Selectors
                | StyleGuideSection::Scrollbar
                | StyleGuideSection::Slider
                | StyleGuideSection::TextField
                | StyleGuideSection::Listbox
                | StyleGuideSection::ListView
                | StyleGuideSection::TreeView
                | StyleGuideSection::Accordion
        )
    }

    fn scroll_progress(&self) -> f32 {
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        if max_scroll <= 0.5 {
            return 0.0;
        }

        (-self.scroll_handle.offset().y.as_f32()).clamp(0.0, max_scroll) / max_scroll
    }

    fn render_section(&self, section: StyleGuideSection, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match section {
            StyleGuideSection::Sidebar => sections::sidebar::render_sidebar_template_section(
                self.sidebar_preview.clone().expect("sidebar preview"),
                self.look.as_ref(),
            ),
            StyleGuideSection::Feedback => {
                sections::feedback::render_feedback_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Buttons => sections::buttons::render_button_template_matrix_section(
                self.look.clone(),
                self.buttons_preview_tabs.clone().expect("buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::IconButtons => sections::icon_buttons::render_icon_button_template_matrix_section(
                self.look.clone(),
                self.icon_buttons_preview_tabs.clone().expect("icon buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Checkbox => sections::checkbox::render_checkbox_template_matrix_section(
                self.look.clone(),
                self.checkbox_preview_tabs.clone().expect("checkbox preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Radio => sections::radio::render_radio_template_matrix_section(
                self.look.clone(),
                self.radio_preview_tabs.clone().expect("radio preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Switch => sections::switch::render_switch_template_matrix_section(
                self.look.clone(),
                self.switch_preview_tabs.clone().expect("switch preview tabs"),
                self.switch_customization_preview.clone().expect("switch customization preview"),
                window,
                cx,
            ),
            StyleGuideSection::Toggle => sections::toggle::render_toggle_template_matrix_section(
                self.look.clone(),
                self.toggles_preview_tabs.clone().expect("toggles preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Menus => sections::menus::render_menu_template_state_section(
                self.look.clone(),
                self.menus_preview_tabs.clone().expect("menus preview tabs"),
                Arc::new(cx.listener(Self::handle_section_overlay_scroll_wheel)),
                window,
                cx,
            ),
            StyleGuideSection::Pager => sections::pager::render_pager_template_section(
                self.look.clone(),
                self.pager_preview.clone().expect("pager preview"),
            ),
            StyleGuideSection::Selectors => sections::selectors::render_selector_templates_section(
                self.look.clone(),
                self.selectors_preview_tabs.clone().expect("selectors preview tabs"),
                Arc::new(cx.listener(Self::handle_section_overlay_scroll_wheel)),
                window,
                cx,
            ),
            StyleGuideSection::Tabs => {
                sections::tabs::render_tabs_navigation_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Scrollbar => sections::scrollbar::render_scrollbar_template_section(
                self.look.clone(),
                self.scrollbar_preview_tabs.clone().expect("scrollbar preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Slider => sections::slider::render_slider_template_section(
                self.look.clone(),
                self.slider_preview_tabs.clone().expect("slider preview tabs"),
                self.slider_customization_preview.clone().expect("slider customization preview"),
                window,
                cx,
            ),
            StyleGuideSection::TextField => sections::textfield::render_textfield_template_section(
                self.look.clone(),
                self.textfield_preview_tabs.clone().expect("textfield preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::TextArea => {
                sections::textarea::render_textarea_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Listbox => sections::listbox::render_listbox_template_section(
                self.look.clone(),
                self.listbox_preview_tabs.clone().expect("listbox preview tabs"),
                self.listbox_preview.clone().expect("listbox preview"),
                cx,
            ),
            StyleGuideSection::ListView => sections::list_view::render_list_view_template_section(
                self.look.clone(),
                self.list_view_preview_tabs.clone().expect("list view preview tabs"),
                self.list_view_preview.clone().expect("list view preview"),
                cx,
            ),
            StyleGuideSection::TreeView => sections::tree_view::render_tree_view_template_section(
                self.look.clone(),
                self.tree_view_preview_tabs.clone().expect("tree view preview tabs"),
                self.tree_view_preview.clone().expect("tree view preview"),
                cx,
            ),
            StyleGuideSection::Accordion => sections::accordion::render_accordion_template_section(
                self.look.clone(),
                self.accordion_preview_tabs.clone().expect("accordion preview tabs"),
                self.accordion_preview.clone().expect("accordion preview"),
                cx,
            ),
            StyleGuideSection::Typography => sections::typography::render_typography_section(self.look.as_ref()),
        }
    }

    fn render_scroll_tracker(&self, progress: f32) -> AnyElement {
        let chrome = self.look.chrome();
        let track_span = TRACKER_HEIGHT - (TRACKER_INSET * 2.0) - TRACKER_THUMB_HEIGHT;
        let thumb_top = TRACKER_INSET + (track_span.max(0.0) * progress.clamp(0.0, 1.0));

        div()
            .id("style-guide-scroll-tracker-shell")
            .w(px(28.0))
            .flex_shrink_0()
            .child(
                div().w_full().h_full().flex().items_center().justify_center().child(
                    div()
                        .relative()
                        .w(px(12.0))
                        .h(px(TRACKER_HEIGHT))
                        .child(
                            div()
                                .absolute()
                                .top(px(TRACKER_INSET))
                                .bottom(px(TRACKER_INSET))
                                .left(px(5.0))
                                .w(px(2.0))
                                .rounded_full()
                                .bg(gpui::hsla(chrome.border.h, chrome.border.s, chrome.border.l, 0.9)),
                        )
                        .children(
                            StyleGuideSection::ALL
                                .into_iter()
                                .enumerate()
                                .map(|(index, _)| render_tracker_marker(index, chrome.muted_text)),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(thumb_top))
                                .left(px(0.0))
                                .w(px(12.0))
                                .h(px(TRACKER_THUMB_HEIGHT))
                                .rounded_full()
                                .bg(chrome.title_text),
                        ),
                ),
            )
            .into_any_element()
    }

    fn handle_section_overlay_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(px(40.0)).y.as_f32();
        if !delta.is_finite() || delta.abs() <= f32::EPSILON {
            return;
        }

        if self.scroll_vertical_by(px(-delta), cx) {
            window.prevent_default();
            cx.stop_propagation();
        }
    }

    fn handle_section_overlay_key_down(&mut self, _: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.prevent_default();
        cx.stop_propagation();
    }
}

fn render_tracker_marker(index: usize, color: gpui::Hsla) -> AnyElement {
    let count = StyleGuideSection::ALL.len();
    let usable_height = TRACKER_HEIGHT - (TRACKER_INSET * 2.0);
    let step = if count > 1 {
        usable_height / (count.saturating_sub(1) as f32)
    } else {
        0.0
    };
    let top = TRACKER_INSET + (index as f32 * step) - 2.0;

    div()
        .absolute()
        .top(px(top))
        .left(px(3.0))
        .size(px(6.0))
        .rounded_full()
        .bg(gpui::hsla(color.h, color.s, color.l, 0.85))
        .into_any_element()
}

const INITIAL_SIDEBAR_SELECTION_ID: &str = "dimensions";

fn style_guide_sidebar_nodes() -> Vec<NavNode> {
    vec![
        NavNode::section("pinned-label", "Pinned"),
        sidebar_leaf_node("summary", "Summary", Some(LucideIcon::Info), true),
        sidebar_leaf_node("tokens", "Design Tokens", Some(LucideIcon::Tags), true),
        NavNode::section("properties-label", "Properties"),
        NavNode::new("layout").label("Layout").icon(LucideIcon::Ruler).expanded(true).children([
            sidebar_leaf_node("position", "Position", None, true),
            sidebar_leaf_node(INITIAL_SIDEBAR_SELECTION_ID, "Dimensions", None, true),
            sidebar_leaf_node("constraints", "Constraints", None, true),
            sidebar_leaf_node("grid", "Grid", None, true),
        ]),
        NavNode::new("look").label("Look").icon(LucideIcon::Palette).expanded(true).children([
            sidebar_leaf_node("fill", "Fill", None, true),
            sidebar_leaf_node("stroke", "Stroke", None, true),
            sidebar_leaf_node("typography", "Typography", None, true),
            sidebar_leaf_node("effects", "Effects", None, true),
        ]),
        NavNode::new("behavior")
            .label("Behavior")
            .icon(LucideIcon::MousePointer2)
            .expanded(false)
            .children([
                sidebar_leaf_node("interactions", "Interactions", None, true),
                sidebar_leaf_node("conditions", "Conditions", None, true),
                sidebar_leaf_node("validation", "Validation", None, true),
                sidebar_leaf_node("data-binding", "Data Binding", None, false),
            ]),
    ]
}

fn style_guide_sidebar_footer_nodes() -> Vec<NavNode> {
    vec![
        sidebar_leaf_node("audit-log", "Audit Log", Some(LucideIcon::FileText), true),
        sidebar_leaf_node("reset-overrides", "Reset Overrides", Some(LucideIcon::RotateCcw), false),
    ]
}

fn sidebar_leaf_node(id: &'static str, label: &'static str, icon: Option<LucideIcon>, enabled: bool) -> NavNode {
    let mut node = NavNode::new(id).label(label).enabled(enabled);
    if let Some(icon) = icon {
        node = node.icon(icon);
    }
    node
}
