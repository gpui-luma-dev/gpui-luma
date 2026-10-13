use gpui_luma_look_radix::Look;
use gpui_luma_plot::RadarPresentation;
use crate::theme;

pub(crate) fn style(look: &Look) -> RadarPresentation {
    let chrome = theme::chrome(look);
    let mut style = RadarPresentation::default();
    style.label_text = theme::pie::style(look).label_text;
    style.label_text.color = chrome.muted_text;
    if let Some(empty) = &mut style.empty_state {
        empty.text_color = chrome.muted_text;
    }
    if let Some(grid) = &mut style.grid {
        grid.color = chrome.border;
    }
    style
}
