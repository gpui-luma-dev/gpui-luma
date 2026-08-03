use gpui_luma_look_shadcn_inspect::{
    AccordionContentInspectPalette, AccordionTriggerInspectPalette, ListBoxListInspectPalette,
    ListBoxRowInspectPalette, ListViewInspectPalette, ListViewRowInspectPalette,
    NavigationSidebarContainerInspectPalette, NavigationSidebarItemInspectPalette,
    NavigationSidebarSectionInspectPalette, PagerShellInspectPalette, ResizablePanelsInspectPalette,
    SplitViewInspectPalette, TabsNavigationItemInspectPalette, TabsNavigationListInspectPalette, ToolbarInspectPalette,
    TreeViewRowInspectPalette,
};

use super::provenance::color_row;
use super::schema::InspectColorRow;

pub fn listbox_list_color_rows(palette: &ListBoxListInspectPalette) -> Vec<InspectColorRow> {
    let rows = vec![
        color_row("background", &palette.background),
        color_row("border", &palette.border),
        color_row("divider", &palette.divider),
    ];
    rows
}

pub fn listbox_row_color_rows(palette: &ListBoxRowInspectPalette) -> Vec<InspectColorRow> {
    vec![color_row("background", &palette.background), color_row("label", &palette.label_color)]
}

pub fn list_view_surface_color_rows(palette: &ListViewInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("background", &palette.background),
        color_row("border", &palette.border),
        color_row("header background", &palette.header_background),
        color_row("header label", &palette.header_label_color),
    ]
}

pub fn list_view_row_color_rows(palette: &ListViewRowInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("background", &palette.background),
        color_row("label", &palette.label_color),
        color_row("divider", &palette.divider),
    ]
}

pub fn tree_view_row_color_rows(palette: &TreeViewRowInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = Vec::new();
    if let Some(background) = &palette.background {
        rows.push(color_row("background", background));
    }
    rows.push(color_row("foreground", &palette.foreground));
    rows.push(color_row("icon", &palette.icon_color));
    rows.push(color_row("chevron", &palette.chevron_color));
    rows
}

pub fn navigation_sidebar_container_color_rows(
    palette: &NavigationSidebarContainerInspectPalette,
) -> Vec<InspectColorRow> {
    vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
    ]
}

pub fn navigation_sidebar_section_color_rows(palette: &NavigationSidebarSectionInspectPalette) -> Vec<InspectColorRow> {
    vec![color_row("label", &palette.label_color)]
}

pub fn navigation_sidebar_item_color_rows(palette: &NavigationSidebarItemInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = Vec::new();
    if let Some(background) = &palette.background {
        rows.push(color_row("background", background));
    }
    rows.push(color_row("foreground", &palette.foreground));
    rows.push(color_row("icon", &palette.icon_color));
    rows
}

pub fn tabs_navigation_item_color_rows(palette: &TabsNavigationItemInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = vec![color_row("label", &palette.label_color)];
    if let Some(indicator) = &palette.indicator {
        rows.push(color_row("indicator", indicator));
    }
    rows
}

pub fn tabs_navigation_list_color_rows(palette: &TabsNavigationListInspectPalette) -> Vec<InspectColorRow> {
    palette
        .background
        .as_ref()
        .map(|background| vec![color_row("list background", background)])
        .unwrap_or_default()
}

pub fn toolbar_shell_color_rows(palette: &ToolbarInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("background", &palette.background),
        color_row("border", &palette.border),
        color_row("separator", &palette.separator),
    ]
}

pub fn pager_shell_color_rows(palette: &PagerShellInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("panel background", &palette.panel_background),
        color_row("border", &palette.border),
        color_row("body text", &palette.body_text),
        color_row("muted text", &palette.muted_text),
        color_row("selected background", &palette.selected_background),
        color_row("selected foreground", &palette.selected_foreground),
    ]
}

pub fn accordion_trigger_color_rows(palette: &AccordionTriggerInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = Vec::new();
    if let Some(background) = &palette.background {
        rows.push(color_row("background", background));
    }
    rows.push(color_row("foreground", &palette.foreground));
    rows.push(color_row("border", &palette.border_color));
    rows.push(color_row("icon", &palette.icon_color));
    rows.push(color_row("chevron", &palette.chevron_color));
    rows
}

pub fn accordion_content_color_rows(palette: &AccordionContentInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = Vec::new();
    if let Some(background) = &palette.background {
        rows.push(color_row("background", background));
    }
    rows.push(color_row("foreground", &palette.foreground));
    rows
}

pub fn resizable_panels_color_rows(palette: &ResizablePanelsInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("border", &palette.border),
        color_row("divider", &palette.divider),
        color_row("grip", &palette.grip),
        color_row("grip emphasis", &palette.grip_emphasis),
    ]
}

pub fn split_view_color_rows(palette: &SplitViewInspectPalette) -> Vec<InspectColorRow> {
    vec![color_row("separator", &palette.separator), color_row("separator hover", &palette.separator_hover)]
}
