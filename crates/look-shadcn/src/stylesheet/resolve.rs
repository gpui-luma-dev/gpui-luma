use std::collections::HashMap;

use gpui::{FontWeight, px};
use gpui_luma::controls::sidebar::SidebarMetricScale;
use gpui_luma::theme::{ControlSize, InteractionLayer, LumaTextStyle, MetricTokens};

use crate::controls::ShadcnButtonStyle;
use crate::provenance::{LookResolver, ResolvedColor};

use super::config::{
    AccordionContentColorRule, AccordionTriggerColorRule, AutocompleteChromeColorRule, BadgeColorRule, ButtonColorRule,
    ButtonMetricsRule, CardColorRule, CheckboxColorRule, ControlGroupListColorRule, FloatingMenuSurfaceColorRule,
    FloatingMenuTriggerColorRule, ListboxListColorRule, ListboxRowColorRule, ListViewRowColorRule,
    ListViewSurfaceColorRule, SidebarBranchColorRule, SidebarContainerColorRule, SidebarItemColorRule,
    SidebarMetricsRule, SidebarSectionColorRule, ProgressColorRule, ProgressMetricsRule, RadioColorRule,
    ResizablePanelsColorRule, ScrollbarMetricsRule, SliderMetricsRule, SliderColorRule, SplitViewColorRule,
    StepperMetricsRule, SwitchColorRule, SwitchMetricsRule, ScrollbarColorRule, TabsNavigationItemColorRule,
    TabsNavigationListColorRule, TextfieldColorRule, TreeViewRowColorRule, TypographyRule,
};

/// Context for resolving derived stylesheet tokens (`@outline_layer`, `@action_layer`, etc.).
#[derive(Clone, Copy, Debug)]
pub struct ResolveContext {
    pub style: ShadcnButtonStyle,
    pub layer: InteractionLayer,
    pub disabled: bool,
}

impl Default for ResolveContext {
    fn default() -> Self {
        Self { style: ShadcnButtonStyle::Primary, layer: InteractionLayer::Default, disabled: false }
    }
}

/// Resolved field values for a single color rule, used for `@field` interpolation.
#[derive(Clone, Debug, Default)]
pub struct ResolvedFields {
    colors: HashMap<String, ResolvedColor>,
}

impl ResolvedFields {
    pub fn insert(&mut self, field: impl Into<String>, color: ResolvedColor) {
        self.colors.insert(field.into(), color);
    }

    pub fn get(&self, field: &str) -> Option<&ResolvedColor> {
        self.colors.get(field)
    }
}

pub fn resolve_color_ref(
    resolver: &LookResolver<'_>,
    raw: &str,
    fields: &ResolvedFields,
) -> anyhow::Result<ResolvedColor> {
    resolve_stylesheet_color(resolver, raw, fields, &ResolveContext::default())
}

pub fn resolve_stylesheet_color(
    resolver: &LookResolver<'_>,
    raw: &str,
    fields: &ResolvedFields,
    ctx: &ResolveContext,
) -> anyhow::Result<ResolvedColor> {
    let raw = raw.trim();
    if let Some(field) = raw.strip_prefix('@') {
        if let Some(color) = fields.get(field) {
            return Ok(color.clone());
        }
        return resolve_derived_token(resolver, field, ctx);
    }
    if let Some(inner) = parse_first_fn(raw) {
        return resolver.resolve_first_decl(&inner);
    }
    if let Some(inner) = parse_first_layer_fn(raw) {
        return resolver.resolve_first_layer_decl(&inner, ctx.layer);
    }
    resolver.resolve_decl(raw)
}

pub fn resolve_optional_stylesheet_color(
    resolver: &LookResolver<'_>,
    raw: Option<&str>,
    fields: &ResolvedFields,
    ctx: &ResolveContext,
) -> anyhow::Result<Option<ResolvedColor>> {
    raw.map(|value| resolve_stylesheet_color(resolver, value, fields, ctx)).transpose()
}

fn resolve_derived_token(
    resolver: &LookResolver<'_>,
    token: &str,
    ctx: &ResolveContext,
) -> anyhow::Result<ResolvedColor> {
    match token {
        "outline_layer" => resolver.resolve_outline_layer_decl(ctx.layer),
        "action_layer" => resolver.resolve_action_layer_decl(ctx.style, ctx.layer),
        "action_foreground" => resolver.resolve_action_foreground_decl(ctx.style),
        "action_default" => resolver.resolve_action_layer_decl(ctx.style, InteractionLayer::Default),
        "primary_layer" => resolver.resolve_action_layer_decl(ShadcnButtonStyle::Primary, ctx.layer),
        "primary_default" => resolver.resolve_action_layer_decl(ShadcnButtonStyle::Primary, InteractionLayer::Default),
        "darken_border" => resolver.resolve_darken_border_decl(0.08),
        "accent_whisper_40" => resolver.resolve_accent_whisper_decl(40),
        "accent_whisper_pressed_40" => resolver.resolve_accent_whisper_pressed_decl(40),
        "label" => resolver.resolve_label_decl(ctx.disabled),
        other => anyhow::bail!("unknown stylesheet field reference `@{other}`"),
    }
}

fn parse_first_fn(raw: &str) -> Option<Vec<&str>> {
    let inner = raw.strip_prefix("first(")?.strip_suffix(')')?;
    Some(inner.split(',').map(str::trim).collect())
}

fn parse_first_layer_fn(raw: &str) -> Option<Vec<&str>> {
    let inner = raw.strip_prefix("first_layer(")?.strip_suffix(')')?;
    Some(inner.split(',').map(str::trim).collect())
}

pub fn resolve_button_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ButtonColorRule,
) -> anyhow::Result<ResolvedButtonColors> {
    let mut fields = ResolvedFields::default();

    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());

    let foreground = resolve_color_ref(resolver, &rule.foreground, &fields)?;
    fields.insert("foreground", foreground.clone());

    let border = match &rule.border {
        Some(value) => Some(resolve_color_ref(resolver, value, &fields)?),
        None => None,
    };

    Ok(ResolvedButtonColors { background, foreground, border })
}

pub fn resolve_progress_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ProgressColorRule,
) -> anyhow::Result<ResolvedProgressColors> {
    Ok(ResolvedProgressColors {
        track_color: resolve_color_ref(resolver, &rule.track_color, &ResolvedFields::default())?,
        progress_color: resolve_color_ref(resolver, &rule.progress_color, &ResolvedFields::default())?,
    })
}

pub fn resolve_card_color_rule(
    resolver: &LookResolver<'_>,
    rule: &CardColorRule,
) -> anyhow::Result<ResolvedCardColors> {
    let mut fields = ResolvedFields::default();
    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());
    let foreground = resolve_color_ref(resolver, &rule.foreground, &fields)?;
    fields.insert("foreground", foreground.clone());
    let muted_foreground = resolve_color_ref(resolver, &rule.muted_foreground, &fields)?;
    let border = resolve_color_ref(resolver, &rule.border, &fields)?;
    Ok(ResolvedCardColors { background, foreground, muted_foreground, border })
}

pub fn resolve_badge_color_rule(
    resolver: &LookResolver<'_>,
    rule: &BadgeColorRule,
) -> anyhow::Result<ResolvedBadgeColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_optional_stylesheet_color(resolver, rule.border.as_deref(), &fields, &ctx)?;
    Ok(ResolvedBadgeColors { background, foreground, border })
}

pub fn resolve_split_view_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SplitViewColorRule,
) -> anyhow::Result<ResolvedSplitViewColors> {
    Ok(ResolvedSplitViewColors {
        separator: resolve_color_ref(resolver, &rule.separator, &ResolvedFields::default())?,
        separator_hover: resolve_color_ref(resolver, &rule.separator_hover, &ResolvedFields::default())?,
    })
}

pub fn resolve_control_group_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ControlGroupListColorRule,
) -> anyhow::Result<ResolvedControlGroupListColors> {
    let mut fields = ResolvedFields::default();
    let background = resolve_color_ref(resolver, &rule.background, &fields)?;
    fields.insert("background", background.clone());
    let border = resolve_color_ref(resolver, &rule.border, &fields)?;
    Ok(ResolvedControlGroupListColors { background, border })
}

pub fn resolve_checkbox_color_rule(
    resolver: &LookResolver<'_>,
    rule: &CheckboxColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedCheckboxColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let indicator_background = resolve_stylesheet_color(resolver, &rule.indicator_background, &fields, &ctx)?;
    fields.insert("indicator_background", indicator_background.clone());
    let checkmark_color = resolve_stylesheet_color(resolver, &rule.checkmark_color, &fields, &ctx)?;
    fields.insert("checkmark_color", checkmark_color.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedCheckboxColors { indicator_background, checkmark_color, label_color })
}

pub fn resolve_radio_color_rule(
    resolver: &LookResolver<'_>,
    rule: &RadioColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedRadioColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let indicator_background = resolve_stylesheet_color(resolver, &rule.indicator_background, &fields, &ctx)?;
    fields.insert("indicator_background", indicator_background.clone());
    let selection_ring = resolve_stylesheet_color(resolver, &rule.selection_ring, &fields, &ctx)?;
    fields.insert("selection_ring", selection_ring.clone());
    let dot_color = resolve_stylesheet_color(resolver, &rule.dot_color, &fields, &ctx)?;
    fields.insert("dot_color", dot_color.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedRadioColors { indicator_background, selection_ring, dot_color, label_color })
}

pub fn resolve_switch_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SwitchColorRule,
    style: ShadcnButtonStyle,
) -> anyhow::Result<ResolvedSwitchColors> {
    let ctx = ResolveContext { style, layer: InteractionLayer::Default, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    fields.insert("thumb_background", thumb_background.clone());
    let thumb_border = resolve_stylesheet_color(resolver, &rule.thumb_border, &fields, &ctx)?;
    fields.insert("thumb_border", thumb_border.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    Ok(ResolvedSwitchColors { track_background, thumb_background, thumb_border, label_color })
}

pub fn resolve_slider_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SliderColorRule,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSliderColors> {
    let ctx = ResolveContext { style, layer, disabled: layer == InteractionLayer::Disabled };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let fill_background = resolve_stylesheet_color(resolver, &rule.fill_background, &fields, &ctx)?;
    fields.insert("fill_background", fill_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    fields.insert("thumb_background", thumb_background.clone());
    let thumb_border = resolve_stylesheet_color(resolver, &rule.thumb_border, &fields, &ctx)?;
    Ok(ResolvedSliderColors { track_background, fill_background, thumb_background, thumb_border })
}

pub fn resolve_scrollbar_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ScrollbarColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedScrollbarColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let track_background = resolve_stylesheet_color(resolver, &rule.track_background, &fields, &ctx)?;
    fields.insert("track_background", track_background.clone());
    let thumb_background = resolve_stylesheet_color(resolver, &rule.thumb_background, &fields, &ctx)?;
    Ok(ResolvedScrollbarColors { track_background, thumb_background })
}

pub fn resolve_accordion_trigger_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AccordionTriggerColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedAccordionTriggerColors> {
    let disabled = rule.disabled.unwrap_or(false);
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let chevron_color = resolve_stylesheet_color(resolver, &rule.chevron_color, &fields, &ctx)?;
    fields.insert("chevron_color", chevron_color.clone());
    let border_color = resolve_stylesheet_color(resolver, &rule.border_color, &fields, &ctx)?;
    fields.insert("border_color", border_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedAccordionTriggerColors { foreground, icon_color, chevron_color, border_color, background })
}

pub fn resolve_accordion_content_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AccordionContentColorRule,
) -> anyhow::Result<ResolvedAccordionContentColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedAccordionContentColors { foreground, background })
}

pub fn resolve_resizable_panels_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ResizablePanelsColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedResizablePanelsColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    fields.insert("divider", divider.clone());
    let grip = resolve_stylesheet_color(resolver, &rule.grip, &fields, &ctx)?;
    fields.insert("grip", grip.clone());
    let grip_emphasis = resolve_stylesheet_color(resolver, &rule.grip_emphasis, &fields, &ctx)?;
    Ok(ResolvedResizablePanelsColors { border, divider, grip, grip_emphasis })
}

pub fn resolve_listbox_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListboxListColorRule,
) -> anyhow::Result<ResolvedListboxListColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    Ok(ResolvedListboxListColors { background, border, divider })
}

pub fn resolve_listbox_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListboxRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedListboxRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    Ok(ResolvedListboxRowColors { label_color, background })
}

pub fn resolve_list_view_surface_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListViewSurfaceColorRule,
) -> anyhow::Result<ResolvedListViewSurfaceColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let header_background = resolve_stylesheet_color(resolver, &rule.header_background, &fields, &ctx)?;
    fields.insert("header_background", header_background.clone());
    let header_label_color = resolve_stylesheet_color(resolver, &rule.header_label_color, &fields, &ctx)?;
    Ok(ResolvedListViewSurfaceColors { background, border, header_background, header_label_color })
}

pub fn resolve_list_view_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &ListViewRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedListViewRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let divider = resolve_stylesheet_color(resolver, &rule.divider, &fields, &ctx)?;
    Ok(ResolvedListViewRowColors { background, label_color, divider })
}

pub fn resolve_floating_menu_surface_color_rule(
    resolver: &LookResolver<'_>,
    rule: &FloatingMenuSurfaceColorRule,
) -> anyhow::Result<ResolvedFloatingMenuColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let item_hover_background = resolve_stylesheet_color(resolver, &rule.item_hover_background, &fields, &ctx)?;
    fields.insert("item_hover_background", item_hover_background.clone());
    let item_hover_foreground = resolve_stylesheet_color(resolver, &rule.item_hover_foreground, &fields, &ctx)?;
    fields.insert("item_hover_foreground", item_hover_foreground.clone());
    let item_disabled_foreground = resolve_stylesheet_color(resolver, &rule.item_disabled_foreground, &fields, &ctx)?;
    Ok(ResolvedFloatingMenuColors {
        background,
        foreground,
        border,
        item_hover_background,
        item_hover_foreground,
        item_disabled_foreground,
    })
}

pub fn resolve_floating_menu_trigger_color_rule(
    resolver: &LookResolver<'_>,
    rule: &FloatingMenuTriggerColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedGhostTriggerColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    Ok(ResolvedGhostTriggerColors { background, foreground })
}

pub fn resolve_tabs_navigation_list_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TabsNavigationListColorRule,
) -> anyhow::Result<ResolvedTabsNavigationListColors> {
    let disabled_background = resolve_stylesheet_color(
        resolver,
        &rule.disabled_background,
        &ResolvedFields::default(),
        &ResolveContext::default(),
    )?;
    Ok(ResolvedTabsNavigationListColors { disabled_background })
}

pub fn resolve_tabs_navigation_item_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TabsNavigationItemColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedTabsNavigationItemColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: false };
    let mut fields = ResolvedFields::default();
    let label_color = resolve_stylesheet_color(resolver, &rule.label_color, &fields, &ctx)?;
    fields.insert("label_color", label_color.clone());
    let indicator = resolve_optional_stylesheet_color(resolver, rule.indicator.as_deref(), &fields, &ctx)?;
    Ok(ResolvedTabsNavigationItemColors { label_color, indicator })
}

pub fn resolve_tree_view_row_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TreeViewRowColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedTreeViewRowColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let chevron_color = resolve_stylesheet_color(resolver, &rule.chevron_color, &fields, &ctx)?;
    fields.insert("chevron_color", chevron_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedTreeViewRowColors { foreground, icon_color, chevron_color, background })
}

pub fn resolve_sidebar_container_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarContainerColorRule,
) -> anyhow::Result<ResolvedSidebarContainerColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    Ok(ResolvedSidebarContainerColors { background, foreground, border })
}

pub fn resolve_sidebar_section_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarSectionColorRule,
) -> anyhow::Result<ResolvedSidebarSectionColors> {
    let label_color =
        resolve_stylesheet_color(resolver, &rule.label_color, &ResolvedFields::default(), &ResolveContext::default())?;
    Ok(ResolvedSidebarSectionColors { label_color })
}

pub fn resolve_sidebar_branch_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarBranchColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSidebarBranchColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedSidebarBranchColors { foreground, icon_color, background })
}

pub fn resolve_sidebar_item_color_rule(
    resolver: &LookResolver<'_>,
    rule: &SidebarItemColorRule,
    layer: InteractionLayer,
) -> anyhow::Result<ResolvedSidebarItemColors> {
    let ctx = ResolveContext { style: ShadcnButtonStyle::Primary, layer, disabled: rule.disabled.unwrap_or(false) };
    let mut fields = ResolvedFields::default();
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let icon_color = resolve_stylesheet_color(resolver, &rule.icon_color, &fields, &ctx)?;
    fields.insert("icon_color", icon_color.clone());
    let background = resolve_optional_stylesheet_color(resolver, rule.background.as_deref(), &fields, &ctx)?;
    Ok(ResolvedSidebarItemColors { foreground, icon_color, background })
}

pub fn resolve_textfield_color_rule(
    resolver: &LookResolver<'_>,
    rule: &TextfieldColorRule,
) -> anyhow::Result<ResolvedTextfieldColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let background = resolve_stylesheet_color(resolver, &rule.background, &fields, &ctx)?;
    fields.insert("background", background.clone());
    let foreground = resolve_stylesheet_color(resolver, &rule.foreground, &fields, &ctx)?;
    fields.insert("foreground", foreground.clone());
    let border = resolve_stylesheet_color(resolver, &rule.border, &fields, &ctx)?;
    fields.insert("border", border.clone());
    let placeholder = resolve_stylesheet_color(resolver, &rule.placeholder, &fields, &ctx)?;
    fields.insert("placeholder", placeholder.clone());
    let icon = resolve_stylesheet_color(resolver, &rule.icon, &fields, &ctx)?;
    fields.insert("icon", icon.clone());
    let selection_background = resolve_stylesheet_color(resolver, &rule.selection_background, &fields, &ctx)?;
    fields.insert("selection_background", selection_background.clone());
    let selection_foreground = resolve_stylesheet_color(resolver, &rule.selection_foreground, &fields, &ctx)?;
    fields.insert("selection_foreground", selection_foreground.clone());
    let caret = resolve_stylesheet_color(resolver, &rule.caret, &fields, &ctx)?;
    Ok(ResolvedTextfieldColors {
        background,
        foreground,
        border,
        placeholder,
        icon,
        selection_background,
        selection_foreground,
        caret,
    })
}

pub fn resolve_autocomplete_chrome_color_rule(
    resolver: &LookResolver<'_>,
    rule: &AutocompleteChromeColorRule,
) -> anyhow::Result<ResolvedAutocompleteChromeColors> {
    let ctx = ResolveContext::default();
    let mut fields = ResolvedFields::default();
    let status_color = resolve_stylesheet_color(resolver, &rule.status_color, &fields, &ctx)?;
    fields.insert("status_color", status_color.clone());
    let muted_text_color = resolve_stylesheet_color(resolver, &rule.muted_text_color, &fields, &ctx)?;
    fields.insert("muted_text_color", muted_text_color.clone());
    let clear_icon_color = resolve_stylesheet_color(resolver, &rule.clear_icon_color, &fields, &ctx)?;
    fields.insert("clear_icon_color", clear_icon_color.clone());
    let clear_icon_hover_color = resolve_stylesheet_color(resolver, &rule.clear_icon_hover_color, &fields, &ctx)?;
    Ok(ResolvedAutocompleteChromeColors { status_color, muted_text_color, clear_icon_color, clear_icon_hover_color })
}

pub fn resolve_slider_metrics(
    rule: &SliderMetricsRule,
    metrics: &MetricTokens,
    size: ControlSize,
) -> ResolvedSliderMetrics {
    ResolvedSliderMetrics {
        width: rule.width,
        height: rule.height,
        track_height: rule.track_height,
        thumb_size: rule.thumb_size,
        radius: resolve_stylesheet_metric(&rule.radius, metrics, size).unwrap_or(metrics.radius.pill),
    }
}

pub fn resolve_switch_metrics(rule: &SwitchMetricsRule) -> ResolvedSwitchMetrics {
    ResolvedSwitchMetrics { width: rule.width, height: rule.height, thumb_size: rule.thumb_size }
}

pub fn resolve_scrollbar_metrics(rule: &ScrollbarMetricsRule) -> ResolvedScrollbarMetrics {
    ResolvedScrollbarMetrics {
        thickness: rule.thickness,
        track_thickness: rule.track_thickness,
        thumb_thickness: rule.thumb_thickness,
        min_thumb_length: rule.min_thumb_length,
    }
}

pub fn resolve_progress_metrics(rule: &ProgressMetricsRule) -> ResolvedProgressMetrics {
    ResolvedProgressMetrics {
        size: rule.size,
        stroke_width: rule.stroke_width,
        track_height: rule.track_height,
        thumb_size: rule.thumb_size,
    }
}

pub fn resolve_stepper_metrics(rule: &StepperMetricsRule) -> ResolvedStepperMetrics {
    ResolvedStepperMetrics { step_badge_size: rule.step_badge_size, track_thickness: rule.track_thickness }
}

#[derive(Clone, Debug)]
pub struct ResolvedProgressColors {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedStepperMetrics {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedCardColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub muted_foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedBadgeColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSplitViewColors {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedCheckboxColors {
    pub indicator_background: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedRadioColors {
    pub indicator_background: ResolvedColor,
    pub selection_ring: ResolvedColor,
    pub dot_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSwitchColors {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSliderColors {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedScrollbarColors {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedAccordionTriggerColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub border_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedAccordionContentColors {
    pub foreground: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedResizablePanelsColors {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListboxListColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListboxRowColors {
    pub label_color: ResolvedColor,
    pub background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListViewSurfaceColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedListViewRowColors {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedFloatingMenuColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedGhostTriggerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTabsNavigationListColors {
    pub disabled_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedTabsNavigationItemColors {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedTreeViewRowColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarContainerColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarSectionColors {
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarBranchColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedSidebarItemColors {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

#[derive(Clone, Debug)]
pub struct ResolvedTextfieldColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ResolvedAutocompleteChromeColors {
    pub status_color: ResolvedColor,
    pub muted_text_color: ResolvedColor,
    pub clear_icon_color: ResolvedColor,
    pub clear_icon_hover_color: ResolvedColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSliderMetrics {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedSwitchMetrics {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedScrollbarMetrics {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedControlGroupListColors {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedProgressMetrics {
    pub size: f32,
    pub stroke_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
}

#[derive(Clone, Debug)]
pub struct ResolvedButtonColors {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedButtonMetrics {
    pub height: f32,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}

pub fn resolve_button_metrics_rule(
    rule: &ButtonMetricsRule,
    metrics: &MetricTokens,
    size: ControlSize,
) -> ResolvedButtonMetrics {
    ResolvedButtonMetrics {
        height: resolve_stylesheet_metric(&rule.height, metrics, size).unwrap_or_else(|| metrics.control_height(size)),
        padding_horizontal: rule.padding_horizontal,
        font_size: rule.font_size,
        icon_size: rule.icon_size,
        corner_radius: resolve_stylesheet_metric(&rule.corner_radius, metrics, size)
            .unwrap_or_else(|| metrics.radius(size)),
    }
}

pub fn resolve_sidebar_metrics(rule: &SidebarMetricsRule, metrics: &MetricTokens) -> SidebarMetricScale {
    let defaults = SidebarMetricScale::default();
    let size = ControlSize::Md;
    let resolve =
        |raw: &str, fallback: gpui::Pixels| resolve_stylesheet_metric(raw, metrics, size).map(px).unwrap_or(fallback);

    SidebarMetricScale {
        width_expanded: resolve(&rule.width_expanded, defaults.width_expanded),
        width_icon_rail: resolve(&rule.width_icon_rail, defaults.width_icon_rail),
        width_mobile: resolve(&rule.width_mobile, defaults.width_mobile),
        padding_expanded: resolve(&rule.padding_expanded, defaults.padding_expanded),
        padding_icon_rail: resolve(&rule.padding_icon_rail, defaults.padding_icon_rail),
        item_height: resolve(&rule.item_height, defaults.item_height),
        icon_size: resolve(&rule.icon_size, defaults.icon_size),
        rail_hit_width: resolve(&rule.rail_hit_width, defaults.rail_hit_width),
        popover_offset: resolve(&rule.popover_offset, defaults.popover_offset),
    }
}

pub fn resolve_stylesheet_shadow_token(raw: &str) -> Option<String> {
    match raw.trim() {
        "" | "none" => None,
        token => Some(token.to_string()),
    }
}

pub fn resolve_layered_elevation_shadow(
    rules: &[crate::stylesheet::config::LayeredElevationRule],
    layer: InteractionLayer,
) -> Option<String> {
    let rule = if layer == InteractionLayer::Disabled {
        rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"))
    } else {
        rules.iter().find(|rule| rule.layer.is_none())
    }?;
    resolve_stylesheet_shadow_token(&rule.shadow)
}

pub fn resolve_typography_rule(rule: &TypographyRule) -> LumaTextStyle {
    LumaTextStyle { size: rule.size, line_height: rule.line_height, weight: FontWeight(rule.weight) }
}

pub fn resolve_stylesheet_metric(raw: &str, metrics: &MetricTokens, size: ControlSize) -> Option<f32> {
    match raw.trim() {
        "metrics.control.sm" => Some(metrics.control_height(ControlSize::Sm)),
        "metrics.control.md" => Some(metrics.control_height(ControlSize::Md)),
        "metrics.control.lg" => Some(metrics.control_height(ControlSize::Lg)),
        "radius" => Some(metrics.radius(size)),
        "radius.pill" => Some(metrics.radius.pill),
        value => resolve_f32_literal(value).ok(),
    }
}

fn resolve_f32_literal(raw: &str) -> anyhow::Result<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().map(|n| n * 16.0).map_err(Into::into);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().map_err(Into::into);
    }
    value.parse().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui::px;
    use gpui_luma::theme::{ControlSize, MetricTokens, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::provenance::LookResolver;

    use super::*;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330 64% 52%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(0 0% 100%)".into()),
        ]))
    }

    #[test]
    fn field_reference_uses_prior_resolved_color() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let mut fields = ResolvedFields::default();
        fields.insert("background", resolver.resolve_decl("primary").expect("primary"));

        let referenced = resolve_color_ref(&resolver, "@background", &fields).expect("reference");
        assert_eq!(referenced.value, fields.get("background").expect("background").value);
    }

    #[test]
    fn f32_literal_parses_px_and_rem() {
        assert!((resolve_f32_literal("12.0").expect("literal") - 12.0).abs() < f32::EPSILON);
        assert!((resolve_f32_literal("1rem").expect("rem") - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_sidebar_metrics_from_rem_literals() {
        use crate::stylesheet::config::SidebarMetricsRule;

        let metrics = MetricTokens::default();
        let rule = SidebarMetricsRule::default();
        let scale = resolve_sidebar_metrics(&rule, &metrics);
        assert_eq!(scale.width_expanded, px(256.0));
        assert_eq!(scale.width_icon_rail, px(48.0));
        assert_eq!(scale.width_mobile, px(288.0));
        assert_eq!(scale.padding_expanded, px(12.0));
        assert_eq!(scale.item_height, px(32.0));
        assert_eq!(scale.icon_size, px(16.0));
    }

    #[test]
    fn metric_token_resolves_control_height_and_radius() {
        let metrics = MetricTokens::default();
        assert_eq!(
            resolve_stylesheet_metric("metrics.control.md", &metrics, ControlSize::Md),
            Some(metrics.control_height(ControlSize::Md))
        );
        assert_eq!(
            resolve_stylesheet_metric("radius", &metrics, ControlSize::Sm),
            Some(metrics.radius(ControlSize::Sm))
        );
    }

    #[test]
    fn typography_rule_resolves_to_text_style() {
        let style = resolve_typography_rule(&TypographyRule { size: 16.0, line_height: 22.0, weight: 600.0 });
        assert_eq!(style.size, 16.0);
        assert_eq!(style.line_height, 22.0);
        assert_eq!(style.weight, FontWeight(600.0));
    }
}
