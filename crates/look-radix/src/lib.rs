//! Minimal Radix Themes–shaped look for GPUI-Luma.
//!
//! Vertical slice: 12-step scales, semantic roles, look-owned [`Button`] / [`Checkbox`] /
//! [`Radio`] / [`Switch`] / [`TextField`] / [`TextArea`] / [`Slider`] / [`Tabs`] /
//! [`Toggle`] / [`PopupMenu`] / [`TreeView`] / [`Toolbar`], plus [`callout`] and an overlay-window theme.
//! No Shadcn token or variant names. Does not require SDK API changes.

mod button;
mod button_builder;
mod button_layout;
mod colors;
mod custom_colors;
mod checkbox;
mod checkbox_builder;
mod ext;
mod look;
mod overlay_window;
mod palette;
mod popup_menu;
mod popup_menu_builder;
mod radio;
mod radio_builder;
mod scale;
mod semantic;
mod slider;
mod slider_builder;
mod switch;
mod switch_builder;
mod tabs;
mod tabs_builder;
mod textarea;
mod textarea_builder;
mod textfield;
mod textfield_builder;
mod toggle;
mod toggle_builder;
mod tone;
mod typography;
mod tree_view;
mod toolbar;
mod toolbar_builder;
mod callout;

pub use button::{
    ClassicButtonParams, Paint, ButtonVariant, button_family_theme, button_family_theme_with, button_look_for,
    button_template, classic_button_template, classic_button_template_with,
};
pub use button_builder::Button;
pub use button_layout::{ButtonSize, Radius, button_box_for, metric_tokens, resolve_button_radius};
pub use colors::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors::{RawColorScale, ColorValueKind, families as color_families, parse_color};
pub use checkbox::{
    CheckboxSize, CheckboxVariant, checkbox_scale_for, checkbox_template, checkbox_template_for, checkbox_theme,
    checkbox_theme_for, checkbox_theme_with, resolve_checkbox_radius,
};
pub use checkbox_builder::Checkbox;
pub use ext::LookControlExt;
pub use look::{PageBackground, Look};
pub use overlay_window::overlay_window_theme;
pub use palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
pub use popup_menu::{PopupMenuVariant, popup_menu_template, popup_menu_theme};
pub use popup_menu_builder::PopupMenu;
pub use radio::{
    RadioSize, RadioVariant, radio_scale_for, radio_template, radio_template_for, radio_theme, radio_theme_for,
    radio_theme_with,
};
pub use radio_builder::Radio;
pub use scale::{CUSTOM_PALETTE, ColorScale, ModeScales, SCALE_LEN, ScaleFamily, ScalePair, ScaleStep};
pub use semantic::{SemanticMapping, SemanticRole};

pub use slider::{SliderSize, SliderVariant, slider_template, slider_theme, slider_theme_with};
pub use slider_builder::Slider;
pub use switch::{
    SwitchSize, SwitchVariant, resolve_switch_radius, switch_scale_for, switch_template, switch_template_for,
    switch_theme, switch_theme_for, switch_theme_with,
};
pub use switch_builder::Switch;
pub use tabs::{
    TabsBaselineStyle, TabsMetrics, TabsStyle, TabsSize, TabsVariant, tabs_template, tabs_template_for, tabs_theme,
    tabs_theme_for,
};
pub use tabs_builder::Tabs;
pub use textarea::{TextAreaSize, textarea_template, textarea_theme, textarea_theme_with};
pub use textarea_builder::TextArea;
pub use textfield::{TextFieldSize, TextFieldVariant, textfield_template, textfield_theme, textfield_theme_with};
pub use textfield_builder::TextField;
pub use toggle::{page_toggle_template, toggle_template, toggle_template_for};
pub use toggle_builder::Toggle;
pub use tone::Tone;
pub use tree_view::{TreeView, tree_view_frame, tree_view_template, tree_view_theme};
pub use toolbar::{toolbar_template, toolbar_theme};
pub use toolbar_builder::Toolbar;
pub use callout::callout;

mod segmented;
pub use segmented::segmented_radio_template;

mod badge;
pub use badge::{Badge, BadgeSize, BadgeVariant};

pub use custom_colors::{CustomColors, GeneratedColors, generate_colors};

mod avatar;
pub use avatar::{Avatar, AvatarSize, AvatarVariant};

mod card;
pub use card::{Card, CardSize, CardVariant};
