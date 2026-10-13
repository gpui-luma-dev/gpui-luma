/// Combines data-space geometry and visual appearance without backend types.
#[derive(Clone, Debug)]
pub struct ChartAnnotation {
    pub geometry: crate::Annotation,
    pub presentation: crate::AnnotationPresentation,
}
impl ChartAnnotation {
    pub fn new(geometry: crate::Annotation, presentation: crate::AnnotationPresentation) -> Self {
        Self { geometry, presentation }
    }
}
