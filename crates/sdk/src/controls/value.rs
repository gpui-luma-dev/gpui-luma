use std::ops::{Range, RangeInclusive};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRange {
    pub start: f32,
    pub end: f32,
}

impl Default for ControlRange {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 100.0,
        }
    }
}

impl ControlRange {
    pub fn new(start: f32, end: f32) -> Self {
        let start = finite_or(start, Self::default().start);
        let end = finite_or(end, Self::default().end);

        if start < end {
            Self { start, end }
        } else if end < start {
            Self {
                start: end,
                end: start,
            }
        } else {
            Self {
                start,
                end: start + 1.0,
            }
        }
    }

    pub fn span(self) -> f32 {
        self.end - self.start
    }

    pub fn clamp(self, value: f32) -> f32 {
        finite_or(value, self.start).clamp(self.start, self.end)
    }

    pub fn percentage(self, value: f32) -> f32 {
        ((self.clamp(value) - self.start) / self.span()).clamp(0.0, 1.0)
    }

    pub fn value_at(self, percentage: f32) -> f32 {
        self.start + self.span() * percentage.clamp(0.0, 1.0)
    }

    pub fn snap(self, value: f32, step: f32) -> f32 {
        let step = normalized_step(step);
        let value = self.clamp(value);
        let stepped = self.start + ((value - self.start) / step).round() * step;

        self.clamp(stepped)
    }
}

impl From<Range<f32>> for ControlRange {
    fn from(range: Range<f32>) -> Self {
        Self::new(range.start, range.end)
    }
}

impl From<RangeInclusive<f32>> for ControlRange {
    fn from(range: RangeInclusive<f32>) -> Self {
        let (start, end) = range.into_inner();
        Self::new(start, end)
    }
}

macro_rules! impl_integer_range {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<Range<$ty>> for ControlRange {
                fn from(range: Range<$ty>) -> Self {
                    Self::new(range.start as f32, range.end as f32)
                }
            }

            impl From<RangeInclusive<$ty>> for ControlRange {
                fn from(range: RangeInclusive<$ty>) -> Self {
                    let (start, end) = range.into_inner();
                    Self::new(start as f32, end as f32)
                }
            }
        )*
    };
}

impl_integer_range!(i8, i16, i32, u8, u16, u32, usize);

pub(crate) fn normalized_step(step: f32) -> f32 {
    if step.is_finite() && step > 0.0 {
        step
    } else {
        1.0
    }
}

pub(crate) fn value_from_input(value: impl Into<f64>) -> f32 {
    finite_or(value.into() as f32, 0.0)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::{ControlRange, normalized_step};

    #[test]
    fn integer_ranges_convert_to_control_ranges() {
        assert_eq!(ControlRange::from(1..100), ControlRange::new(1.0, 100.0));
        assert_eq!(ControlRange::from(1..=100), ControlRange::new(1.0, 100.0));
    }

    #[test]
    fn snapping_is_relative_to_range_start() {
        let range = ControlRange::new(1.0, 100.0);

        assert_eq!(range.snap(5.9, 10.0), 1.0);
        assert_eq!(range.snap(16.0, 10.0), 21.0);
    }

    #[test]
    fn invalid_steps_fall_back_to_one() {
        assert_eq!(normalized_step(0.0), 1.0);
        assert_eq!(normalized_step(f32::NAN), 1.0);
    }
}
