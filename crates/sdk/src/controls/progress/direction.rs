#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProgressDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}

impl ProgressDirection {
    pub fn orientation(self) -> ProgressOrientation {
        match self {
            Self::LeftToRight | Self::RightToLeft => ProgressOrientation::Horizontal,
            Self::BottomToTop | Self::TopToBottom => ProgressOrientation::Vertical,
        }
    }

    pub fn is_reversed(self) -> bool {
        matches!(self, Self::RightToLeft | Self::TopToBottom)
    }
}

pub fn display_position(position: f32, reversed: bool) -> f32 {
    let position = position.clamp(0.0, 1.0);
    if reversed { 1.0 - position } else { position }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orientation_from_direction() {
        assert_eq!(ProgressDirection::LeftToRight.orientation(), ProgressOrientation::Horizontal);
        assert_eq!(ProgressDirection::BottomToTop.orientation(), ProgressOrientation::Vertical);
    }

    #[test]
    fn display_position_mirrors_when_reversed() {
        assert_eq!(display_position(0.25, false), 0.25);
        assert_eq!(display_position(0.25, true), 0.75);
    }

    #[test]
    fn direction_inversion_formulas() {
        let val = 0.4;
        assert_eq!(display_position(val, ProgressDirection::LeftToRight.is_reversed()), val);
        assert_eq!(display_position(val, ProgressDirection::RightToLeft.is_reversed()), 1.0 - val);
        assert_eq!(display_position(val, ProgressDirection::BottomToTop.is_reversed()), val);
        assert_eq!(display_position(val, ProgressDirection::TopToBottom.is_reversed()), 1.0 - val);
    }
}
