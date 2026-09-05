//! Shared label typography for radix control themes.

use gpui::{FontWeight, SharedString};
use luma::theme::{ControlSize, LumaTextStyle};

use crate::look::RadixLook;

pub(crate) fn label_typography(size: ControlSize) -> LumaTextStyle {
    match size {
        ControlSize::Sm => LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
        ControlSize::Md => LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
        ControlSize::Lg => LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::NORMAL },
    }
}

pub(crate) fn font_family(_look: &RadixLook) -> SharedString {
    SharedString::from("System UI")
}
