//! Look-owned toolbar composition, including its icon-button recipe.

use gpui::{Context, SharedString};
use gpui_luma::controls::button::ControlIcon;
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::toolbar::{ToolbarBuilder, ToolbarItem};
use gpui_luma::theme::ControlSize;

use crate::{Button, ButtonSize, Look, ToolbarStyle, toolbar_template_with};

enum Item {
    Command { id: SharedString, label: SharedString, icon: ControlIcon },
    Hosted(ToolbarItem),
}

/// Radix toolbar with compact, neutral icon buttons and a static outline.
/// Command IDs, labels, icons, and hosted controls are supplied by the application.
pub struct Toolbar {
    id: SharedString,
    look: Option<Look>,
    items: Vec<Item>,
    style: Option<ToolbarStyle>,
    size: ControlSize,
    command_size: ButtonSize,
    icon_size: f32,
}

impl Toolbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            items: Vec::new(),
            style: None,
            size: ControlSize::Sm,
            command_size: ButtonSize::One,
            icon_size: 16.0,
        }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Shell spacing and radius; applies to this toolbar only.
    pub fn style(mut self, style: ToolbarStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// SDK shell size, including the reference height for separators.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Size of buttons created by `command`; hosted items keep their own sizing.
    pub fn command_size(mut self, size: ButtonSize) -> Self {
        self.command_size = size;
        self
    }

    /// Logical icon size for generated command buttons. Hosted items are unchanged.
    pub fn icon_size(mut self, size: f32) -> Self {
        if size.is_finite() && size >= 0.0 {
            self.icon_size = size;
        }
        self
    }

    /// Add a command using the look's icon size, hover fill, and focus-border policy.
    pub fn command(
        mut self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<ControlIcon>,
    ) -> Self {
        self.items.push(Item::Command { id: id.into(), label: label.into(), icon: icon.into() });
        self
    }

    /// Host an existing SDK toolbar item, retaining its event handlers.
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(Item::Hosted(item));
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> gpui_luma::controls::toolbar::Toolbar {
        let look = crate::look::resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        let items = self
            .items
            .into_iter()
            .map(|item| match item {
                Item::Hosted(item) => item,
                Item::Command { id, label, icon } => {
                    let button = Button::new(id.clone())
                        .look(&look)
                        .ghost_quiet()
                        .gray()
                        .size(self.command_size)
                        .role(ButtonFamilyRole::Icon)
                        .tab_stop(false)
                        .icon(icon)
                        .icon_size(self.icon_size)
                        .spawn(cx);
                    ToolbarItem::command_button(id, button, cx).label(label)
                }
            })
            .collect::<Vec<_>>();
        ToolbarBuilder::new(self.id)
            .template(match self.style {
                Some(style) => toolbar_template_with(&look, style),
                None => crate::toolbar_template(&look),
            })
            .size(self.size)
            .items(items)
            .spawn(cx)
    }
}
