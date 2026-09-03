use std::collections::HashMap;

use luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use crate::controls::ShadcnButtonStyle;
use super::selector::interaction_layer_key;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StylesheetConfig {
    #[serde(default)]
    pub typography: TypographyStylesheet,
    #[serde(default)]
    pub button: ButtonStylesheet,
    #[serde(default)]
    pub toggle: ToggleStylesheet,
    #[serde(default)]
    pub checkbox: CheckboxStylesheet,
    #[serde(default)]
    pub radio: RadioStylesheet,
    #[serde(default)]
    pub switch: SwitchStylesheet,
    #[serde(default)]
    pub slider: SliderStylesheet,
    #[serde(default)]
    pub scrollbar: ScrollbarStylesheet,
    #[serde(default)]
    pub accordion: AccordionStylesheet,
    #[serde(default)]
    pub resizable_panels: ResizablePanelsStylesheet,
    #[serde(default)]
    pub listbox: ListboxStylesheet,
    #[serde(default)]
    pub list_view: ListViewStylesheet,
    #[serde(default)]
    pub floating_menu: FloatingMenuStylesheet,
    #[serde(default)]
    pub tabs: TabsStylesheet,
    #[serde(default)]
    pub tree_view: TreeViewStylesheet,
    #[serde(default, alias = "navigation_sidebar")]
    pub sidebar: SidebarStylesheet,
    #[serde(default)]
    pub textfield: TextfieldStylesheet,
    #[serde(default)]
    pub autocomplete: AutocompleteStylesheet,
    #[serde(default)]
    pub progress: ProgressStylesheet,
    #[serde(default)]
    pub stepper: StepperStylesheet,
    #[serde(default)]
    pub card: CardStylesheet,
    #[serde(default)]
    pub badge: BadgeStylesheet,
    #[serde(default)]
    pub split_view: SplitViewStylesheet,
    #[serde(default)]
    pub control_group: ControlGroupStylesheet,
}

impl StylesheetConfig {
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        toml::from_str(source).map_err(|err| anyhow::anyhow!("parse style.toml: {err}"))
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TypographyStylesheet {
    #[serde(default)]
    pub semantic: HashMap<String, TypographyRule>,
    #[serde(default)]
    pub scale: HashMap<String, TypographyRule>,
}

impl TypographyStylesheet {
    pub fn semantic_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.semantic.get(key)
    }

    pub fn scale_rule(&self, key: &str) -> Option<&TypographyRule> {
        self.scale.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct TypographyRule {
    pub size: f32,
    pub line_height: f32,
    pub weight: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ButtonStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<ButtonElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<ButtonColorRule>,
}

impl ButtonStylesheet {
    pub fn find_color_rule(&self, style: &str, layer: &str, mode: &str, selected: bool) -> Option<&ButtonColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, layer, mode, selected))
    }

    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&ButtonElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ToggleStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
}

impl ToggleStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ButtonColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub mode: Option<String>,
    pub selected: Option<bool>,

    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl ButtonColorRule {
    pub fn matches(&self, style: &str, layer: &str, mode: &str, selected: bool) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.layer.as_deref().is_none_or(|value| value == "any" || value == layer)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
            && self.selected.is_none_or(|value| value == selected)
    }
}

pub fn find_enabled_color_rule<R: EnabledColorRule>(rules: &[R], enabled: bool) -> Option<&R> {
    rules.iter().find(|rule| rule.matches_enabled(enabled))
}

pub trait EnabledColorRule {
    fn enabled(&self) -> Option<bool>;

    fn matches_enabled(&self, enabled: bool) -> bool {
        self.enabled().is_none_or(|value| value == enabled)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ProgressStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ProgressMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ProgressColorRule>,
}

impl ProgressStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ProgressMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&ProgressColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ProgressMetricsRule {
    pub size: f32,
    pub stroke_width: f32,
    #[serde(default = "default_progress_track_height")]
    pub track_height: f32,
    #[serde(default = "default_progress_thumb_size")]
    pub thumb_size: f32,
}

fn default_progress_track_height() -> f32 {
    6.0
}

fn default_progress_thumb_size() -> f32 {
    16.0
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProgressColorRule {
    pub enabled: Option<bool>,
    pub track_color: String,
    pub progress_color: String,
}

impl EnabledColorRule for ProgressColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StepperStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, StepperMetricsRule>,
}

impl StepperStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&StepperMetricsRule> {
        self.metrics.get(control_size_key(size))
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct StepperMetricsRule {
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CardStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rule: Option<CardColorRule>,
}

impl CardStylesheet {
    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&CardColorRule> {
        self.color_rule.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CardColorRule {
    pub background: String,
    pub foreground: String,
    pub muted_foreground: String,
    pub border: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BadgeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<BadgeColorRule>,
}

impl BadgeStylesheet {
    pub fn find_color_rule(&self, style: &str, mode: &str) -> Option<&BadgeColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, mode))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BadgeColorRule {
    pub style: Option<String>,
    pub mode: Option<String>,
    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl BadgeColorRule {
    pub fn matches(&self, style: &str, mode: &str) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SplitViewStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SplitViewColorRule>,
}

impl SplitViewStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&SplitViewColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SplitViewColorRule {
    pub enabled: Option<bool>,
    pub separator: String,
    pub separator_hover: String,
}

impl EnabledColorRule for SplitViewColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ControlGroupStylesheet {
    #[serde(default)]
    pub list: ControlGroupListStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ControlGroupListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ControlGroupListColorRule>,
}

impl ControlGroupListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ControlGroupListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ControlGroupListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
}

impl EnabledColorRule for ControlGroupListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

pub(crate) fn matches_optional_bool(selector: Option<bool>, value: bool) -> bool {
    selector.is_none_or(|expected| expected == value)
}

pub(crate) fn matches_optional_layer(selector: Option<&str>, layer: InteractionLayer) -> bool {
    selector.is_none_or(|expected| expected == interaction_layer_key(layer))
}

pub(crate) fn matches_optional_str(selector: Option<&str>, value: &str) -> bool {
    selector.is_none_or(|expected| expected == value)
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayeredElevationRule {
    pub layer: Option<String>,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CheckboxStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<CheckboxColorRule>,
}

impl CheckboxStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(&self, checked: bool, layer: InteractionLayer) -> Option<&CheckboxColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            rule.layer.is_none() && matches_optional_bool(rule.checked, checked)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CheckboxColorRule {
    pub layer: Option<String>,
    pub checked: Option<bool>,
    pub indicator_background: String,
    pub checkmark_color: String,
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RadioIndicatorDefaults {
    pub primary: Option<String>,
    pub secondary: Option<String>,
    pub outline: Option<String>,
    pub ghost: Option<String>,
}

impl RadioIndicatorDefaults {
    pub fn for_style(&self, style: ShadcnButtonStyle) -> &str {
        match style {
            ShadcnButtonStyle::Primary => self.primary.as_deref().unwrap_or("filled"),
            ShadcnButtonStyle::Secondary => self.secondary.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::Outline => self.outline.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::Ghost => self.ghost.as_deref().unwrap_or("ring"),
            ShadcnButtonStyle::ContentOnly => self.primary.as_deref().unwrap_or("filled"),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RadioStylesheet {
    #[serde(default)]
    pub indicator_defaults: RadioIndicatorDefaults,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<RadioColorRule>,
}

impl RadioStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(
        &self,
        style: ShadcnButtonStyle,
        selected: bool,
        layer: InteractionLayer,
    ) -> Option<&RadioColorRule> {
        let indicator = self.indicator_defaults.for_style(style);
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            if rule.layer.is_some() {
                return false;
            }
            if !matches_optional_bool(rule.selected, selected) {
                return false;
            }
            matches_optional_str(rule.indicator.as_deref(), indicator)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RadioColorRule {
    pub layer: Option<String>,
    pub selected: Option<bool>,
    pub indicator: Option<String>,
    pub indicator_background: String,
    pub selection_ring: String,
    pub dot_color: String,
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SwitchStylesheet {
    /// Switch geometry keyed by `sm` / `md` / `lg`. All variants share these metrics.
    #[serde(default)]
    pub metrics: HashMap<String, SwitchMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<SwitchColorRule>,
}

impl SwitchStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(&self, on: bool, disabled: bool) -> Option<&SwitchColorRule> {
        self.color_rules
            .iter()
            .find(|rule| matches_optional_bool(rule.disabled, disabled) && matches_optional_bool(rule.on, on))
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&SwitchMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn metrics_for_style(&self, _style: ShadcnButtonStyle, size: ControlSize) -> Option<&SwitchMetricsRule> {
        self.metrics_for_size(size)
    }
}

fn control_size_key(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct SwitchMetricsRule {
    pub width: f32,
    pub height: f32,
    pub thumb_size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SwitchColorRule {
    pub on: Option<bool>,
    pub disabled: Option<bool>,
    pub track_background: String,
    pub thumb_background: String,
    pub thumb_border: String,
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SliderStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, SliderMetricsRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<SliderColorRule>,
}

impl SliderStylesheet {
    pub fn find_color_rule(&self, style: &str, layer: InteractionLayer) -> Option<&SliderColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }

    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&SliderMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderMetricsRule {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub track_background: String,
    pub fill_background: String,
    pub thumb_background: String,
    pub thumb_border: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ScrollbarStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ScrollbarMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ScrollbarColorRule>,
}

impl ScrollbarStylesheet {
    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ScrollbarMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, style: &str, disabled: bool, layer: InteractionLayer) -> Option<&ScrollbarColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_str(rule.style.as_deref(), style)
                && matches_optional_bool(rule.disabled, disabled)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ScrollbarMetricsRule {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScrollbarColorRule {
    pub style: Option<String>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub track_background: String,
    pub thumb_background: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionStylesheet {
    #[serde(default)]
    pub trigger: AccordionTriggerStylesheet,
    #[serde(default)]
    pub content: AccordionContentStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionTriggerColorRule>,
}

impl AccordionTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&AccordionTriggerColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccordionTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub border_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AccordionContentStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionContentColorRule>,
}

impl AccordionContentStylesheet {
    pub fn find_color_rule(&self, expanded: bool) -> Option<&AccordionContentColorRule> {
        self.color_rules.iter().find(|rule| matches_optional_bool(rule.expanded, expanded))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AccordionContentColorRule {
    pub expanded: Option<bool>,
    pub foreground: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ResizablePanelsStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ResizablePanelsColorRule>,
}

impl ResizablePanelsStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&ResizablePanelsColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_bool(rule.disabled, disabled) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResizablePanelsColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub border: String,
    pub divider: String,
    pub grip: String,
    pub grip_emphasis: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxStylesheet {
    #[serde(default)]
    pub list: ListboxListStylesheet,
    #[serde(default)]
    pub row: ListboxRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxListColorRule>,
}

impl ListboxListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ListboxListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListboxListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub divider: String,
}

impl EnabledColorRule for ListboxListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListboxRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListboxRowColorRule>,
}

impl ListboxRowStylesheet {
    pub fn find_color_rule(
        &self,
        disabled: bool,
        focused: bool,
        layer: InteractionLayer,
    ) -> Option<&ListboxRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.focused, focused) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListboxRowColorRule {
    pub disabled: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub label_color: String,
    pub background: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListViewStylesheet {
    #[serde(default)]
    pub surface: ListViewSurfaceStylesheet,
    #[serde(default)]
    pub row: ListViewRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListViewSurfaceStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ListViewSurfaceMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ListViewSurfaceColorRule>,
}

impl ListViewSurfaceStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ListViewSurfaceColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ListViewSurfaceMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListViewSurfaceMetricsRule {
    pub padding_y_factor: f32,
    pub radius: String,
    #[serde(default)]
    pub max_radius: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListViewSurfaceColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub header_background: String,
    pub header_label_color: String,
}

impl EnabledColorRule for ListViewSurfaceColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListViewRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ListViewRowColorRule>,
}

impl ListViewRowStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        focused: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&ListViewRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            if rule.selected == Some(true) {
                return selected;
            }
            if rule.focused == Some(true) {
                return focused;
            }
            if selected || focused {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListViewRowColorRule {
    pub disabled: Option<bool>,
    pub selected: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub label_color: String,
    pub divider: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FloatingMenuStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, FloatingMenuMetricsRule>,
    #[serde(default)]
    pub surface: FloatingMenuSurfaceStylesheet,
    #[serde(default)]
    pub trigger: FloatingMenuTriggerStylesheet,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuMetricsRule {
    pub min_width: f32,
    pub radius: String,
    pub item_height_factor: f32,
    pub item_padding_x_factor: f32,
    pub item_radius: String,
    pub submenu_offset_x_factor: f32,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FloatingMenuSurfaceStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuSurfaceColorRule>,
}

impl FloatingMenuSurfaceStylesheet {
    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&FloatingMenuSurfaceColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuSurfaceElevationRule {
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuSurfaceColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub item_hover_background: String,
    pub item_hover_foreground: String,
    pub item_disabled_foreground: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FloatingMenuTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuTriggerColorRule>,
}

impl FloatingMenuTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&FloatingMenuTriggerColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct FloatingMenuTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub foreground: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsStylesheet {
    #[serde(default)]
    pub list: TabsListStylesheet,
    #[serde(default)]
    pub item: TabsItemStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsListColorRule>,
}

impl TabsListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TabsListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TabsListColorRule {
    pub enabled: Option<bool>,
    pub disabled_background: String,
}

impl EnabledColorRule for TabsListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TabsItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsItemColorRule>,
}

impl TabsItemStylesheet {
    pub fn find_color_rule(&self, active: bool, layer: InteractionLayer, focused: bool) -> Option<&TabsItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            matches_optional_bool(rule.active, active)
                && matches_optional_layer(rule.layer.as_deref(), layer)
                && matches_optional_bool(rule.focused, focused)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TabsItemColorRule {
    pub active: Option<bool>,
    pub layer: Option<String>,
    pub focused: Option<bool>,
    pub label_color: String,
    pub indicator: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TreeViewStylesheet {
    #[serde(default)]
    pub row: TreeViewRowStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TreeViewRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TreeViewRowColorRule>,
}

impl TreeViewRowStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&TreeViewRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TreeViewRowColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarStylesheet {
    #[serde(default)]
    pub metrics: SidebarMetricsRule,
    #[serde(default)]
    pub container: SidebarContainerStylesheet,
    #[serde(default)]
    pub section: SidebarSectionStylesheet,
    #[serde(default)]
    pub branch: SidebarBranchStylesheet,
    #[serde(default)]
    pub item: SidebarItemStylesheet,
}

/// Layout metrics for [`luma::controls::sidebar::SidebarMetricScale`].
///
/// Values accept bare numbers, `px`, or `rem` (1rem = 16px).
#[derive(Debug, Deserialize, Clone)]
pub struct SidebarMetricsRule {
    #[serde(default = "sidebar_metric_width_expanded")]
    pub width_expanded: String,
    #[serde(default = "sidebar_metric_width_icon_rail")]
    pub width_icon_rail: String,
    #[serde(default = "sidebar_metric_width_mobile")]
    pub width_mobile: String,
    #[serde(default = "sidebar_metric_padding_expanded")]
    pub padding_expanded: String,
    #[serde(default = "sidebar_metric_padding_icon_rail")]
    pub padding_icon_rail: String,
    #[serde(default = "sidebar_metric_item_height")]
    pub item_height: String,
    #[serde(default = "sidebar_metric_icon_size")]
    pub icon_size: String,
    #[serde(default = "sidebar_metric_rail_hit_width")]
    pub rail_hit_width: String,
    #[serde(default = "sidebar_metric_popover_offset")]
    pub popover_offset: String,
}

impl Default for SidebarMetricsRule {
    fn default() -> Self {
        Self {
            width_expanded: sidebar_metric_width_expanded(),
            width_icon_rail: sidebar_metric_width_icon_rail(),
            width_mobile: sidebar_metric_width_mobile(),
            padding_expanded: sidebar_metric_padding_expanded(),
            padding_icon_rail: sidebar_metric_padding_icon_rail(),
            item_height: sidebar_metric_item_height(),
            icon_size: sidebar_metric_icon_size(),
            rail_hit_width: sidebar_metric_rail_hit_width(),
            popover_offset: sidebar_metric_popover_offset(),
        }
    }
}

fn sidebar_metric_width_expanded() -> String {
    "16rem".into()
}
fn sidebar_metric_width_icon_rail() -> String {
    "3rem".into()
}
fn sidebar_metric_width_mobile() -> String {
    "18rem".into()
}
fn sidebar_metric_padding_expanded() -> String {
    "0.75rem".into()
}
fn sidebar_metric_padding_icon_rail() -> String {
    "0.375rem".into()
}
fn sidebar_metric_item_height() -> String {
    "2rem".into()
}
fn sidebar_metric_icon_size() -> String {
    "1rem".into()
}
fn sidebar_metric_rail_hit_width() -> String {
    "0.375rem".into()
}
fn sidebar_metric_popover_offset() -> String {
    "0.5rem".into()
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarContainerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarContainerColorRule>,
}

impl SidebarContainerStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarContainerColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarContainerColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarSectionStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarSectionColorRule>,
}

impl SidebarSectionStylesheet {
    pub fn color_rule(&self) -> Option<&SidebarSectionColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarSectionColorRule {
    pub label_color: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarBranchStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarBranchColorRule>,
}

impl SidebarBranchStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&SidebarBranchColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarBranchColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SidebarItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SidebarItemColorRule>,
}

impl SidebarItemStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&SidebarItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled && matches_optional_bool(rule.selected, selected);
            }
            if disabled {
                return false;
            }
            matches_optional_bool(rule.selected, selected) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SidebarItemColorRule {
    pub selected: Option<bool>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TextfieldStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<TextfieldElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<TextfieldColorRule>,
}

impl TextfieldStylesheet {
    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&TextfieldElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn find_color_rule(
        &self,
        style: &str,
        enabled: bool,
        invalid: bool,
        mode: &str,
    ) -> Option<&TextfieldColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && rule.enabled.is_none_or(|value| value == enabled)
                && rule.invalid.is_none_or(|value| value == invalid)
                && rule.mode.as_deref().is_none_or(|value| value == mode)
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextfieldElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TextfieldColorRule {
    pub style: Option<String>,
    pub enabled: Option<bool>,
    pub invalid: Option<bool>,
    pub mode: Option<String>,
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub placeholder: String,
    pub icon: String,
    pub selection_background: String,
    pub selection_foreground: String,
    pub caret: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AutocompleteStylesheet {
    #[serde(default)]
    pub chrome: AutocompleteChromeStylesheet,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AutocompleteChromeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AutocompleteChromeColorRule>,
}

impl AutocompleteChromeStylesheet {
    pub fn color_rule(&self) -> Option<&AutocompleteChromeColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutocompleteChromeColorRule {
    pub status_color: String,
    pub muted_text_color: String,
    pub clear_icon_color: String,
    pub clear_icon_hover_color: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TYPOGRAPHY_SNIPPET: &str = r#"
[typography.semantic.h1]
size = 44.0
line_height = 52.0
weight = 700.0

[typography.scale.xs]
size = 11.0
line_height = 16.0
weight = 500.0

[typography.scale."2xl"]
size = 20.0
line_height = 28.0
weight = 600.0
"#;

    const BUTTON_SNIPPET: &str = r#"
[button.metrics.md]
height = "metrics.control.md"
padding_horizontal = 16.0
font_size = 14.0
icon_size = 16.0
corner_radius = "radius"

[[button.color_rules]]
style = "primary"
layer = "default"
background = "primary"
foreground = "primary-foreground"

[[button.color_rules]]
background = "transparent"
foreground = "foreground"
"#;

    #[test]
    fn parses_typography_rules() {
        let config = StylesheetConfig::parse(TYPOGRAPHY_SNIPPET).expect("parse typography snippet");
        assert_eq!(config.typography.semantic_rule("h1").expect("h1").size, 44.0);
        assert_eq!(config.typography.scale_rule("xs").expect("xs").line_height, 16.0);
        assert_eq!(config.typography.scale_rule("2xl").expect("2xl").weight, 600.0);
    }

    #[test]
    fn parses_button_metrics_and_rules() {
        let config = StylesheetConfig::parse(BUTTON_SNIPPET).expect("parse snippet");
        assert!(config.button.metrics.contains_key("md"));
        assert_eq!(config.button.color_rules.len(), 2);
    }

    #[test]
    fn parses_button_elevation_rules() {
        let config = StylesheetConfig::parse(
            r#"
[[button.elevation_rules]]
style = "outline"
shadow = "shadow-xs"

[[button.elevation_rules]]
style = "primary"
shadow = "none"
"#,
        )
        .expect("parse elevation snippet");
        assert_eq!(config.button.elevation_rules.len(), 2);
        let outline = config.button.elevation_rule_for_style("outline").expect("outline rule");
        assert_eq!(outline.shadow, "shadow-xs");
    }

    #[test]
    fn first_matching_rule_wins() {
        let config = StylesheetConfig::parse(BUTTON_SNIPPET).expect("parse snippet");
        let rule = config.button.find_color_rule("primary", "default", "light", false).expect("primary default rule");
        assert_eq!(rule.background, "primary");
    }
}
