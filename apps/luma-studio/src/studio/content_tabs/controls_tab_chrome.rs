use std::sync::{Arc, Mutex};

use gpui::{Bounds, Pixels};

#[derive(Clone, Default)]
pub(crate) struct ControlsTabChrome {
    inner: Arc<ControlsTabChromeInner>,
}

#[derive(Default)]
struct ControlsTabChromeInner {
    tab_bounds: Mutex<Option<Bounds<Pixels>>>,
    picker_open: Mutex<bool>,
    catalog_picker_width: Mutex<Option<Pixels>>,
}

impl ControlsTabChrome {
    pub(crate) fn set_tab_bounds(&self, bounds: Bounds<Pixels>) {
        *self.inner.tab_bounds.lock().expect("controls tab bounds") = Some(bounds);
    }

    pub(crate) fn tab_bounds(&self) -> Option<Bounds<Pixels>> {
        self.inner.tab_bounds.lock().expect("controls tab bounds").clone()
    }

    pub(crate) fn set_picker_open(&self, open: bool) {
        *self.inner.picker_open.lock().expect("controls tab picker open") = open;
    }

    pub(crate) fn is_picker_open(&self) -> bool {
        *self.inner.picker_open.lock().expect("controls tab picker open")
    }

    pub(crate) fn set_catalog_picker_width(&self, width: Pixels) -> bool {
        let mut stored = self.inner.catalog_picker_width.lock().expect("controls catalog picker width");
        if *stored == Some(width) {
            return false;
        }
        *stored = Some(width);
        true
    }

    pub(crate) fn catalog_picker_width(&self) -> Option<Pixels> {
        *self.inner.catalog_picker_width.lock().expect("controls catalog picker width")
    }
}
