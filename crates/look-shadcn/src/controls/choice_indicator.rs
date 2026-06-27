//! shadcn checkbox / radio: indicator and label colors do not follow hover layers.

use gpui_luma::theme::{InteractionLayer, InteractionState};

pub(crate) fn choice_indicator_color_layer(state: InteractionState) -> InteractionLayer {
    if state.disabled {
        InteractionLayer::Disabled
    } else {
        InteractionLayer::Default
    }
}
