use gpui::{AnyElement, Div, IntoElement, div, px, prelude::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    #[default]
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexAlign {
    Start,
    End,
    Center,
    Baseline,
    Stretch,
    FlexStart,
    FlexEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexJustify {
    Start,
    End,
    Center,
    Between,
    Around,
    Evenly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexItemSize {
    pub basis: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub grow: f32,
    pub shrink: f32,
}

impl FlexItemSize {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn card_fill(preferred_width: f32, min_width: f32, max_width: f32) -> Self {
        Self::new().basis(preferred_width).min_width(min_width).max_width(max_width).grow(1.0).shrink(1.0)
    }

    pub fn basis(mut self, basis: f32) -> Self {
        self.basis = Some(basis);
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = grow;
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = shrink;
        self
    }
}

impl Default for FlexItemSize {
    fn default() -> Self {
        Self {
            basis: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            grow: 0.0,
            shrink: 1.0,
        }
    }
}

pub struct FlexChild {
    pub element: AnyElement,
    pub size: FlexItemSize,
    pub align_self: Option<FlexAlign>,
}

impl FlexChild {
    pub fn new(element: impl IntoElement) -> Self {
        Self { element: element.into_any_element(), size: FlexItemSize::default(), align_self: None }
    }

    pub fn size(mut self, size: FlexItemSize) -> Self {
        self.size = size;
        self
    }

    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.align_self = Some(align_self);
        self
    }
}

pub struct FlexLayout {
    direction: FlexDirection,
    wrap: FlexWrap,
    gap_x: f32,
    gap_y: f32,
    align_items: Option<FlexAlign>,
    justify_content: Option<FlexJustify>,
    children: Vec<FlexChild>,
}

impl FlexLayout {
    pub fn new() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::Wrap,
            gap_x: 0.0,
            gap_y: 0.0,
            align_items: None,
            justify_content: None,
            children: Vec::new(),
        }
    }

    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap_x = gap;
        self.gap_y = gap;
        self
    }

    pub fn gap_x(mut self, gap_x: f32) -> Self {
        self.gap_x = gap_x;
        self
    }

    pub fn gap_y(mut self, gap_y: f32) -> Self {
        self.gap_y = gap_y;
        self
    }

    pub fn align_items(mut self, align_items: FlexAlign) -> Self {
        self.align_items = Some(align_items);
        self
    }

    pub fn justify_content(mut self, justify_content: FlexJustify) -> Self {
        self.justify_content = Some(justify_content);
        self
    }

    pub fn child(mut self, element: impl IntoElement, size: FlexItemSize) -> Self {
        self.children.push(FlexChild::new(element).size(size));
        self
    }

    pub fn flex_child(mut self, child: FlexChild) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = FlexChild>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn build(self) -> Div {
        let FlexLayout { direction, wrap, gap_x, gap_y, align_items, justify_content, children } = self;

        let mut root = div().w_full().min_w(px(0.0)).flex();

        root.style().gap.width = Some(px(gap_x).into());
        root.style().gap.height = Some(px(gap_y).into());
        root = apply_direction(root, direction);
        root = apply_wrap(root, wrap);

        if let Some(align_items) = align_items {
            root = apply_align(root, align_items);
        }

        if let Some(justify_content) = justify_content {
            root = apply_justify(root, justify_content);
        }

        for child in children {
            root = root.child(build_flex_child(child));
        }

        root
    }

    pub fn into_any_element(self) -> AnyElement {
        self.build().into_any_element()
    }
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for FlexLayout {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        self.build()
    }
}

fn apply_direction(panel: Div, direction: FlexDirection) -> Div {
    match direction {
        FlexDirection::Row => panel.flex_row(),
        FlexDirection::Column => panel.flex_col(),
    }
}

fn apply_wrap(panel: Div, wrap: FlexWrap) -> Div {
    match wrap {
        FlexWrap::NoWrap => panel.flex_nowrap(),
        FlexWrap::Wrap => panel.flex_wrap(),
        FlexWrap::WrapReverse => panel.flex_wrap_reverse(),
    }
}

fn apply_align(panel: Div, align: FlexAlign) -> Div {
    match align {
        FlexAlign::Start | FlexAlign::FlexStart => panel.items_start(),
        FlexAlign::End | FlexAlign::FlexEnd => panel.items_end(),
        FlexAlign::Center => panel.items_center(),
        FlexAlign::Baseline => panel.items_baseline(),
        FlexAlign::Stretch => panel.items_stretch(),
    }
}

fn apply_justify(panel: Div, justify: FlexJustify) -> Div {
    match justify {
        FlexJustify::Start => panel.justify_start(),
        FlexJustify::End => panel.justify_end(),
        FlexJustify::Center => panel.justify_center(),
        FlexJustify::Between => panel.justify_between(),
        FlexJustify::Around => panel.justify_around(),
        FlexJustify::Evenly => panel.justify_evenly(),
    }
}

fn build_flex_child(child: FlexChild) -> Div {
    let FlexChild { element, size, align_self } = child;
    let FlexItemSize { basis, min_width, max_width, min_height, max_height, grow, shrink } = size;

    let mut wrapper = div().min_w(px(0.0));
    wrapper.style().flex_grow = Some(grow);
    wrapper.style().flex_shrink = Some(shrink);

    if let Some(basis) = basis {
        wrapper = wrapper.flex_basis(px(basis));
    }
    if let Some(min_width) = min_width {
        wrapper = wrapper.min_w(px(min_width));
    }
    if let Some(max_width) = max_width {
        wrapper = wrapper.max_w(px(max_width));
    }
    if let Some(min_height) = min_height {
        wrapper = wrapper.min_h(px(min_height));
    }
    if let Some(max_height) = max_height {
        wrapper = wrapper.max_h(px(max_height));
    }

    if let Some(align_self) = align_self {
        wrapper = apply_align_self(wrapper, align_self);
    }

    wrapper.child(element)
}

fn apply_align_self(panel: Div, align: FlexAlign) -> Div {
    match align {
        FlexAlign::Start => panel.self_start(),
        FlexAlign::End => panel.self_end(),
        FlexAlign::Center => panel.self_center(),
        FlexAlign::Baseline => panel.self_baseline(),
        FlexAlign::Stretch => panel.self_stretch(),
        FlexAlign::FlexStart => panel.self_flex_start(),
        FlexAlign::FlexEnd => panel.self_flex_end(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FlexAlign, FlexChild, FlexDirection, FlexItemSize, FlexJustify, FlexLayout, FlexWrap};
    use gpui::{IntoElement, div};

    fn assert_into_element<E: IntoElement>(_element: E) {}

    #[test]
    fn flex_layout_is_into_element() {
        assert_into_element(
            FlexLayout::new()
                .direction(FlexDirection::Row)
                .wrap(FlexWrap::Wrap)
                .gap(16.0)
                .align_items(FlexAlign::Stretch)
                .justify_content(FlexJustify::Between)
                .child(div(), FlexItemSize::card_fill(320.0, 260.0, 420.0)),
        );
    }

    #[test]
    fn card_fill_policy_sets_expected_defaults() {
        let size = FlexItemSize::card_fill(320.0, 260.0, 420.0);
        assert_eq!(size.basis, Some(320.0));
        assert_eq!(size.min_width, Some(260.0));
        assert_eq!(size.max_width, Some(420.0));
        assert_eq!(size.grow, 1.0);
        assert_eq!(size.shrink, 1.0);
    }

    #[test]
    fn flex_layout_collects_children() {
        let layout = FlexLayout::new()
            .children([FlexChild::new(div()), FlexChild::new(div()).size(FlexItemSize::new().grow(1.0))]);
        assert_eq!(layout.children.len(), 2);
    }
}
