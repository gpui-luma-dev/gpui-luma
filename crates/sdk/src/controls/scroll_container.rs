use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, Pixels, ScrollHandle, SharedString, Stateful, Window, div,
    point, prelude::*, px,
};

use crate::controls::scrollbar::{Scrollbar, ScrollbarTemplate};
use crate::controls::value::ControlRange;

#[derive(Clone)]
pub struct ScrollContainer {
    id: SharedString,
    scroll_handle: ScrollHandle,
    scrollbar: Entity<Scrollbar>,
    scrollbar_width: Pixels,
    last_max_scroll: Rc<Cell<f32>>,
}

impl ScrollContainer {
    pub fn new(
        id: impl Into<SharedString>,
        scrollbar_template: Arc<dyn ScrollbarTemplate>,
        cx: &mut impl AppContext,
    ) -> Self {
        let id = id.into();
        let scrollbar = Scrollbar::new(format!("{id}-scrollbar")).vertical().template(scrollbar_template).spawn(cx);

        Self {
            id,
            scroll_handle: ScrollHandle::new(),
            scrollbar,
            scrollbar_width: px(12.0),
            last_max_scroll: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn scrollbar(&self) -> Entity<Scrollbar> {
        self.scrollbar.clone()
    }

    pub fn set_vertical_offset<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    pub fn vertical_offset(&self) -> Pixels {
        px((-self.scroll_handle.offset().y.as_f32()).max(0.0))
    }

    pub fn max_vertical_offset(&self) -> Pixels {
        px(self.scroll_handle.max_offset().y.as_f32().max(0.0))
    }

    pub fn scroll_vertical_by<T: 'static>(&self, delta: Pixels, cx: &mut Context<T>) -> bool {
        let current = self.vertical_offset().as_f32();
        let target = (current + delta.as_f32()).clamp(0.0, self.max_vertical_offset().as_f32());
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn ensure_vertical_bounds_visible<T: 'static>(&self, top: Pixels, bottom: Pixels, cx: &mut Context<T>) -> bool {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let current = self.vertical_offset().as_f32();
        let mut target = current;

        let visible_top = current;
        let visible_bottom = current + viewport_height;

        let top = top.as_f32();
        let bottom = bottom.as_f32();

        if top < visible_top {
            target -= visible_top - top;
        } else if bottom > visible_bottom {
            target += bottom - visible_bottom;
        }

        target = target.clamp(0.0, max_scroll);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn sync_scrollbar<T: 'static>(&self, cx: &mut Context<T>) {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let value = (-self.scroll_handle.offset().y.as_f32()).clamp(0.0, max_scroll);
        let content_height = viewport_height + max_scroll;
        let thumb_fraction = if content_height > 0.0 {
            (viewport_height / content_height).clamp(0.05, 1.0)
        } else {
            1.0
        };

        self.scrollbar.update(cx, |scrollbar, cx| {
            scrollbar.set_length(viewport_height.max(1.0), cx);
            scrollbar.set_range(ControlRange::new(0.0, max_scroll.max(1.0)), cx);
            scrollbar.set_step(24.0, cx);
            scrollbar.set_page_step((viewport_height * 0.85).max(1.0), cx);
            scrollbar.set_thumb_fraction(thumb_fraction, cx);
            scrollbar.set_value(value, cx);
            scrollbar.set_enabled(max_scroll > 0.5, cx);
        });
    }

    pub fn render(&self, content: AnyElement) -> Stateful<gpui::Div> {
        let scrollable = self.scroll_handle.max_offset().y.as_f32() > 0.5;
        let viewport_right = if scrollable { self.scrollbar_width } else { px(0.0) };
        let scroll_handle = self.scroll_handle.clone();
        let last_max_scroll = self.last_max_scroll.clone();

        let viewport = div()
            .on_children_prepainted(move |_: Vec<Bounds<Pixels>>, window: &mut Window, cx: &mut App| {
                let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                if (last_max_scroll.get() - max_scroll).abs() > 0.5 {
                    last_max_scroll.set(max_scroll);
                    cx.notify(window.current_view());
                }
            })
            .id(format!("{}-viewport", self.id))
            .absolute()
            .top(px(0.0))
            .bottom(px(0.0))
            .left(px(0.0))
            .right(viewport_right)
            .overflow_y_scroll()
            .scrollbar_width(px(0.0))
            .track_scroll(&self.scroll_handle)
            .child(content);

        div().id(self.id.clone()).relative().size_full().child(viewport).when(scrollable, |root| {
            root.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .right(px(0.0))
                    .bottom(px(0.0))
                    .w(self.scrollbar_width)
                    .flex()
                    .justify_center()
                    .child(self.scrollbar.clone()),
            )
        })
    }
}
