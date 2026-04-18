use gpui::{App, KeyBinding, actions};

actions!(
    luma_controls,
    [
        ActivateControl,
        OpenControl,
        CommitSelection,
        SelectNextItem,
        SelectPreviousItem,
        SelectFirstItem,
        SelectLastItem,
        OpenSubmenu,
        CloseSubmenu,
        IncreaseValue,
        DecreaseValue,
        IncreaseValueLarge,
        DecreaseValueLarge,
        MoveToStart,
        MoveToEnd,
        OpenContextMenu
    ]
);

pub const LUMA_COMMAND_CONTEXT: &str = "LumaCommandControl";
pub const LUMA_CHOICE_CONTEXT: &str = "LumaChoiceControl";
pub const LUMA_RADIO_GROUP_CONTEXT: &str = "LumaRadioGroup";
pub const LUMA_SLIDER_CONTEXT: &str = "LumaSlider";
pub const LUMA_MENU_CONTROL_CONTEXT: &str = "LumaMenuControl";
pub const LUMA_CONTEXT_MENU_CONTROL_CONTEXT: &str = "LumaContextMenuControl";

pub fn bind_default_control_keys(cx: &mut App) {
    cx.bind_keys(default_control_key_bindings());
}

fn default_control_key_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("enter", ActivateControl, Some(LUMA_COMMAND_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(LUMA_COMMAND_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(LUMA_CHOICE_CONTEXT)),
        KeyBinding::new("left", SelectPreviousItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("up", SelectPreviousItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("right", SelectNextItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("down", SelectNextItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("home", SelectFirstItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("end", SelectLastItem, Some(LUMA_RADIO_GROUP_CONTEXT)),
        KeyBinding::new("left", DecreaseValue, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("down", DecreaseValue, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("right", IncreaseValue, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("up", IncreaseValue, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("pagedown", DecreaseValueLarge, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("pageup", IncreaseValueLarge, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("home", MoveToStart, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("end", MoveToEnd, Some(LUMA_SLIDER_CONTEXT)),
        KeyBinding::new("down", SelectNextItem, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("up", SelectPreviousItem, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("home", SelectFirstItem, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("end", SelectLastItem, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("right", OpenSubmenu, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("left", CloseSubmenu, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("enter", ActivateControl, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(LUMA_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("shift-f10", OpenContextMenu, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("menu", OpenContextMenu, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("down", SelectNextItem, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("up", SelectPreviousItem, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("home", SelectFirstItem, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("end", SelectLastItem, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("right", OpenSubmenu, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("left", CloseSubmenu, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("enter", ActivateControl, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(LUMA_CONTEXT_MENU_CONTROL_CONTEXT)),
    ]
}

#[cfg(test)]
mod tests {
    use super::default_control_key_bindings;

    #[test]
    fn default_control_key_bindings_are_parseable() {
        assert_eq!(default_control_key_bindings().len(), 35);
    }
}
