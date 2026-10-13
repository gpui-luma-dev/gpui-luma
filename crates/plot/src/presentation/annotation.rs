use gpui::{Background, Pixels, SharedString, px, rgb};
use super::{Stroke, TextPresentation};
#[derive(Clone, Debug)]
pub struct AnnotationPresentation {
    pub stroke: Option<Stroke>,
    pub fill: Option<Background>,
    pub marker_radius: Pixels,
    pub text: Option<SharedString>,
    pub text_style: TextPresentation,
}
impl Default for AnnotationPresentation {
    fn default() -> Self {
        let color = rgb(0xf59e0b).into();
        Self {
            stroke: Some(Stroke { color, width: px(1.0) }),
            fill: Some(gpui::rgb(0xf59e0b).opacity(0.15).into()),
            marker_radius: px(4.0),
            text: None,
            text_style: TextPresentation::new(color),
        }
    }
}
