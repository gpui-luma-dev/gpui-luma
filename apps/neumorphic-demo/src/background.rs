use gpui::{Background, linear_color_stop, linear_gradient, rgb};

/// Soft deck backdrop: lighter top-left, darker bottom-right.
/// Uses explicit RGB stops so the gradient reads clearly on GPUI's RGB pipeline.
pub fn deck_background() -> Background {
    linear_gradient(135., linear_color_stop(rgb(0xF7F9FC), 0.), linear_color_stop(rgb(0xA8B4C4), 1.))
}
