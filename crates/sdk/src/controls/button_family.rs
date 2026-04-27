use crate::theme::{ControlSize, InteractionState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonKind {
    #[default]
    Standard,
    Ghost,
    Prominent,
}

pub type ButtonSize = ControlSize;
pub type ButtonInteractionState = InteractionState;
