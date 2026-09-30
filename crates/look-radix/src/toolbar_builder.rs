//! Look-owned toolbar composition, including its icon-button recipe.

use gpui::{Context, SharedString};
use luma::controls::button::ControlIcon;
use luma::controls::button_family::ButtonFamilyRole;
use luma::controls::toolbar::{ToolbarBuilder, ToolbarItem};
use luma::theme::ControlSize;

use crate::{Button, ButtonSize, Look, toolbar_template};

const ICON_SIZE: f32 = 16.0;

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
}

impl Toolbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { id: id.into(), look: None, items: Vec::new() }
    }

    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::toolbar::Toolbar {
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
                        .size(ButtonSize::One)
                        .role(ButtonFamilyRole::Icon)
                        .tab_stop(false)
                        .icon(icon)
                        .icon_size(ICON_SIZE)
                        .spawn(cx);
                    ToolbarItem::command_button(id, button, cx).label(label)
                }
            })
            .collect::<Vec<_>>();
        ToolbarBuilder::new(self.id)
            .template(toolbar_template(&look))
            .size(ControlSize::Sm)
            .items(items)
            .spawn(cx)
    }
}
