//! Radix look for GPUI-Luma: palette scales, control styling, and SDK adapters.
//!
//! [`Look`] owns named or custom color scales, light/dark mode, metrics, and shared
//! styling settings. Interactive builders produce SDK controls, retaining their
//! input, focus, event, and accessibility behavior. Display elements render directly.
//!
//! # Available controls
//!
//! - Display: [`Avatar`], [`Badge`], [`Card`], and [`callout`].
//! - Actions and choices: [`Button`], [`Checkbox`], [`Radio`], [`Switch`], [`Toggle`].
//! - Input: [`TextField`], [`TextArea`], [`Slider`].
//! - Progress: [`Progress`] (Surface / Soft bars, sizes 1–3).
//! - Navigation and collections: [`Tabs`], [`Toolbar`], [`PopupMenu`], [`ContextMenu`], [`TreeView`].
//! - Composition: [`segmented_radio_template`] and [`LookControlExt::overlay_window`].
//!
//! # Start with a control builder
//!
//! Choose the control's supported variant, size, tone, and radius options first.
//! Bind `.look(&look)` explicitly when using a fork. Without an explicit look,
//! interactive builders use the ambient GPUI global [`Look`], then built-in defaults.
//!
//! ```no_run
//! use gpui::{Context, Entity};
//! use gpui_luma::infra::presenter::HasPresenter;
//! use gpui_luma_look_radix::{Button, ButtonSize, Look};
//!
//! fn save_button<M: 'static>(look: &Look, cx: &mut Context<M>)
//!     -> Entity<gpui_luma::controls::button::Button>
//! {
//!     Button::new("save").look(look).solid().size(ButtonSize::Two)
//!         .label("Save").spawn(cx)
//! }
//! ```
//!
//! [`Avatar`], [`Badge`], and [`Card`] take a look in `new` and implement
//! [`gpui::IntoElement`]; rebuild them in the owning view's render method.
//!
//! # Choose the customization scope
//!
//! | Need | Extension point |
//! | --- | --- |
//! | Standard control appearance | Builder options such as `variant`, `size`, `tone`, and `radius`, where supported |
//! | Application-specific label/icon composition | Presenter via [`gpui_luma::infra::presenter::HasPresenter::content`] |
//! | Local button/tab layout | [`Button::with_template_modifier`] or [`Tabs::with_template_modifier`] |
//! | Local display styling | [`Avatar::style_override`] and [`Card::style_override`] |
//! | Local toolbar geometry | [`Toolbar::style`], [`Toolbar::command_size`], [`Toolbar::icon_size`] |
//! | Shared tab or Classic button styling | [`Look::set_tabs_style`] or [`Look::set_classic_params`] |
//! | Custom SDK integration or forced-state previews | Exported `*_theme`, `*_template`, and resolver functions; [`LookControlExt`] |
//!
//! Keep profile content, selected-label emphasis, and other application-specific
//! presentation in the application. Local overrides do not mutate the shared look.
//! Avatar/Card overrides receive resolved styles: change only the fields needed
//! so the remaining colors still follow mode and palette changes.
//!
//! ```
//! use gpui_luma_look_radix::{Card, CardVariant, Look};
//! let look = Look::built_in();
//! let card = Card::new(&look).variant(CardVariant::Classic)
//!     .style_override(|style| style.padding = 20.0);
//! assert_eq!(card.resolve_style().padding, 20.0);
//! ```
//!
//! # Palettes, ownership, and redraws
//!
//! [`Accent`] and [`Gray`] select named palettes; [`Tone`] chooses accent or neutral
//! control colors. [`ScaleFamily`] and [`SemanticRole`] provide direct resolution.
//! [`generate_colors`] adapts [`CustomColors`] into sRGB scales; GPUI rendering here
//! does not use Display P3. [`Look::set_custom_colors`] updates only the active mode.
//!
//! Cloning a look shares its state. [`Look::fork`] copies it independently for a
//! preview or editor. Shared settings affect controls bound to that same look;
//! control-local overrides remain local.
//!
//! Setters update state and revision, **not GPUI notifications**. Notify affected
//! control entities and their owning view after changes. See [`Look`]'s compiling
//! redraw example. A revision is a cache key, not a redraw subscription.
//!
//! # Source organization
//!
//! Palette state and resolution live in `look`, `palette`, `scale`, `semantic`, and
//! `custom_colors`. Each interactive control's `*_builder` module adapts public
//! options to SDK construction; its companion module resolves styling/templates.
//! Display controls keep their element and resolved style together. All public
//! entry points are re-exported here; private modules are implementation details.

mod avatar;
mod badge;
mod button;
mod button_builder;
mod button_layout;
mod callout;
mod card;
mod checkbox;
mod checkbox_builder;
mod colors;
mod custom_colors;
mod context_menu;
mod context_menu_builder;
mod ext;
mod look;
mod overlay_window;
mod palette;
mod popup_menu;
mod popup_menu_builder;
mod progress;
mod progress_builder;
mod radio;
mod radio_builder;
mod scale;
mod segmented;
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
mod tooltip;
mod toolbar;
mod toolbar_builder;
mod tree_view;
mod typography;

// Palette state, scales, and shared styling.
pub use look::{PageBackground, Look};
pub use palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
pub use scale::{CUSTOM_PALETTE, ColorScale, ModeScales, SCALE_LEN, ScaleFamily, ScalePair, ScaleStep};
pub use colors::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors::{RawColorScale, ColorValueKind, families as color_families, parse_color};
pub use custom_colors::{CustomColors, GeneratedColors, generate_colors};
pub use semantic::{SemanticMapping, SemanticRole};
pub use tone::Tone;
pub use button_layout::{ButtonSize, Radius, button_box_for, metric_tokens, resolve_button_radius};

// Controls and their styling adapters (alphabetical).
pub use avatar::{Avatar, AvatarSize, AvatarStyle, AvatarVariant};
pub use badge::{Badge, BadgeSize, BadgeVariant};
pub use button::{
    ClassicButtonParams, Paint, ButtonVariant, button_family_theme, button_family_theme_with, button_look_for,
    button_template, classic_button_template, classic_button_template_with,
};
pub use button_builder::Button;
pub use callout::callout;
pub use context_menu::{ContextMenuVariant, context_menu_template, context_menu_theme};
pub use context_menu_builder::ContextMenu;
pub use card::{Card, CardSize, CardStyle, CardVariant};
pub use checkbox::{
    CheckboxSize, CheckboxVariant, checkbox_scale_for, checkbox_template, checkbox_template_for, checkbox_theme,
    checkbox_theme_for, checkbox_theme_with, resolve_checkbox_radius,
};
pub use checkbox_builder::Checkbox;
pub use popup_menu::{PopupMenuVariant, popup_menu_template, popup_menu_theme};
pub use popup_menu_builder::PopupMenu;
pub use progress::{
    ProgressSize, ProgressVariant, progress_template, progress_theme, progress_theme_with, resolve_progress_radius,
};
pub use progress_builder::Progress;
pub use radio::{
    RadioSize, RadioVariant, radio_scale_for, radio_template, radio_template_for, radio_theme, radio_theme_for,
    radio_theme_with,
};
pub use radio_builder::Radio;
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
pub use toolbar::{ToolbarStyle, toolbar_template, toolbar_template_with, toolbar_theme, toolbar_theme_with};
pub use toolbar_builder::Toolbar;
pub use tree_view::{TreeView, tree_view_frame, tree_view_template, tree_view_theme};

// SDK composition helpers.
pub use ext::LookControlExt;
pub use overlay_window::overlay_window_theme;
pub use segmented::segmented_radio_template;

pub use tooltip::tooltip_theme;
