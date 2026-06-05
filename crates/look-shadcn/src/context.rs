use std::cell::RefCell;
use std::sync::Arc;

use crate::look::ShadcnLook;

thread_local! {
    static ACTIVE_LOOK: RefCell<Option<Arc<ShadcnLook>>> = const { RefCell::new(None) };
}

/// Binds the active look to the thread's scope during layout/render block execution.
pub fn with_look<R>(look: &Arc<ShadcnLook>, f: impl FnOnce() -> R) -> R {
    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = Some(look.clone());
    });
    let result = f();
    ACTIVE_LOOK.with(|cell| {
        *cell.borrow_mut() = None;
    });
    result
}

pub(crate) fn with_active_look<R>(f: impl FnOnce(Option<Arc<ShadcnLook>>) -> R) -> R {
    ACTIVE_LOOK.with(|cell| f(cell.borrow().clone()))
}
