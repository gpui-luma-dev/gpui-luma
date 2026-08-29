//! Synchronized rounded layers for GPUI compositions.
//!
//! GPUI's descendant overflow mask is rectangular. A rounded parent therefore
//! cannot be relied on to clip a square background painted by a child. This
//! primitive keeps the layer roots aligned and applies the same radius to each
//! layer root, which is the boundary that owns that layer's paint.

use gpui::{AnyElement, Corners, IntoElement, ParentElement, Pixels, RenderOnce, div, prelude::*};

use crate::controls::color::style::StyledExt;

/// A full-size, ordered stack of rounded layers.
///
/// Layers are painted in insertion order, with later layers above earlier
/// layers. Each layer is wrapped in an absolute, full-size rounded root. A
/// layer that contains further painted descendants must apply the same radius
/// to its own painted root; [`LayerStack`] cannot turn GPUI's rectangular
/// descendant overflow mask into a rounded mask.
#[derive(IntoElement)]
pub struct LayerStack {
    children: Vec<AnyElement>,
    corners: Corners<Pixels>,
    absolute: bool,
}

impl LayerStack {
    /// Creates an empty layer stack with the shared corner radius.
    pub fn new(radius: impl Into<Pixels>) -> Self {
        Self { children: Vec::new(), corners: Corners::all(radius.into()), absolute: false }
    }

    /// Returns the corner geometry used for every layer root.
    pub fn corners(&self) -> Corners<Pixels> {
        self.corners
    }

    /// Sets independent radii for each corner.
    pub fn corner_radii(mut self, corners: Corners<Pixels>) -> Self {
        self.corners = corners;
        self
    }

    /// Adds a layer above the previously-added layers.
    pub fn layer(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Returns the number of layers currently in the stack.
    pub fn layer_count(&self) -> usize {
        self.children.len()
    }

    /// Positions the stack over its containing box.
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
        self
    }
}

impl ParentElement for LayerStack {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for LayerStack {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let corners = self.corners;
        let mut root = div().relative().w_full().h_full().corner_radii(corners);
        if self.absolute {
            root = root.absolute().inset_0();
        }
        root.children(
            self.children.into_iter().map(|child| div().absolute().inset_0().corner_radii(corners).child(child)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    #[test]
    fn layer_stack_preserves_radius_and_ordered_layer_count() {
        let stack = LayerStack::new(px(12.0)).layer(div()).layer(div());

        assert_eq!(stack.corners(), Corners::all(px(12.0)));
        assert_eq!(stack.layer_count(), 2);
    }
}
