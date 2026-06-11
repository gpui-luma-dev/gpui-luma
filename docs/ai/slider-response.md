# Slider Response Post-Mortem

This note captures the narrower interaction problem that surfaced during the neumorphic dial work: the dial response felt too fast and unstable in some regions, and the first implementation attempt to improve that response introduced new regressions. This is intentionally separate from the broader [slider-enhancement.md](file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/slider-enhancement.md) proposal.

---

## 1. Original Issue

The immediate problem was not that the SDK slider lacked angular support in general. The specific issue was that the neumorphic dial felt too sensitive and inconsistent during drag:

- grabbing the visible handle and rotating it around the dial was unreliable
- in some positions the value moved too fast
- edge and seam behavior near minimum and maximum was brittle
- trackpad or off-ring interaction needed a more precise mode

The request was effectively for a **better drag response model** for an angular control, not just a different coordinate projection.

---

## 2. Options Considered

### Option A: Pure Angular Track-Follow

Interpret pointer position as an angle around the dial center and map that directly to value.

Pros:
- intuitive when the user grabs the visible handle
- matches the physical metaphor of rotating a knob

Cons:
- can feel too fast when the pointer is not near the handle path
- needs careful seam handling around the `atan2` wrap
- can feel bad on trackpads or when dragging from the center area

### Option B: Pure Vertical Precision Drag

Ignore pointer angle during drag and translate vertical delta into relative value change.

Pros:
- stable and predictable for precision work
- good on trackpads
- easy to slow down with a pixels-per-sweep parameter

Cons:
- breaks the main affordance of grabbing the visible handle and rotating it
- feels wrong when the user starts on the indicator itself

### Option C: Generic Response / Curve API

Introduce a configurable response curve or easing-style function for drag-to-value mapping.

Pros:
- flexible
- can support many feel profiles later

Cons:
- too abstract too early
- does not directly solve the geometry question of when the dial should follow the visible handle versus use precision drag
- risks overdesign before the basic interaction policy is stable

### Option D: Hybrid Drag Policy

Use direct track-follow when the drag starts near the visible handle path, and use vertical precision drag when the drag starts away from that path. Lock the choice for the duration of the gesture.

Pros:
- preserves direct manipulation when grabbing the handle
- gives a slower precision mode away from the ring
- keeps the first design pragmatic and explainable

Cons:
- requires accurate agreement between rendered handle geometry and interaction hit geometry
- still needs seam handling and separate keyboard-step behavior

---

## 3. Pragmatic Choice

The pragmatic choice was **Option D: Hybrid Drag Policy**.

The intended model was:

1. If the pointer starts near the visible dial handle path, use direct angular track-follow.
2. If the pointer starts away from that path, use a slower vertical precision drag.
3. Keep the behavior fixed for the whole drag gesture so the mode does not switch mid-drag.
4. Make the slower precision mode configurable with a simple parameter such as `pixels_per_sweep`.

This was the smallest design that preserved the visible handle affordance while still addressing the "moves too fast" complaint.

---

## 4. What Went Wrong

The first implementation attempt went astray because several separate concerns were collapsed together.

### A. Angular geometry was treated as the whole problem

The initial design assumed that adding `Angular { min_angle, max_angle }` plus some drag math was enough. It was not. A dial also needs an explicit **drag policy**, not just angular projection.

### B. Mouse-down semantics were wrong

Early behavior effectively let the dial jump on press, which is slider-like but not knob-like. For a knob, pressing on the surface and grabbing the current handle are not the same gesture.

### C. Precision dragging was over-applied

Switching too hard toward vertical-delta dragging fixed some sensitivity problems but broke the expected "grab the tracker and move it around the dial" behavior.

### D. Seam behavior had to be repaired after the fact

The dial could hit minimum, continue moving, and wrap toward maximum because the drag path did not begin with a continuous unwrapped pointer-angle model.

### E. Drag smoothness was incorrectly coupled to `step`

Reducing `step` made dragging feel smoother, but it also changed keyboard arrow increments because the shared slider uses `step` for key actions. This made arrow keys appear broken even though focus and keybinding logic were still intact.

### F. Rendered geometry and interaction geometry diverged

The hybrid hit zone was initially aligned to the outer tick ring rather than the actual indicator orbit. That meant the system often chose precision mode even when the user was clearly trying to grab the visible handle.

---

## 5. Resulting Issues

The implementation produced a chain of regressions:

- jump on drag start
- loss of direct handle-follow behavior
- wrap/seam errors near minimum and maximum
- over-fast response in some regions
- poor track-follow hit detection
- keyboard arrows moving too little because drag smoothness and key step size were tied together

At that point, the interaction work had become unstable enough that reverting was the right decision.

---

## 6. Lessons

### Keep these concerns separate

- **Geometry**
  Angular arc definition such as `min_angle` and `max_angle`
- **Drag policy**
  Track-follow, precision drag, or hybrid gesture choice
- **Drag sensitivity**
  Pixels-to-value mapping for drag only
- **Keyboard increment**
  Arrow/Page key movement, independent from drag sensitivity
- **Rendered handle path**
  The actual visible orbit used for direct-manipulation hit testing

### Generalize later

The knob interaction model should be proven first, then promoted into the SDK shape. The initial implementation generalized too early while the interaction policy was still unsettled.

### Validate user gestures, not just math helpers

Helper tests for angle wrapping were useful, but the real acceptance criteria were gesture-level:

- can the user grab the visible handle from max and rotate it
- does minimum clamp without wrapping to max
- does off-ring drag enter a slower precision mode
- do keyboard arrows still move by a visible amount

---

## 7. Follow-Up Recommendation

If this work is retried, start with a narrower goal:

1. Keep the broader slider generalization separate from dial-response tuning.
2. Add a drag-only precision parameter that does not affect keyboard step size.
3. Keep direct handle-follow as the default knob affordance.
4. Only add hybrid track-follow versus precision mode after the visible handle path and hit zone are explicitly modeled together.

That keeps the next attempt focused on the actual response problem instead of reopening the entire slider architecture at the same time.
