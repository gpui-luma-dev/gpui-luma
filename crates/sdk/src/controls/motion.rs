use std::time::Duration;

use gpui::{Animation, AnimationExt, AnyElement, ElementId, IntoElement, ease_in_out};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionEasing {
    Linear,
    EaseInOut,
    EaseOutQuint,
}

#[derive(Clone, Copy, Debug)]
pub struct MotionSpec {
    pub duration_ms: u64,
    pub easing: MotionEasing,
}

impl MotionSpec {
    pub fn animation(self) -> Animation {
        let base = Animation::new(Duration::from_millis(self.duration_ms));

        match self.easing {
            MotionEasing::Linear => base,
            MotionEasing::EaseInOut => base.with_easing(ease_in_out),
            MotionEasing::EaseOutQuint => base.with_easing(gpui::ease_out_quint()),
        }
    }
}

pub fn animate_f32<E, F>(
    element: E,
    id: impl Into<ElementId>,
    motion: Option<MotionSpec>,
    from: f32,
    to: f32,
    apply: F,
) -> AnyElement
where
    E: IntoElement + 'static,
    F: Fn(E, f32) -> E + 'static,
{
    if let Some(spec) = motion {
        element
            .with_animation(id, spec.animation(), move |this, delta| {
                let value = from + (to - from) * delta;
                apply(this, value)
            })
            .into_any_element()
    } else {
        apply(element, to).into_any_element()
    }
}

pub fn animate_value(
    id: impl Into<ElementId>,
    motion: Option<MotionSpec>,
    from: f32,
    to: f32,
    build: impl Fn(f32) -> AnyElement + 'static,
) -> AnyElement {
    let element = build(to);

    if let Some(spec) = motion {
        element
            .with_animation(id, spec.animation(), move |_, delta| {
                let value = from + (to - from) * delta;
                build(value)
            })
            .into_any_element()
    } else {
        element
    }
}
