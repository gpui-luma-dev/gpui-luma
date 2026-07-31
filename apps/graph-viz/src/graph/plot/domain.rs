use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq)]
pub struct PlotDomain2D {
    pub x_range: RangeInclusive<f32>,
    pub y_range: RangeInclusive<f32>,
}

impl PlotDomain2D {
    pub fn new(x_range: RangeInclusive<f32>, y_range: RangeInclusive<f32>) -> Self {
        Self { x_range, y_range }
    }

    /// Projects `(x_data, y_data)` into normalized UV `(0.0..=1.0, 0.0..=1.0)`.
    /// Low data values map to low UV Y (bottom of chart domain).
    #[cfg(test)]
    pub fn to_uv(&self, x: f32, y: f32) -> gpui::Point<f32> {
        let x_min = *self.x_range.start();
        let x_max = *self.x_range.end();
        let y_min = *self.y_range.start();
        let y_max = *self.y_range.end();
        let x_span = (x_max - x_min).max(f32::EPSILON);
        let y_span = (y_max - y_min).max(f32::EPSILON);

        gpui::point(((x - x_min) / x_span).clamp(0.0, 1.0), ((y - y_min) / y_span).clamp(0.0, 1.0))
    }

    pub fn x_to_fraction(&self, x: f32) -> f32 {
        let x_min = *self.x_range.start();
        let x_max = *self.x_range.end();
        let span = (x_max - x_min).max(f32::EPSILON);
        ((x - x_min) / span).clamp(0.0, 1.0)
    }

    pub fn y_to_fraction(&self, y: f32) -> f32 {
        let y_min = *self.y_range.start();
        let y_max = *self.y_range.end();
        let span = (y_max - y_min).max(f32::EPSILON);
        ((y - y_min) / span).clamp(0.0, 1.0)
    }

    pub fn x_from_fraction(&self, fraction: f32) -> f32 {
        let x_min = *self.x_range.start();
        let x_max = *self.x_range.end();
        x_min + fraction.clamp(0.0, 1.0) * (x_max - x_min)
    }

    #[cfg(test)]
    pub fn y_from_fraction(&self, fraction: f32) -> f32 {
        let y_min = *self.y_range.start();
        let y_max = *self.y_range.end();
        y_min + fraction.clamp(0.0, 1.0) * (y_max - y_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_uv_maps_corners() {
        let domain = PlotDomain2D::new(0.0..=10.0, 0.0..=100.0);
        assert_eq!(domain.to_uv(0.0, 0.0), gpui::point(0.0, 0.0));
        assert_eq!(domain.to_uv(10.0, 100.0), gpui::point(1.0, 1.0));
        assert_eq!(domain.to_uv(5.0, 50.0), gpui::point(0.5, 0.5));
    }

    #[test]
    fn to_uv_clamps_out_of_range_values() {
        let domain = PlotDomain2D::new(0.0..=10.0, 0.0..=100.0);
        assert_eq!(domain.to_uv(-5.0, 200.0), gpui::point(0.0, 1.0));
    }

    #[test]
    fn x_from_fraction_interpolates_time() {
        let domain = PlotDomain2D::new(0.0..=100.0, 0.0..=1.0);
        assert_eq!(domain.x_from_fraction(0.5), 50.0);
    }

    #[test]
    fn y_from_fraction_interpolates_values() {
        let domain = PlotDomain2D::new(0.0..=1.0, 10.0..=30.0);
        assert_eq!(domain.y_from_fraction(0.0), 10.0);
        assert_eq!(domain.y_from_fraction(1.0), 30.0);
        assert_eq!(domain.y_from_fraction(0.5), 20.0);
    }
}
