//! Icon toolbar preset over [`control_group`](crate::controls::control_group).

use std::sync::Arc;

use gpui::{SharedString, Styled, px};

pub use crate::controls::control_group::{
    ControlGroupBuilder, ControlGroupControl, ControlGroupEvent, ControlGroupItem, ControlGroupItemLike,
    ControlGroupTheme,
};
use crate::controls::control_group::{self as control_group, control_group_template_with_theme};

pub type IconGroup<T> = gpui::Entity<ControlGroupControl<T>>;
pub type IconGroupEvent = ControlGroupEvent;
pub type IconGroupBuilder<T> = ControlGroupBuilder<T>;
pub type IconGroupItem = ControlGroupItem;
pub use ControlGroupItemLike as IconGroupItemLike;

pub fn new<T>(id: impl Into<SharedString>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::new(id).single_allow_none()
}

/// Horizontal icon toolbar with themed group chrome (border, background, padding).
pub fn horizontal<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar(id, theme)
}

/// Icon toolbar preset: horizontal layout, live-themed housing, rounded group container.
pub fn icon_toolbar<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar_with_mode(id, theme, false)
}

/// Multi-select icon toolbar (same chrome as [`icon_toolbar`]).
pub fn icon_toolbar_multiple<T>(id: impl Into<SharedString>, theme: Arc<dyn ControlGroupTheme>) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    icon_toolbar_with_mode(id, theme, true)
}

fn icon_toolbar_with_mode<T>(
    id: impl Into<SharedString>,
    theme: Arc<dyn ControlGroupTheme>,
    multiple: bool,
) -> IconGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    let mut builder = new(id)
        .horizontal()
        .template(control_group_template_with_theme(theme))
        .with_template_modifier(|element, _| element.rounded_full().gap(px(6.0)).px(px(6.0)).py(px(4.0)));

    if multiple {
        builder = builder.multiple();
    }

    builder
}
