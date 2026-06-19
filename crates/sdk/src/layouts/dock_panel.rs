use gpui::{AnyElement, Div, IntoElement, div, px, prelude::*};

/// A lightweight edge-docked layout primitive inspired by WPF's `DockPanel`.
///
/// `DockPanel` keeps children in declaration order. By default, the last child
/// stretches to fill the remaining space (`last_child_fill = true`), while all
/// earlier children dock to their requested side. Undocked children use WPF's
/// default dock behavior (`Left`) when they are not the filling last child.
///
/// The resulting layout fills its parent (`size_full`) and applies the
/// `min_w(0)` / `min_h(0)` constraints needed for scrollable or clipped
/// remainder content inside flex containers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DockSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct DockPanel {
    children: Vec<(Option<DockSide>, AnyElement)>,
    last_child_fill: bool,
}

impl DockPanel {
    pub fn new() -> Self {
        Self { children: Vec::new(), last_child_fill: true }
    }

    pub fn last_child_fill(mut self, last_child_fill: bool) -> Self {
        self.last_child_fill = last_child_fill;
        self
    }

    pub fn top(mut self, element: impl IntoElement) -> Self {
        self.children.push((Some(DockSide::Top), element.into_any_element()));
        self
    }

    pub fn bottom(mut self, element: impl IntoElement) -> Self {
        self.children.push((Some(DockSide::Bottom), element.into_any_element()));
        self
    }

    pub fn left(mut self, element: impl IntoElement) -> Self {
        self.children.push((Some(DockSide::Left), element.into_any_element()));
        self
    }

    pub fn right(mut self, element: impl IntoElement) -> Self {
        self.children.push((Some(DockSide::Right), element.into_any_element()));
        self
    }

    pub fn child(mut self, element: impl IntoElement) -> Self {
        self.children.push((None, element.into_any_element()));
        self
    }

    pub fn fill(self, element: impl IntoElement) -> Self {
        self.child(element)
    }

    pub fn build(self) -> Div {
        let DockPanel { mut children, last_child_fill } = self;

        if children.is_empty() {
            return stretch(div().into_any_element());
        }

        let mut root = if last_child_fill {
            let (_, element) = children.pop().expect("checked non-empty dock panel children");
            stretch(element)
        } else {
            stretch(div().into_any_element())
        };

        for (side, element) in children.into_iter().rev() {
            root = dock_child(side.unwrap_or(DockSide::Left), element, root.into_any_element());
        }

        root
    }

    pub fn into_any_element(self) -> AnyElement {
        self.build().into_any_element()
    }
}

impl Default for DockPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for DockPanel {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        self.build()
    }
}

fn dock_child(side: DockSide, element: AnyElement, remainder_content: AnyElement) -> Div {
    match side {
        DockSide::Top => div()
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .child(element)
            .child(remainder(remainder_content)),
        DockSide::Bottom => div()
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .child(remainder(remainder_content))
            .child(element),
        DockSide::Left => div()
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .child(element)
            .child(remainder(remainder_content)),
        DockSide::Right => div()
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .child(remainder(remainder_content))
            .child(element),
    }
}

fn stretch(content: AnyElement) -> Div {
    div().size_full().min_w(px(0.0)).min_h(px(0.0)).child(content)
}

// `content` is already a full-size root:
// - the initial fill host is wrapped with `stretch(...)`
// - each dock wrapper returns `size_full()`
// so the remainder host only needs to provide flex growth and shrink constraints.
fn remainder(content: AnyElement) -> Div {
    div().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(content)
}

#[cfg(test)]
mod tests {
    use super::{DockPanel, DockSide};
    use gpui::{IntoElement, div};

    fn assert_into_element<E: IntoElement>(_element: E) {}

    #[test]
    fn empty_dock_panel_is_into_element() {
        assert_into_element(DockPanel::new());
    }

    #[test]
    fn dock_panel_with_only_docked_children_is_into_element() {
        assert_into_element(DockPanel::new().top(div()).left(div()).right(div()).bottom(div()));
    }

    #[test]
    fn dock_panel_supports_plain_children() {
        assert_into_element(DockPanel::new().left(div()).child(div()).right(div()));
    }

    #[test]
    fn dock_panel_preserves_child_order_and_dock_metadata() {
        let panel = DockPanel::new().left(div()).child(div()).top(div()).fill(div()).right(div());
        let sides: Vec<_> = panel.children.iter().map(|(side, _)| *side).collect();

        assert_eq!(sides, vec![Some(DockSide::Left), None, Some(DockSide::Top), None, Some(DockSide::Right)]);
    }

    #[test]
    fn dock_panel_defaults_last_child_fill_to_true() {
        assert!(DockPanel::new().last_child_fill);
    }

    #[test]
    fn dock_panel_can_disable_last_child_fill() {
        assert!(!DockPanel::new().last_child_fill(false).last_child_fill);
    }
}
