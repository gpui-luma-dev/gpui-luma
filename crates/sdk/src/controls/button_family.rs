use crate::theme::{ControlSize, InteractionState};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonKind {
    #[default]
    Default,
    Primary,
    Destructive,
}

pub type ButtonSize = ControlSize;
pub type ButtonInteractionState = InteractionState;
