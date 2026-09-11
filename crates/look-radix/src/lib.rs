//! Minimal Radix Themes–shaped look for GPUI-Luma.
//!
//! Vertical slice: 12-step scales, semantic roles, and theme adapters for button,
//! textfield, textarea, slider, checkbox, radio, switch, toggle, tabs, popup menu,
//! and overlay window.
//! No Shadcn token or variant names. Does not require SDK API changes.

mod button;
mod button_layout;
mod colors;
mod checkbox;
mod ext;
mod look;
mod overlay_window;
mod palette;
mod popup_menu;
mod radio;
mod scale;
mod semantic;
mod signup_mesh;
mod slider;
mod switch;
mod tabs;
mod textarea;
mod textfield;
mod toggle;
mod tone;
mod typography;

pub use button::{
    ClassicButtonParams, RadixButtonPaint, RadixButtonVariant, button_family_theme, button_family_theme_with,
    button_look_for, classic_button_template, classic_button_template_with,
};
pub use button_layout::{RadixButtonSize, RadixRadius, button_box_for, radix_metric_tokens, resolve_button_radius};
pub use colors::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, RADIX_COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors::{RadixColorScale, RadixColorValueKind, families as color_families, parse_color};
pub use checkbox::{
    RadixCheckboxSize, RadixCheckboxVariant, checkbox_scale_for, checkbox_theme, checkbox_theme_for,
    checkbox_theme_with, resolve_checkbox_radius,
};
pub use ext::RadixLookControlExt;
pub use look::{PageBackground, RadixLook, SignupMeshColors, SignupStage};
pub use overlay_window::overlay_window_theme;
pub use palette::{PaletteSlot, RadixAccent, RadixGray, ThemePalettes, scale_pair};
pub use popup_menu::{RadixPopupMenuVariant, popup_menu_theme};
pub use radio::{RadixRadioSize, RadixRadioVariant, radio_scale_for, radio_theme, radio_theme_for, radio_theme_with};
pub use scale::{CUSTOM_PALETTE, ColorScale, ModeScales, SCALE_LEN, ScaleFamily, ScalePair, ScaleStep};
pub use semantic::{SemanticMapping, SemanticRole};
pub use signup_mesh::{
    MESH_DISPLAY_LEFT_FRAC, MESH_DISPLAY_WIDTH_FRAC, MESH_VIEWBOX_H, MESH_VIEWBOX_W, SignupMeshCacheKey,
    rasterize_signup_mesh, rasterize_signup_mesh_for_look, rasterize_signup_mesh_stage,
};
pub use slider::{RadixSliderVariant, slider_theme, slider_theme_with};
pub use switch::{
    RadixSwitchSize, RadixSwitchVariant, resolve_switch_radius, switch_scale_for, switch_theme, switch_theme_for,
    switch_theme_with,
};
pub use tabs::tabs_theme;
pub use textarea::{textarea_theme, textarea_theme_with};
pub use textfield::{RadixTextFieldVariant, textfield_theme, textfield_theme_with};
pub use toggle::toggle_template;
pub use tone::RadixTone;
