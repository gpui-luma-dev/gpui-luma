use std::{fmt, sync::Arc};

use super::Sample;

/// A stable identifier used by specifications and presentation overrides.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SeriesId(Arc<str>);

impl SeriesId {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SeriesId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<&str> for SeriesId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SeriesId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Identified immutable samples and a semantic display label.
///
/// The label describes data for a legend; colors and other appearance never live here.
#[derive(Clone, Debug)]
pub struct Series {
    id: SeriesId,
    label: Arc<str>,
    samples: Arc<[Sample]>,
    sorted_x: bool,
    finite_count: usize,
}

impl Series {
    pub fn new(id: impl Into<SeriesId>, label: impl Into<Arc<str>>, samples: impl Into<Arc<[Sample]>>) -> Self {
        let samples: Arc<[Sample]> = samples.into();
        let sorted_x =
            samples.iter().all(|sample| sample.x.is_finite()) && samples.windows(2).all(|pair| pair[0].x <= pair[1].x);
        let finite_count = samples.iter().filter(|sample| sample.is_finite()).count();
        Self { id: id.into(), label: label.into(), samples, sorted_x, finite_count }
    }

    pub fn id(&self) -> &SeriesId {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn sorted_x(&self) -> bool {
        self.sorted_x
    }
    pub(crate) fn finite_count(&self) -> usize {
        self.finite_count
    }
    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn immutable_series_cache_order_and_finite_counts() {
        let ordered = Series::new("a", "A", [Sample::new(1.0, 2.0), Sample::new(1.0, f64::NAN), Sample::new(3.0, 4.0)]);
        assert!(ordered.sorted_x());
        assert_eq!(ordered.finite_count(), 2);
        assert!(ordered.clone().sorted_x());
        assert!(!Series::new("a", "A", [Sample::new(2.0, 1.0), Sample::new(1.0, 2.0)]).sorted_x());
        assert!(!Series::new("a", "A", [Sample::new(f64::NAN, 1.0)]).sorted_x());
    }
}
