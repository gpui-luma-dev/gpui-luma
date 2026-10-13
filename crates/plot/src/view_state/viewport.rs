use std::ops::RangeInclusive;
/// A bounded, finite numeric viewport. Hosts retain it and connect SDK gestures or controls.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewport {
    x: RangeInclusive<f64>,
    y: RangeInclusive<f64>,
    initial: (RangeInclusive<f64>, RangeInclusive<f64>),
    limits: (RangeInclusive<f64>, RangeInclusive<f64>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportChange {
    pub x: RangeInclusive<f64>,
    pub y: RangeInclusive<f64>,
    pub action: ViewportAction,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportAction {
    Pan,
    Zoom,
    Reset,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidViewport;
impl std::fmt::Display for InvalidViewport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("viewport domains must have finite, positive spans and fit their limits")
    }
}
impl std::error::Error for InvalidViewport {}
fn valid(r: &RangeInclusive<f64>) -> bool {
    r.start().is_finite() && r.end().is_finite() && r.start() < r.end() && (r.end() - r.start()).is_finite()
}
fn fit(start: f64, span: f64, limit: &RangeInclusive<f64>) -> Option<RangeInclusive<f64>> {
    if !start.is_finite() || !span.is_finite() || span <= 0.0 {
        return None;
    }
    let span = span.min(limit.end() - limit.start());
    let start = start.clamp(*limit.start(), limit.end() - span);
    let end = start + span;
    (end > start && end.is_finite()).then_some(start..=end)
}
impl Viewport {
    pub fn new(x: RangeInclusive<f64>, y: RangeInclusive<f64>) -> Result<Self, InvalidViewport> {
        if !valid(&x) || !valid(&y) {
            return Err(InvalidViewport);
        }
        Ok(Self { initial: (x.clone(), y.clone()), limits: (x.clone(), y.clone()), x, y })
    }
    /// Limits must include the initial domains. Invalid requests leave the viewport unchanged.
    pub fn limits(mut self, x: RangeInclusive<f64>, y: RangeInclusive<f64>) -> Result<Self, InvalidViewport> {
        if !valid(&x)
            || !valid(&y)
            || x.start() > self.initial.0.start()
            || x.end() < self.initial.0.end()
            || y.start() > self.initial.1.start()
            || y.end() < self.initial.1.end()
        {
            return Err(InvalidViewport);
        }
        self.limits = (x, y);
        Ok(self)
    }
    pub fn x(&self) -> RangeInclusive<f64> {
        self.x.clone()
    }
    pub fn y(&self) -> RangeInclusive<f64> {
        self.y.clone()
    }
    fn commit(
        &mut self,
        x: RangeInclusive<f64>,
        y: RangeInclusive<f64>,
        action: ViewportAction,
    ) -> Option<ViewportChange> {
        if self.x == x && self.y == y {
            return None;
        }
        self.x = x.clone();
        self.y = y.clone();
        Some(ViewportChange { x, y, action })
    }
    /// Pan in data units; clamps to configured limits. Invalid values do nothing.
    pub fn pan(&mut self, dx: f64, dy: f64) -> Option<ViewportChange> {
        let x = fit(self.x.start() + dx, self.x.end() - self.x.start(), &self.limits.0)?;
        let y = fit(self.y.start() + dy, self.y.end() - self.y.start(), &self.limits.1)?;
        self.commit(x, y, ViewportAction::Pan)
    }
    /// Factors greater than one zoom in. Anchors are fractions from the low end (0..=1).
    /// Convert pointer coordinates using the plot's bounds and orientation in the host.
    pub fn zoom_at(&mut self, factor: f64, anchor_x: f64, anchor_y: f64) -> Option<ViewportChange> {
        if !factor.is_finite() || factor <= 0.0 || !anchor_x.is_finite() || !anchor_y.is_finite() {
            return None;
        }
        let zoom = |r: &RangeInclusive<f64>, limit: &RangeInclusive<f64>, anchor: f64| {
            let anchor = anchor.clamp(0.0, 1.0);
            let old = r.end() - r.start();
            let span = old / factor;
            fit(r.start() + old * anchor - span * anchor, span, limit)
        };
        let x = zoom(&self.x, &self.limits.0, anchor_x)?;
        let y = zoom(&self.y, &self.limits.1, anchor_y)?;
        self.commit(x, y, ViewportAction::Zoom)
    }
    pub fn reset(&mut self) -> Option<ViewportChange> {
        self.commit(self.initial.0.clone(), self.initial.1.clone(), ViewportAction::Reset)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_keeps_anchor_pan_clamps_and_reset_restores() {
        let mut v = Viewport::new(0.0..=100.0, 0.0..=200.0).unwrap();
        assert_eq!(v.zoom_at(2.0, 0.25, 0.5).unwrap().action, ViewportAction::Zoom);
        assert_eq!(v.x(), 12.5..=62.5);
        assert_eq!(v.y(), 50.0..=150.0);
        v.pan(1000.0, -1000.0);
        assert_eq!(v.x(), 50.0..=100.0);
        assert_eq!(v.y(), 0.0..=100.0);
        assert!(v.zoom_at(f64::NAN, 0.5, 0.5).is_none());
        v.reset();
        assert_eq!(v.x(), 0.0..=100.0);
    }
}
