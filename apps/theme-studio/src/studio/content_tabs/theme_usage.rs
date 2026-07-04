use gpui::{Div, div, prelude::*};

pub fn viewport() -> Div {
    div().flex_1().min_h_0().size_full().overflow_hidden()
}
