use gpui::{AnyElement, Div, IntoElement, div, px, prelude::*};

/// A lightweight edge-docked layout primitive inspired by WPF's `DockPanel`.
///
/// `DockPanel` composes optional `top`, `bottom`, `left`, and `right` regions
/// around a required `fill` region that occupies the remaining space.
///
/// The resulting layout fills its parent (`size_full`) and applies the
/// `min_w(0)` / `min_h(0)` constraints needed for scrollable or clipped fill
/// content inside flex containers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DockSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct DockPanel<Fill = MissingFill> {
    docked: Vec<(DockSide, AnyElement)>,
    fill: Fill,
}

/// Marker type used until a `DockPanel` receives its required fill region.
pub struct MissingFill;

/// Marker type indicating the panel has a fill region and can be rendered.
pub struct Filled {
    element: AnyElement,
}

impl DockPanel<MissingFill> {
    pub fn new() -> Self {
        Self { docked: Vec::new(), fill: MissingFill }
    }
}

impl Default for DockPanel<MissingFill> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Fill> DockPanel<Fill> {
    pub fn top(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Top, element.into_any_element()));
        self
    }

    pub fn bottom(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Bottom, element.into_any_element()));
        self
    }

    pub fn left(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Left, element.into_any_element()));
        self
    }

    pub fn right(mut self, element: impl IntoElement) -> Self {
        self.docked.push((DockSide::Right, element.into_any_element()));
        self
    }

    pub fn fill(self, element: impl IntoElement) -> DockPanel<Filled> {
        DockPanel { docked: self.docked, fill: Filled { element: element.into_any_element() } }
    }
}

impl DockPanel<Filled> {
    pub fn build(self) -> Div {
        let DockPanel { docked, fill } = self;

        let mut root = stretch(fill.element);

        for (side, element) in docked.into_iter().rev() {
            root = match side {
                DockSide::Top => div()
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .flex()
                    .flex_col()
                    .child(element)
                    .child(remainder(root.into_any_element())),
                DockSide::Bottom => div()
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .flex()
                    .flex_col()
                    .child(remainder(root.into_any_element()))
                    .child(element),
                DockSide::Left => div()
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .flex()
                    .child(element)
                    .child(remainder(root.into_any_element())),
                DockSide::Right => div()
                    .size_full()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .flex()
                    .child(remainder(root.into_any_element()))
                    .child(element),
            };
        }

        root
    }

    pub fn into_any_element(self) -> AnyElement {
        self.build().into_any_element()
    }
}

impl IntoElement for DockPanel<Filled> {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        self.build()
    }
}

fn stretch(content: AnyElement) -> Div {
    div().size_full().min_w(px(0.0)).min_h(px(0.0)).child(content)
}

// `content` is already a full-size root:
// - the initial fill is wrapped with `stretch(...)`
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
    fn dock_panel_with_fill_is_into_element() {
        assert_into_element(DockPanel::new().fill(div()));
    }

    #[test]
    fn dock_panel_supports_regions_before_fill() {
        assert_into_element(DockPanel::new().top(div()).left(div()).right(div()).bottom(div()).fill(div()));
    }

    #[test]
    fn dock_panel_supports_regions_after_fill() {
        assert_into_element(DockPanel::new().fill(div()).top(div()).left(div()).right(div()).bottom(div()));
    }

    #[test]
    fn dock_panel_preserves_docking_order() {
        let panel = DockPanel::new().left(div()).top(div()).right(div()).bottom(div());
        let sides: Vec<_> = panel.docked.iter().map(|(side, _)| *side).collect();

        assert_eq!(sides, vec![DockSide::Left, DockSide::Top, DockSide::Right, DockSide::Bottom]);
    }
}
