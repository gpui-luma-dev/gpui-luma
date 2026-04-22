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

const COMMAND_CONTEXT: &str = "LumaCommandControl";
const CHOICE_CONTEXT: &str = "LumaChoiceControl";
const RADIO_GROUP_CONTEXT: &str = "LumaRadioGroup";
const RANGE_VALUE_CONTEXT: &str = "LumaRangeValue";
const SCROLL_OFFSET_CONTEXT: &str = "LumaScrollOffset";
const MENU_CONTROL_CONTEXT: &str = "LumaMenuControl";
const CONTEXT_MENU_CONTROL_CONTEXT: &str = "LumaContextMenuControl";
const NAVIGATION_CONTEXT: &str = "LumaNavigationControl";
const TAB_LIST_CONTEXT: &str = "LumaTabList";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ControlKeyProfile {
    Command,
    Choice,
    RadioGroup,
    RangeValue,
    ScrollOffset,
    Menu,
    ContextMenu,
    Navigation,
    TabList,
}

impl ControlKeyProfile {
    pub const fn context(self) -> &'static str {
        match self {
            Self::Command => COMMAND_CONTEXT,
            Self::Choice => CHOICE_CONTEXT,
            Self::RadioGroup => RADIO_GROUP_CONTEXT,
            Self::RangeValue => RANGE_VALUE_CONTEXT,
            Self::ScrollOffset => SCROLL_OFFSET_CONTEXT,
            Self::Menu => MENU_CONTROL_CONTEXT,
            Self::ContextMenu => CONTEXT_MENU_CONTROL_CONTEXT,
            Self::Navigation => NAVIGATION_CONTEXT,
            Self::TabList => TAB_LIST_CONTEXT,
        }
    }

    pub fn default_bindings(self) -> Vec<KeyBinding> {
        let context = self.context();

        match self {
            Self::Command => vec![
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::Choice => vec![KeyBinding::new("space", ActivateControl, Some(context))],
            Self::RadioGroup => vec![
                KeyBinding::new("left", SelectPreviousItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("right", SelectNextItem, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::RangeValue => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("down", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("up", IncreaseValue, Some(context)),
                KeyBinding::new("pagedown", DecreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", IncreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
            ],
            Self::ScrollOffset => vec![
                KeyBinding::new("left", DecreaseValue, Some(context)),
                KeyBinding::new("up", DecreaseValue, Some(context)),
                KeyBinding::new("right", IncreaseValue, Some(context)),
                KeyBinding::new("down", IncreaseValue, Some(context)),
                KeyBinding::new("pagedown", IncreaseValueLarge, Some(context)),
                KeyBinding::new("pageup", DecreaseValueLarge, Some(context)),
                KeyBinding::new("home", MoveToStart, Some(context)),
                KeyBinding::new("end", MoveToEnd, Some(context)),
            ],
            Self::Menu => vec![
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::ContextMenu => vec![
                KeyBinding::new("shift-f10", OpenContextMenu, Some(context)),
                KeyBinding::new("menu", OpenContextMenu, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::Navigation => vec![
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("right", OpenSubmenu, Some(context)),
                KeyBinding::new("left", CloseSubmenu, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
            Self::TabList => vec![
                KeyBinding::new("left", SelectPreviousItem, Some(context)),
                KeyBinding::new("up", SelectPreviousItem, Some(context)),
                KeyBinding::new("right", SelectNextItem, Some(context)),
                KeyBinding::new("down", SelectNextItem, Some(context)),
                KeyBinding::new("home", SelectFirstItem, Some(context)),
                KeyBinding::new("end", SelectLastItem, Some(context)),
                KeyBinding::new("enter", ActivateControl, Some(context)),
                KeyBinding::new("space", ActivateControl, Some(context)),
            ],
        }
    }
}

pub fn bind_default_control_keys(cx: &mut App) {
    cx.bind_keys(default_control_key_bindings());
}

pub fn default_control_key_bindings() -> Vec<KeyBinding> {
    [
        ControlKeyProfile::Command,
        ControlKeyProfile::Choice,
        ControlKeyProfile::RadioGroup,
        ControlKeyProfile::RangeValue,
        ControlKeyProfile::ScrollOffset,
        ControlKeyProfile::Menu,
        ControlKeyProfile::ContextMenu,
        ControlKeyProfile::Navigation,
        ControlKeyProfile::TabList,
    ]
    .into_iter()
    .flat_map(ControlKeyProfile::default_bindings)
    .collect()
}

#[cfg(test)]
mod tests {
    use gpui::{Action, KeyBinding};

    use super::{
        ActivateControl, CloseSubmenu, ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue,
        IncreaseValueLarge, OpenSubmenu, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
        default_control_key_bindings,
    };

    #[test]
    fn default_control_key_bindings_are_parseable() {
        assert_eq!(default_control_key_bindings().len(), 61);
    }

    #[test]
    fn profile_binding_counts_are_stable() {
        assert_eq!(ControlKeyProfile::Command.default_bindings().len(), 2);
        assert_eq!(ControlKeyProfile::Choice.default_bindings().len(), 1);
        assert_eq!(ControlKeyProfile::RadioGroup.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::RangeValue.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::ScrollOffset.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::Menu.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::ContextMenu.default_bindings().len(), 10);
        assert_eq!(ControlKeyProfile::Navigation.default_bindings().len(), 8);
        assert_eq!(ControlKeyProfile::TabList.default_bindings().len(), 8);
    }

    #[test]
    fn range_value_and_scroll_offset_profiles_have_different_vertical_semantics() {
        let range = ControlKeyProfile::RangeValue.default_bindings();
        let scroll = ControlKeyProfile::ScrollOffset.default_bindings();

        assert!(has_binding::<DecreaseValue>(&range, "down"));
        assert!(has_binding::<IncreaseValueLarge>(&range, "pageup"));
        assert!(has_binding::<IncreaseValue>(&scroll, "down"));
        assert!(has_binding::<DecreaseValueLarge>(&scroll, "pageup"));
    }

    #[test]
    fn navigation_profile_binds_collection_accelerators() {
        let bindings = ControlKeyProfile::Navigation.default_bindings();

        assert!(has_binding::<SelectPreviousItem>(&bindings, "up"));
        assert!(has_binding::<SelectNextItem>(&bindings, "down"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "home"));
        assert!(has_binding::<SelectLastItem>(&bindings, "end"));
        assert!(has_binding::<OpenSubmenu>(&bindings, "right"));
        assert!(has_binding::<CloseSubmenu>(&bindings, "left"));
        assert!(has_binding::<ActivateControl>(&bindings, "enter"));
        assert!(has_binding::<ActivateControl>(&bindings, "space"));
    }

    #[test]
    fn tab_list_profile_binds_roving_tab_accelerators() {
        let bindings = ControlKeyProfile::TabList.default_bindings();

        assert!(has_binding::<SelectPreviousItem>(&bindings, "left"));
        assert!(has_binding::<SelectPreviousItem>(&bindings, "up"));
        assert!(has_binding::<SelectNextItem>(&bindings, "right"));
        assert!(has_binding::<SelectNextItem>(&bindings, "down"));
        assert!(has_binding::<SelectFirstItem>(&bindings, "home"));
        assert!(has_binding::<SelectLastItem>(&bindings, "end"));
        assert!(has_binding::<ActivateControl>(&bindings, "enter"));
        assert!(has_binding::<ActivateControl>(&bindings, "space"));
    }

    fn has_binding<A: Action>(bindings: &[KeyBinding], key: &str) -> bool {
        bindings.iter().any(|binding| {
            binding.keystrokes().len() == 1
                && binding.keystrokes()[0].key() == key
                && binding.action().as_any().is::<A>()
        })
    }
}
