#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
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
