use crate::Sample;
/// Data-space annotation geometry. Bands are rectangles between two coordinates.
#[derive(Clone, Debug)]
pub enum Annotation {
    Point(Sample),
    Line { from: Sample, to: Sample },
    Band { from: Sample, to: Sample },
    Text(Sample),
}
