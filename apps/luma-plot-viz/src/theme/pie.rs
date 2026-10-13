use gpui::px;
use gpui_luma_look_radix::Look;
use gpui_luma_plot::{PiePresentation, TextPresentation, LegendPresentation, EmptyStatePresentation};
use crate::theme;

pub(crate) fn style(look: &Look) -> PiePresentation {
    let chrome = theme::chrome(look);
    let typography = theme::typography();
    let mut text = TextPresentation::new(chrome.body_text);
    text.font_family = typography.font.sans.family.into();
    text.font_size = px(12.0);
    text.line_height = px(18.0);
    let mut title = text.clone();
    title.font_size = px(28.0);
    title.line_height = px(34.0);
    let mut caption = text.clone();
    caption.color = chrome.muted_text;
    let mut style = PiePresentation {
        label_text: text,
        center_title: title,
        center_caption: caption,
        legend: LegendPresentation::new(chrome.body_text),
        empty_state: Some(EmptyStatePresentation::new("No positive finite values", chrome.muted_text)),
        ..PiePresentation::default()
    };
    style.legend.visible = false;
    style
}

pub(crate) fn month_selector<M: 'static>(
    look: &Look,
    cx: &mut gpui::Context<M>,
) -> gpui::Entity<gpui_luma::controls::selector::Selector> {
    use gpui_luma::controls::selector::SelectorItem;
    gpui_luma_look_radix::Selector::new("pie-month-selector")
        .look(look)
        .items(
            ["January", "February", "March", "April", "May", "June"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| SelectorItem::new(index.to_string()).label(label)),
        )
        .selected_id("0")
        .spawn(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pie_text_resolves_the_current_theme_mode() {
        let look = Look::built_in();
        look.set_mode(gpui_luma::theme::ThemeMode::Light);
        let light = style(&look);
        look.set_mode(gpui_luma::theme::ThemeMode::Dark);
        let dark = style(&look);
        assert_ne!(light.label_text.color, dark.label_text.color);
        assert_ne!(light.center_title.color, dark.center_title.color);
    }
}
