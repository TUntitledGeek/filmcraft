Implemented follow-up: one transient clip drawer with canonical read-only hierarchy and a unique intrinsic Opacity editor. See the V7.4 section in v7-implementation-status.md. The remaining architecture below is still future work.

# V7 animation integration: source audit and implementation boundaries

Status: integration plan, **not an implemented animation workspace**. The first V7 runtime
slice changes browser host controls and Wiggle eligibility; no animation schema or timeline
geometry has changed. Read `v7-implementation-status.md` for the complete scope ledger.

## What can be recovered from the HTML

The supplied HTML contains the layout, visual tokens, state transitions and event handlers.
Its four scripts define a toy animation store, inline disclosure/filters, Focus Clip,
value/speed graph controls, inspector synchronization and demonstration shortcuts. Inspecting
these sources is enough to recover that interaction specification without clicking the page.
It does not establish that the production application's corresponding features work.

| Reference behavior | Production owner / constraint |
| --- | --- |
| Clip/effect/property hierarchy | Existing sequence `TrackItem.effects`, `EffectInstance.params`, parameter definitions |
| Current numeric value | Canonical `Param` evaluation at the effect's source-time domain |
| Static/animated edit and key insertion | Existing `effects.setParam`, `effects.toggleAnimation`, keyframe commands |
| Key selection and group retiming | Shared `KeyframeRef` selection and Effect Controls interaction machinery |
| Inline time positions | Main Timeline `Layout::x_of` and ruler transform; never a second 0–100% ruler |
| Focus Clip | A view of one real clip instance; other sequence tracks remain available |
| Value/speed graphs | Existing canonical interpolation/value graph; derivative display must describe its units and limits |
| Dock/floating graph | Existing dock and floating-panel persistence, with a shared selected property |
| Animated/Modified/All/Selected/search | UI-only filtering; no effect/key deletion and no project dirty state |
| Undo/redo | Engine history; never the prototype's separate JavaScript history |
| Mock Hub, save, cloud controls | Audit real host capabilities before exposing an action or claiming a status |

No JavaScript store, illustrative graph interpolation, artwork or icon geometry was copied.
The reference's filename keys, floating 30 fps positions and sample keyframe data are unsuitable
for production identity, frame mapping or persistence.

## Identity is the first unresolved animation dependency

Clips already have `ClipId`; property definitions have parameter IDs. General effects do not
yet have a stable instance UID. `EffectSel` uses effect type plus occurrence, and existing
inspector disclosure strings use clip ID plus array index. Neither alone proves identity across
same-type duplicate/reorder/remove/undo operations. Graphic-layer UIDs cover only graphic layers.

Choose and test a compatibility-safe identity strategy before persisting effect disclosure or
binding a detached graph to an effect. A filename, current track, array offset or content hash
must not silently become the identity. Validate selection against the current project after
edits and clear stale references on project replacement. Do not change existing clip IDs or
render semantics merely to allocate view state.

## Geometry and time requirements for the next slice

Reserve each expanded clip's drawer height in the containing track's vertical group. Multiple
expanded clips on one track stack; they do not occupy the same Y coordinates. Keep main clip
row rectangles separate from drawer rectangles, and exclude drawers from clip trim, marquee,
drop-to-new-track and track resize hit zones. Include them in video/audio scroll extents and
clip painting. With all drawers closed, retain the current geometry and editing interactions.

Read actual `Tick` key times and use existing source/effect-time mappings. Audit frame holds
and Hold Filters separately; do not assume an inverse linear speed formula is correct for all
future retiming paths. A non-unique or unsupported mapping needs an explicit unavailable edit
state instead of moving a key to an invented time. Trim/split/slip/reverse/speed/nesting and
transition-domain tests are still required before animation acceptance can pass.

Persist view preferences with UI/workspace state, not in an alternative project animation
store. Scope clip disclosure to the project, sequence and workspace. Add shortcuts through
the command registry only after auditing existing editing shortcuts and text-input focus.
