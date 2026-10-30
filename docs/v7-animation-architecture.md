# V7 inline animation architecture

V7.5 replaces V7.4's fixed 112-point inspector and nested scrollbar with a measured
**clip → effect group → property → keyframes** hierarchy in the real Rust/egui timeline.
Read [the implementation ledger](v7-implementation-status.md) for verification and the entire
unfinished V7 scope. The HTML and plan remain workflow references. Their mock JavaScript
store, filename identity, percentage time and separate undo history are not production code.

## Canonical data and persistent instance identity

The canonical Sequence supplies TrackItem.effects, EffectInstance.params and ParamDef schemas.
Schema v16 adds owner-local EffectInstance.instance_id and Project.next_effect_instance_id.
Zero denotes an unattached definition/preset. The checked allocator assigns IDs at load/edit
boundaries; legacy missing/duplicate metadata is repaired without changing values, keys, masks,
processing order or existing graphic LayerExtra.uid. New edits reject ambiguous duplicate IDs.
The identity is (ClipId, instance_id), so snapshots and copied/split clips can retain internal
IDs under different clip owners. New copies within one stack use fresh_copy; intrinsic value
replacement preserves the destination ID. A non-undoable session allocation floor prevents
abandoned redo branches from recycling retired IDs. Save/recovery snapshots retain that floor.
Older readers refuse v16 rather than silently dropping identity. Existing string/index command
selectors remain compatible; effect:{instanceId:id} is authoritative and never falls back.

Disclosure stores ClipId, instance ID, section and property ID. Duplicate effects open and edit
independently. Frame-local row offsets address only the immutable paint snapshot. Document
replacement has a generation distinct from revision/sequence IDs: reused demo IDs cannot inherit
old disclosure/selection. Missing owners/effects/properties are pruned before layout. None of
these references own key values or create a second animation store.

EffectSel, MaskSel, asynchronous mask tracking and KeyframeRef prefer stable IDs; UI index hints
are rebased and deleted references cleared. New Clip Fx Editors use a ClipInstance target.
Effect Controls queued actions resolve offsets before executing stack mutations. Inline instance
menus duplicate, move up/down, reset and remove added effects; fixed/graphic ordering stays under
its specialized commands. Legacy unique-type automation names remain aliases, never identity.

Identity metadata is excluded from preview content hashes, including nested mixes/transition
instances; allocation, legacy reopen and undo must not invalidate rendered picture/audio caches.
Multicam processing equality ignores owner identity; appended outer effects get fresh identities.

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
The V7.6 context menu reuses canonical interpolation and single-key move/delete commands.
All seven existing temporal interpolation modes are available; glyphs reuse Effect Controls'
linear/hold/curved painting. Move to playhead uses the exact current source/effect Tick and is
disabled outside the owning clip or when another key occupies the destination. A move retains
the key's value, interpolation and influence, with one ordinary engine undo step. Menu actions
resolve the persisted instance at dispatch and do not own independent animation data.

Inline and Effect Controls now share the existing UiState.keyframe_selection. References carry
clip, authoritative effectId, parameter, optional mask and exact source Tick; effect is only a
legacy/index hint. Individual/Shift/Ctrl selection and marquee selection use one common gesture
implementation. Inline registers marquee backgrounds only in actual key lanes, preserving clip
bodies, trim edges, drawer labels, scrolling and ordinary clip marquee. The inspector follows
the real selected clip/effect. Selection can span independently disclosed clips/tracks.

Group drags preview a common timeline delta in UI state; project values remain unchanged until
release. The anchor snaps to sequence frames; Shift additionally magnets to visible keys and
the playhead. Movement is bounded by the intersection of selected clips' visible ranges.
Escape, document replacement or concurrent project revision cancels a preview. Commit uses the
existing atomic effects.moveKeyframes command once, retaining values/interpolation/influence
and relative timing. The UI requests collision rejection; occupied unselected destinations
refuse the entire edit. Legacy command overwrite behavior remains available to older clients.
strictForward validates the supported inline domain again at the engine boundary. Unsupported
reversed, speed/held, Time Remapping and mixed mask selections are not enabled inline.

Key identity still includes source time, rather than a persistent per-key UUID. When undo or an
external edit removes a referenced time, selection is safely pruned; moved-key highlighting is
not promised to survive undo/redo. Canonical animation/history does survive. Graph Editor work
must preserve this boundary or add deliberate selection-history mapping, never infer another key.

Each clip drawer has compact All/Animated/Modified/Selected filters. Modified compares schema
defaults resolved to actual sequence/source sizes; Selected shows parameters containing selected
keys. Filters change measured view geometry only. Existing per-property stopwatch, previous,
add/remove and next controls remain the functional animation toolbar. Global toolbar commands,
property-only selection and search remain pending.

## Required follow-up

| Requirement | Integration boundary |
| --- | --- |
| Saved duplicate-effect disclosure | Runtime UID disclosure implemented; persistent workspace preferences remain |
| Selection continuity through time undo | Canonical shared selection/drag implemented; safe pruning instead of inferred per-key identity |
| Reverse / speed / holds / split / slip | Canonical domains, invertibility and cut-boundary regression matrix |
| Bezier influence / graph editing | Existing canonical commands and graph machinery; inline temporal menu is implemented |
| Property-only selection / search | Four view-only filters implemented; property focus/search remain |
| Animation toolbar / shortcuts | Registered commands; preserve editing and text-input bindings |
| Focus Clip | View one real clip with the ordinary timeline recoverable |
| Dockable / floating Graph Editor | Existing docking persistence plus safe property identity |
| Value / Speed graphs | Canonical interpolation, correct derivative units/discontinuities |
| Saved disclosure preferences | Workspace/UI state after identity is safe; no alternative animation store |
| Remaining original UX | Full master-prompt ledger and specialized workflows |

Visible rows are culled; planning still walks expanded canonical effects each frame. No
large-project or frame-pacing performance claim follows from layout tests. GPU Wiggle and the
renderer are unchanged. Severe Wave Warp + Lumetri + Blur remains a separate pending task.
