#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub invalid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionLayer {
    Default,
    Hovered,
    Pressed,
    Disabled,
}

impl InteractionState {
    pub fn layer(self) -> InteractionLayer {
        if self.disabled {
            InteractionLayer::Disabled
        } else if self.pressed {
            InteractionLayer::Pressed
        } else if self.hovered {
            InteractionLayer::Hovered
        } else {
            InteractionLayer::Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_layer_precedence_is_independent_of_focus_and_validation() {
        for bits in 0..32 {
            let state = InteractionState {
                disabled: bits & 1 != 0,
                pressed: bits & 2 != 0,
                hovered: bits & 4 != 0,
                focused: bits & 8 != 0,
                invalid: bits & 16 != 0,
            };
            let expected = if state.disabled {
                InteractionLayer::Disabled
            } else if state.pressed {
                InteractionLayer::Pressed
            } else if state.hovered {
                InteractionLayer::Hovered
            } else {
                InteractionLayer::Default
            };
            assert_eq!(state.layer(), expected);
        }
    }
}
