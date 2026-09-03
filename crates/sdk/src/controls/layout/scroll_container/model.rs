/// Whether the scrollbar reserves layout space or floats over content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarPlacement {
    /// Reserve a gutter beside the viewport (default).
    #[default]
    Inset,
    /// Float the scrollbar over the viewport without reserving layout space.
    Overlay,
}

/// When the scrollbar chrome is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Always show the scrollbar while content is scrollable (default).
    #[default]
    AlwaysVisible,
    /// Show according to [`ScrollbarAutoHideActivate`]; hide when idle / not hovered.
    AutoHide,
    /// Never show the scrollbar chrome.
    Hidden,
}

/// What reveals an auto-hiding scrollbar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarAutoHideActivate {
    /// Reveal while the pointer hovers the container.
    Hover,
    /// Reveal while scrolling (mouse wheel / trackpad); hide after motion stops.
    Move,
    /// Reveal on hover or scroll motion (default).
    #[default]
    HoverOrMove,
}

impl ScrollbarAutoHideActivate {
    pub(super) fn listens_to_hover(self) -> bool {
        matches!(self, Self::Hover | Self::HoverOrMove)
    }

    pub(super) fn listens_to_move(self) -> bool {
        matches!(self, Self::Move | Self::HoverOrMove)
    }
}
