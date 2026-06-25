mod control;
mod model;
mod template;
mod theme;

pub(crate) use control::DialogControl;
pub(crate) use model::{
    DialogBuilder, DialogDismissPolicy, DialogEvent, DialogMode, DialogModel, DialogPosition, DialogRenderModel,
};
pub(crate) use template::{DialogTemplate, DialogTemplateHandlers, DialogTemplateParts, default_dialog_template};
pub(crate) use theme::{DialogLook, DialogTheme, default_dialog_theme};

pub use control::DialogControl as OverlayWindowControl;
pub use model::{
    DialogBuilder as OverlayWindowBuilder, DialogDismissPolicy as OverlayWindowDismissPolicy,
    DialogElementRenderer as OverlayWindowElementRenderer, DialogEvent as OverlayWindowEvent,
    DialogMode as OverlayWindowMode, DialogModel as OverlayWindowModel, DialogPosition as OverlayWindowPosition,
    DialogRenderModel as OverlayWindowRenderModel, new as overlay_window,
};
pub use template::{
    DialogTemplate as OverlayWindowTemplate, DialogTemplateHandlers as OverlayWindowTemplateHandlers,
    DialogTemplateParts as OverlayWindowTemplateParts, ThemedDialogTemplate as ThemedOverlayWindowTemplate,
    default_dialog_template as default_overlay_window_template,
};
pub use theme::{
    DefaultDialogTheme as DefaultOverlayWindowTheme, DialogLook as OverlayWindowLook,
    DialogTheme as OverlayWindowTheme, default_dialog_theme as default_overlay_window_theme,
};

use gpui::Entity;

pub type OverlayWindow = Entity<OverlayWindowControl>;
