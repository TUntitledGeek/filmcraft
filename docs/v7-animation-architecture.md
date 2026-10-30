# V7 inline animation architecture

V7.5 replaces V7.4's fixed 112-point inspector and nested scrollbar with a measured
**clip → effect group → property → keyframes** hierarchy in the real Rust/egui timeline.
Read [the implementation ledger](v7-implementation-status.md) for verification and the entire
unfinished V7 scope. The HTML and plan remain workflow references. Their mock JavaScript
store, filename identity, percentage time and separate undo history are not production code.

## Canonical data and bounded identity

The frame's canonical Sequence supplies TrackItem.effects, EffectInstance.params and typed
ParamDef definitions. The view stores disclosure only: actual ClipId, unique effect type,
schema section and parameter ID. It never stores animation values or keyframes. Project/media
replacement resets the view; changing sequence/workspace resets its scope; missing clips and
ambiguous effects are pruned before layout.

Effects still lack persistent instance UIDs. Unique types are resolved afresh against the
current clip before commands. Duplicate instances remain visibly closed with an explanation.
This is an interim safety boundary, not persistent effect identity. Frame-local row offsets
address an immutable snapshot; they are never disclosure keys. Filenames, tracks, offsets,
occurrence numbers and content hashes must not become persistent identity. Before saving
per-effect disclosure, implement compatibility-safe UIDs and duplicate/reorder/remove/undo,
serialization/import and legacy-project tests.

## Measured layout and shared scrolling

inline_animation::prepare builds a plan per disclosed clip: height is exactly row count ×22
points. Newly opened clips show effect headers only. Fixed attributes precede added effects,
matching Effect Controls; added effects retain their order and processing order is untouched.
Lumetri sections come from schema groups. Opening a property's key lane adds one measured row.
Closing groups removes descendant geometry without removing any project animation.

The existing video/audio divider is unchanged. Track extents include main rows plus all their
clip plans. Multiple clips on one track stack in sequence order, and different track groups
remain disjoint. Existing bottom-up video and top-down audio scrolling are retained. The main
track bars and wheel gestures scroll the whole group. There is no nested ScrollArea or second
vertical scrollbar. Opening anchors the owning main row at the top of its existing half;
subsequent group disclosure preserves that row's position. Resizing reveals an active owner
that was previously visible. Intentional scrolling can move it offscreen naturally.

At short heights, paint and interaction are clipped to the video/audio half; both halves
remain available. Closing the final clip restores compact scroll positions. Main row rectangles
remain separate from animation rectangles. Disclosures and visible animation rows exclude
clip trim/move/marquee/new-track gestures, while wheel/zoom still use the timeline. Actual keys
use the same Layout::x_of(Tick), zoom and horizontal scroll as clips and the main ruler.

## Editing and time

A reusable property row supports scalar bounds/units/precision, angle, point, wheel, boolean,
choice and color controls. Text/curve/path values have readable summaries and are read-only
inline; specialized Effect Controls remain available. Opacity uses this same component.
Mutations dispatch existing effects.setParam, effects.toggleAnimation, effects.addKeyframe and
effects.toggleEnabled commands. Numeric drags use engine history merging. Static edits remain
static; animated edits write canonical playhead keys. The inspector follows the edited real
clip/effect. Undo/redo, serialization and interpolation remain engine/project responsibilities.

This milestone permits inline editing/key positioning only on positive-duration, normal-speed,
forward, non-held clips. Time remains exact Tick; the supported inverse is
clip.start + key.time - clip.source_in. Trimmed-out keys remain stored but are not drawn outside
the clip. Values use canonical effect-time evaluation. Reverse, speed changes, frame holds and
Time Remapping are explicitly read-only inline with an explanation; existing clip/Effect
Controls workflows remain available. No speculative inverse is used.

Key lanes support animation toggling, add/remove, previous/next and clicking a key to seek.
Inline key selection, group retiming and interpolation editing are pending. Diamonds are click
targets, not drag handles. Existing Effect Controls key editing remains intact.

## Required follow-up

| Requirement | Integration boundary |
| --- | --- |
| Persistent duplicate-effect disclosure | Stable model UIDs and backward-compatible round-trip/reorder tests |
| Selection / group retiming | Shared KeyframeRef selection and existing preview/commit/cancel commands |
| Reverse / speed / holds / split / slip | Canonical domains, invertibility and cut-boundary regression matrix |
| Interpolation / Bezier influence | Existing canonical commands and graph machinery |
| Animated / Modified / All / Selected / search | UI-only filters; never delete data or dirty the project |
| Animation toolbar / shortcuts | Registered commands; preserve editing and text-input bindings |
| Focus Clip | View one real clip with the ordinary timeline recoverable |
| Dockable / floating Graph Editor | Existing docking persistence plus safe property identity |
| Value / Speed graphs | Canonical interpolation, correct derivative units/discontinuities |
| Saved disclosure preferences | Workspace/UI state after identity is safe; no alternative animation store |
| Remaining original UX | Full master-prompt ledger and specialized workflows |

Visible rows are culled; planning still walks expanded canonical effects each frame. No
large-project or frame-pacing performance claim follows from layout tests. GPU Wiggle and the
renderer are unchanged. Severe Wave Warp + Lumetri + Blur remains a separate pending task.
