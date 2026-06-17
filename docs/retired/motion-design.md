# Motion Design

## Goal

Motion should be an SDK-level rendering concept, not a property bolted onto individual controls.

```text
state change
  -> render-derived target values
  -> motion transaction
  -> interpolated visual values
  -> GPUI elements
```

Controls should not track animation start/end state.

```text
avoid:
  control.previous_percentage
  appearance.thumb_motion
  animate_f32(...)
  per-control animation flags

prefer:
  motion.value(key, target, role)
```

## Core idea

Borrow the useful part of SwiftUI:

```text
motion is not owned by specific controls
motion belongs to the state -> view pipeline
any visual value can participate if it has identity
```

Motion is an ambient transaction layer over render-derived visual values.

```text
Control state:
  Switch { is_on: bool }

Render target:
  thumb_progress = if is_on { 1.0 } else { 0.0 }

Motion system:
  rendered_progress = cx.motion_value("thumb-progress", thumb_progress, MotionRole::Responsive)

View geometry:
  thumb_left = off_left + travel * rendered_progress
```

## Non-goals

```text
not now:
  theme-driven motion tokens
  per-control motion fields
  spring tuning UI
  animation DSL
  full physics engine

must support later:
  theme overrides
  reduced motion
  spring/timing variants
  interruption policy
```

## SwiftUI reference model

```text
SwiftUI implicit:
  view.animation(animation, value: some_state)
  // whenever some_state changes, animate affected render output

SDK equivalent:
  element.motion(MotionPolicy::responsive(), when: model.is_on)
  // or a keyed MotionValue observes the target change directly
```

```text
SwiftUI explicit:
  withAnimation(.spring()) {
      state.is_on.toggle()
  }

SDK equivalent:
  cx.with_motion(MotionPolicy::spring(SpringMotion::snappy()), |cx| {
      state.is_on = !state.is_on;
      cx.notify();
  });
```

Important distinction:

```text
not: Switch has animation
yes: Switch derives thumb-progress; current transaction decides whether it moves
```

## Built-in motion categories

Some controls should feel animated out of the box, but this should still be pipeline-driven.

```text
Built-in control motion:
  Toggle/Switch       -> thumb progress uses MotionRole::Responsive
  Slider              -> drag is immediate; release/settle may use MotionRole::Responsive
  Progress determined -> percentage uses MotionRole::ValueChange
  Progress spinner    -> continuous timeline motion, not state-transition motion
  Scroll containers   -> inertia/rubber-band/snap physics owned by scroll behavior
```

```text
High-motion containers:
  Navigation stack    -> route transition roles
  Tab/page view       -> drag physics + page snapping
  List                -> insert/remove/reorder transitions
  Matched geometry    -> shared MotionKey across separate render locations
```

These are not exceptions to the pipeline. They are higher-level producers of motion values.

```text
Switch:
  state target -> MotionValue<f32> -> thumb left

List insertion:
  identity appears -> transition value -> row offset/opacity

Matched geometry:
  same MotionKey appears in new layout slot -> rect interpolation
```

## Animatable modifiers

The SDK should eventually expose motion-ready visual modifiers.

```text
.offset(x, y)      -> MotionValue<Point/Pixels>
.scale(factor)     -> MotionValue<f32>
.rotate(angle)     -> MotionValue<f32>
.opacity(alpha)    -> MotionValue<f32>
.blur(radius)      -> MotionValue<f32>
.color(hsla)       -> MotionValue<Hsla>
.size(width,height)-> MotionValue<Size<Pixels>>
```

Possible fluent shape:

```text
div()
    .motion(MotionPolicy::responsive())
    .motion_offset(key!(id, "offset"), target_offset)
    .motion_opacity(key!(id, "opacity"), target_opacity)
```

Lower-level control shape:

```text
let alpha = cx.motion_value(key!(id, "opacity"), target_alpha, MotionRole::EnterExit);
let offset = cx.motion_value(key!(id, "offset"), target_offset, MotionRole::Responsive);
```

## Motion triggers

Two trigger styles are needed.

```text
explicit trigger:
  cx.with_motion(policy, |cx| { mutate_state(); cx.notify(); })

implicit trigger:
  motion value notices target changed for same MotionKey
```

The implicit form is what keeps controls simple.

```text
let rendered = cx.motion_value(key, target, role);

// if previous target for key != target:
//   start transition from current rendered value to target
```

## Concepts

## `MotionTransaction`

A state update may carry a motion policy.

```text
MotionTransaction {
    policy: MotionPolicy,
    reduced_motion: ReducedMotionPolicy,
    source: MotionSource,
}
```

Example usage:

```text
cx.with_motion(MotionPolicy::responsive(), |cx| {
    switch.is_on = !switch.is_on;
    cx.notify();
});
```

No explicit motion:

```text
switch.is_on = !switch.is_on;
cx.notify();

// uses ambient/default transaction policy
```

Disable motion:

```text
cx.without_motion(|cx| {
    model.value = next;
    cx.notify();
});
```

## `MotionScope`

A view subtree can provide default motion.

```text
div()
    .motion(MotionPolicy::responsive())
    .child(switch)
    .child(progress)
```

Override locally:

```text
div()
    .motion(MotionPolicy::instant())
    .child(disabled_preview)
```

Inheritance:

```text
root MotionPolicy::standard
  sidebar inherits standard
  form overrides responsive
    switch inherits responsive
    textfield caret uses instant
```

## `MotionValue<T>`

A stable keyed visual value that knows its previous target, current target, and active interpolation.

```text
MotionValue<T> {
    key: MotionKey,
    target: T,
    rendered: T,
    transition: Option<ActiveTransition<T>>,
}
```

Control render code asks for a rendered value:

```text
let progress = cx.motion_value(
    MotionKey::new(model.id, "thumb-progress"),
    target_progress,
    MotionRole::Responsive,
);
```

The motion system owns:

```text
previous target lookup
initial render behavior
interruption behavior
reduced-motion fallback
active animation clock
```

Controls do not own:

```text
previous_on
previous_percentage
start_left
end_left
Animation::new(...)
```

## `MotionKey`

Motion identity must be stable.

```text
MotionKey = element_id + visual_property
```

Examples:

```text
"settings.notifications.thumb-progress"
"upload.progress.percentage"
"nav.sidebar.width"
"menu.opacity"
```

Bad keys:

```text
// changes every target; prevents clean interruption
format!("progress-motion-{percentage}")

// changes every state; restarts instead of retargeting
format!("switch-thumb-{is_on}")
```

Good keys:

```text
format!("{}:thumb-progress", model.id)
format!("{}:progress", model.id)
```

## `MotionRole`

Controls describe why a value moves, not how many milliseconds it takes.

```text
enum MotionRole {
    Instant,
    Responsive,
    ValueChange,
    Continuous,
    ScrollPhysics,
    ExpandCollapse,
    EnterExit,
    Reorder,
    MatchedGeometry,
    Emphasized,
}
```

Default mapping:

```text
Instant         -> no interpolation
Responsive     -> short spring; toggle, hover, press, slider settle
ValueChange    -> smooth spring/timing; progress fill, numeric value changes
Continuous     -> timeline-driven; spinner, pulse, shimmer
ScrollPhysics  -> velocity/inertia/rubber-band/snap
ExpandCollapse -> structured transition; menus, sidebars
EnterExit      -> opacity/scale/offset pair
Reorder        -> list movement when identity order changes
MatchedGeometry -> rect/position interpolation across layout locations
Emphasized     -> slower/larger transition
```

Control usage:

```text
switch thumb position        -> Responsive
slider thumb while dragging  -> Instant / direct manipulation
slider thumb on release      -> Responsive
progress percentage          -> ValueChange
progress spinner             -> Continuous
scroll offset after release  -> ScrollPhysics
sidebar width                -> ExpandCollapse
popup opacity                -> EnterExit
list row insertion/removal   -> EnterExit
list row reorder             -> Reorder
matched rect                 -> MatchedGeometry
focus ring opacity           -> Responsive or Instant
```

## `MotionPolicy`

Policy is the actual interpolation strategy.

```text
enum MotionPolicy {
    Instant,
    Timing(TimingMotion),
    Spring(SpringMotion),
}

TimingMotion {
    duration_ms: u64,
    easing: Easing,
}

SpringMotion {
    stiffness: f32,
    damping: f32,
    mass: f32,
}
```

Semantic defaults:

```text
MotionDefaults {
    responsive: Spring { stiffness: 300, damping: 24, mass: 1 },
    value_change: Spring { stiffness: 220, damping: 28, mass: 1 },
    expand_collapse: Spring { stiffness: 260, damping: 30, mass: 1 },
    enter_exit: Timing { duration_ms: 140, easing: EaseOutQuint },
    emphasized: Spring { stiffness: 180, damping: 22, mass: 1 },
}
```

Default preference:

```text
prefer springs for interactive state changes
use fixed timing for opacity-only or deterministic enter/exit
use physics policies for scroll/drag release
```
```

## Pipeline sketch

```text
render control
  target = derive_target_from_state()
  rendered = cx.motion_value(key, target, role)
  element = build_element(rendered)
```

Internals:

```text
fn motion_value<T>(key, target, role) -> T
where
    T: Animatable,
{
    let transaction = current_motion_transaction();
    let policy = transaction.policy_for(role);

    let slot = motion_registry.entry(key);

    if slot.first_render {
        slot.target = target;
        slot.rendered = target;
        return target;
    }

    if slot.target != target {
        slot.transition = ActiveTransition {
            from: slot.rendered,
            to: target,
            policy,
            started_at: now(),
        };
        slot.target = target;
    }

    slot.rendered = slot.transition.sample(now());
    slot.rendered
}
```

## `Animatable`

Only a few primitive types need support at first.

```text
trait Animatable: Copy {
    fn interpolate(from: Self, to: Self, t: f32) -> Self;
}
```

Initial implementations:

```text
f32
Pixels
Point<Pixels>
Size<Pixels>
Hsla
```

Example:

```text
impl Animatable for f32 {
    fn interpolate(from, to, t) -> f32 {
        from + (to - from) * t
    }
}
```

## Switch example

Control state:

```text
SwitchModel {
    id: SharedString,
    is_on: bool,
    enabled: bool,
}
```

Render:

```text
fn render_switch(model, cx) -> Element {
    let off_left = 2.0;
    let on_left = 42.0 - 11.0 - 2.0;
    let travel = on_left - off_left;

    let target_progress = if model.is_on { 1.0 } else { 0.0 };

    let progress = cx.motion_value(
        MotionKey::new(&model.id, "thumb-progress"),
        target_progress,
        MotionRole::Responsive,
    );

    let thumb_left = off_left + travel * progress;

    div()
        .relative()
        .child(
            div()
                .id(format!("{}-thumb", model.id))
                .absolute()
                .left(px(thumb_left))
        )
}
```

What is absent:

```text
no previous_is_on
no start_left/end_left
no thumb_motion field
no direct Animation::new
no animate_f32 helper in template code
```

## Slider example

Drag should be direct; release/settle can be motion-driven.

```text
fn render_slider(model, cx) -> Element {
    let target = model.range.percentage(model.value);

    let role = if model.is_dragging {
        MotionRole::Instant
    } else {
        MotionRole::Responsive
    };

    let progress = cx.motion_value(
        MotionKey::new(&model.id, "thumb-progress"),
        target,
        role,
    );

    let thumb_x = track_left + track_width * progress;
}
```

## Progress example

Control state:

```text
ProgressModel {
    id: SharedString,
    value: f32,
    range: ControlRange,
}
```

Render:

```text
fn render_progress(model, cx) -> Element {
    let target = model.range.percentage(model.value);

    let progress = cx.motion_value(
        MotionKey::new(&model.id, "percentage"),
        target,
        MotionRole::ValueChange,
    );

    canvas(move |bounds, window| {
        paint_track(bounds, window);
        paint_progress_arc(bounds, progress, window);
    })
}
```

What is absent:

```text
no previous_percentage
no progress_motion field
no animation wrapper in ProgressTemplate
```

## Continuous motion example

Indeterminate progress is not a state-to-target transition. It is timeline-driven.

```text
fn render_spinner(model, cx) -> Element {
    let angle = cx.motion_timeline(
        MotionKey::new(&model.id, "spinner-rotation"),
        MotionRole::Continuous,
        |elapsed| elapsed.seconds() * 360.0,
    );

    div().rotate(deg(angle))
}
```

Keep these separate:

```text
MotionValue<T>      -> target changes over time
MotionTimeline<T>   -> continuous elapsed-time value
ScrollPhysics       -> velocity and bounds simulation
```

## List / identity transition example

Rows should move because identity/layout changed, not because each row owns animation state.

```text
for row in rows {
    let y = layout.position_for(row.id);
    let rendered_y = cx.motion_value(
        MotionKey::new(row.id, "row-y"),
        y,
        MotionRole::Reorder,
    );

    render_row(row).top(rendered_y)
}
```

Insert/remove:

```text
row appears:
  opacity: 0 -> 1
  offset_y: 8 -> 0

row removed:
  keep ghost slot until exit transition completes
  opacity: 1 -> 0
  offset_y: 0 -> -8
```

## Matched geometry sketch

Shared visual identity across different layout locations.

```text
small_card.icon
    .matched_geometry(MotionKey::new(item.id, "hero-icon"))

fullscreen_header.icon
    .matched_geometry(MotionKey::new(item.id, "hero-icon"))
```

Registry sees same key at old/new rect:

```text
from_rect = previous_layout[key]
to_rect = current_layout[key]
rendered_rect = cx.motion_value(key, to_rect, MotionRole::MatchedGeometry)
```

## Where storage lives

Motion values need persistent storage outside controls.

Options:

```text
WindowMotionRegistry
  lifetime: window
  keys: MotionKey
  good for visual continuity within a window

EntityMotionRegistry
  lifetime: entity
  keys: local MotionKey
  good for entity-scoped cleanup

AppMotionRegistry
  lifetime: app
  likely too broad
```

Preferred first pass:

```text
WindowMotionRegistry
```

Reason:

```text
render pass already has Window
animation frame scheduling is window-driven
visual identity is usually element/window scoped
```

Possible API surface:

```text
trait MotionContextExt {
    fn motion_value<T: Animatable>(
        &mut self,
        key: MotionKey,
        target: T,
        role: MotionRole,
    ) -> T;
}
```

Usage from templates:

```text
let progress = window.motion_value(key, target, MotionRole::Responsive);
```

or if GPUI makes `App`/`Context` easier:

```text
let progress = cx.motion_value(key, target, MotionRole::Responsive);
```

## Frame scheduling

When an active transition exists:

```text
motion_registry.sample_all(now)
if any transition active:
    window.request_animation_frame()
    cx.notify()
```

Conceptual loop:

```text
state change -> notify -> render
render asks motion_value -> transition starts
animation frame -> notify -> render
render asks motion_value -> sampled value advances
transition completes -> registry stores final target
```

## Initial render behavior

```text
first value for key:
  rendered = target
  no animation
```

This avoids mount animations by default.

If desired later:

```text
cx.motion_appear(key, from, to, role)
```

## Interruption behavior

When target changes mid-flight:

```text
new transition.from = current rendered value
new transition.to = new target
new transition.policy = current transaction policy
```

For switch:

```text
off -> on begins
user toggles off halfway
new from = current thumb progress, e.g. 0.45
new to = 0.0
```

Controls do not need to know this happened.

## Reduced motion

Global policy:

```text
enum ReducedMotionPolicy {
    Allow,
    PreferReduced,
    ForceInstant,
}
```

Resolution:

```text
if reduced_motion != Allow {
    policy = MotionPolicy::Instant;
}
```

Control code remains unchanged.

## Public API sketch

```text
pub mod motion {
    pub struct MotionKey;
    pub enum MotionRole;
    pub enum MotionPolicy;
    pub struct TimingMotion;
    pub struct SpringMotion;
    pub trait Animatable;
    pub trait MotionContextExt;
}
```

Public use:

```text
use gpui_luma::motion::{MotionRole, MotionPolicy};
```

or under controls if this stays UI-only:

```text
use gpui_luma::controls::motion::{MotionRole, MotionPolicy};
```

Preferred location:

```text
crate::motion
```

Reason:

```text
motion is not a control
motion can apply to shell, themes, panes, layout, menus, anything render-derived
```

## API levels

The SDK should expose multiple levels, from lowest to highest.

```text
Level 1: motion values
  cx.motion_value(key, target, role)

Level 2: motion modifiers
  div().motion_opacity(key, target)
  div().motion_offset(key, target)

Level 3: motion scopes / transactions
  div().motion(policy)
  cx.with_motion(policy, ...)

Level 4: structural motion
  list row identity transitions
  matched geometry
  navigation transitions
```

Avoid starting at Level 4.

```text
first implementation target:
  MotionValue<f32>
  MotionKey
  MotionRole
  MotionPolicy::Spring / Instant
  Switch thumb progress
```

## Adoption path

Step 1: add primitives without using them everywhere.

```text
motion/mod.rs
MotionKey
MotionRole
MotionPolicy
Animatable for f32/Hsla/Pixels
```

Step 2: add registry + extension.

```text
WindowMotionRegistry
MotionContextExt::motion_value(...)
```

Step 3: migrate one control.

```text
Switch thumb progress
```

Step 4: migrate second shape.

```text
Progress percentage
```

Step 5: add scopes/transactions.

```text
.motion(policy)
cx.with_motion(policy, ...)
cx.without_motion(...)
```

Step 6: add motion modifiers.

```text
.motion_opacity(...)
.motion_offset(...)
.motion_scale(...)
```

Step 7: add reduced motion.

```text
App-level reduced motion setting
policy resolution override
```

Step 8: add structural motion.

```text
list identity transitions
matched geometry
navigation transitions
```

## Review checklist

A control using first-class motion should satisfy:

```text
[ ] no previous visual value stored in control model/control state
[ ] stable MotionKey per animated visual property
[ ] semantic MotionRole instead of direct duration/easing in control
[ ] no direct Animation::new in control template
[ ] initial render snaps to target
[ ] interruption starts from current rendered value
[ ] reduced motion requires no control changes
```

## Red flags

```text
appearance.foo_motion
model.previous_foo
format!("motion-{target}")
animate_f32 in a control template
per-control animation booleans
control-specific motion aliases
```

## Desired switch readability

```text
let target = if model.is_on { 1.0 } else { 0.0 };
let progress = cx.motion_value(key!(model.id, "thumb-progress"), target, Responsive);
let left = off_left + travel * progress;
```

That is the bar.
