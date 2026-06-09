//! Public color-table resolvers for downstream inspect tooling.
//!
//! Rule metadata lives in [`crate::stylesheet`] and is keyed by [`crate::embedded_stylesheet`].

pub use crate::controls::accordion::{
    resolve_accordion_content_colors, AccordionContentColorTable, resolve_accordion_trigger_colors,
    AccordionTriggerColorTable,
};
pub use crate::controls::autocomplete::{resolve_autocomplete_chrome_colors, AutocompleteChromeColorTable};
pub use crate::controls::button::{resolve_button_colors, ButtonColorPalette};
pub use crate::controls::checkbox::{resolve_checkbox_colors, CheckboxColorTable};
pub use crate::controls::control_group::{resolve_control_group_list_colors, ControlGroupListColorTable};
pub use crate::controls::floating_menu::{
    resolve_floating_menu_colors, resolve_ghost_trigger_colors, FloatingMenuColorTable, GhostTriggerColorTable,
};
pub use crate::controls::list_view::{
    resolve_list_view_row_colors, resolve_list_view_surface_colors, ListViewRowColorTable, ListViewSurfaceColorTable,
};
pub use crate::controls::listbox::{
    resolve_listbox_list_colors, resolve_listbox_row_colors, ListBoxListColorTable, ListBoxRowColorTable,
};
pub use crate::controls::navigation_sidebar::{
    resolve_navigation_sidebar_branch_colors, resolve_navigation_sidebar_container_colors,
    resolve_navigation_sidebar_item_colors, resolve_navigation_sidebar_section_colors,
    NavigationSidebarBranchColorTable, NavigationSidebarContainerColorTable, NavigationSidebarItemColorTable,
    NavigationSidebarSectionColorTable,
};
pub use crate::controls::progress::{resolve_progress_colors, ProgressColorTable};
pub use crate::controls::radio::{resolve_radio_colors, RadioColorTable};
pub use crate::controls::resizable_panels::{resolve_resizable_panels_colors, ResizablePanelsColorTable};
pub use crate::controls::scrollbar::{resolve_scrollbar_colors, ScrollbarColorTable};
pub use crate::controls::slider::{resolve_slider_colors, SliderColorTable};
pub use crate::controls::split_view::{resolve_split_view_colors, SplitViewColorTable};
pub use crate::controls::switch::{resolve_switch_colors, SwitchColorTable};
pub use crate::controls::tabs_navigation::{
    resolve_tabs_navigation_item_colors, resolve_tabs_navigation_list_colors, TabsNavigationItemColorTable,
    TabsNavigationListColorTable,
};
pub use crate::controls::textfield::{resolve_textfield_colors, TextFieldColorTable};
pub use crate::controls::tree_view::{resolve_tree_view_row_colors, TreeViewRowColorTable};
