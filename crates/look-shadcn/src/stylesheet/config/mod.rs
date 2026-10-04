use gpui_luma::theme::{ControlSize, InteractionLayer};
use serde::Deserialize;

use super::selector::interaction_layer_key;

mod typography;
pub use typography::*;
mod button;
pub use button::*;
mod progress;
pub use progress::*;
mod stepper;
pub use stepper::*;
mod card;
pub use card::*;
mod badge;
pub use badge::*;
mod split_view;
pub use split_view::*;
mod control_group;
pub use control_group::*;
mod checkbox;
pub use checkbox::*;
mod radio;
pub use radio::*;
mod switch;
pub use switch::*;
mod slider;
pub use slider::*;
mod scrollbar;
pub use scrollbar::*;
mod accordion;
pub use accordion::*;
mod resizable_panels;
pub use resizable_panels::*;
mod listbox;
pub use listbox::*;
mod table;
pub use table::*;
mod floating_menu;
pub use floating_menu::*;
mod tabs;
pub use tabs::*;
mod tree_view;
pub use tree_view::*;
mod sidebar;
pub use sidebar::*;
mod textfield;
pub use textfield::*;
mod autocomplete;
pub use autocomplete::*;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StylesheetConfig {
    #[serde(default)]
    pub common: gpui_luma::theme::stylesheet::CommonStylesheet,
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
    pub table: TableStylesheet,
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
    /// Parses stylesheet TOML without imposing an input-size limit.
    /// Applications own file access and any limits required for untrusted input.
    pub fn parse(source: &str) -> anyhow::Result<Self> {
        toml::from_str(source).map_err(|err| anyhow::anyhow!("parse style.toml: {err}"))
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

fn control_size_key(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
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
