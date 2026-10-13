use std::{error::Error, fmt};

use super::{Series, SeriesId};

/// An immutable collection of uniquely identified series.
#[derive(Clone, Debug, Default)]
pub struct Dataset {
    series: Vec<Series>,
}

impl Dataset {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a series, rejecting duplicate stable identities.
    pub fn with_series(mut self, series: Series) -> Result<Self, DatasetError> {
        if self.series.iter().any(|existing| existing.id() == series.id()) {
            return Err(DatasetError::DuplicateSeriesId(series.id().clone()));
        }
        self.series.push(series);
        Ok(self)
    }

    pub fn series(&self) -> &[Series] {
        &self.series
    }

    pub fn get(&self, id: &SeriesId) -> Option<&Series> {
        self.series.iter().find(|series| series.id() == id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatasetError {
    DuplicateSeriesId(SeriesId),
}

impl fmt::Display for DatasetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSeriesId(id) => write!(formatter, "dataset already contains series '{id}'"),
        }
    }
}

impl Error for DatasetError {}
