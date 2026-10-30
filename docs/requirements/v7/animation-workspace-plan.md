# FilmCraft V7 — Animation Workspace

**Status:** Design and implementation specification, 29 October 2026  
**Visual reference:** `filmcraft_v7_animation_workspace.html`  
**Baseline:** `TUntitledGeek/filmcraft`, `performance-upstream-integration`, verified checkpoint `a7c98a5e`  
**Principle:** Bring After Effects–style property disclosure and graph editing *into* FilmCraft's existing Premiere-style multi-clip editor. This is not a replacement application or a request to port the HTML into the runtime.

## 1. Product contract and design boundaries

FilmCraft must retain the Unified v2 visual language: thin panel boundaries, compact tabs, professional editing controls, familiar sequence ruler, tracks, Source/Program separation, and existing light/dark palette. The ordinary editing experience must be **unchanged when animation is collapsed**. The editor should scale from a 30-minute documentary with hundreds of clips to a five-second animated title without switching editing paradigms.

Three synchronized presentations of **one real project animation model**:

1. **Inline:** Disclosure triangle on a particular *clip instance* expands an animation drawer just below the track containing that clip. Effect groups and parameter rows have their own disclosure states, aligned to the sequence ruler. Multiple clips may be expanded; their drawers stack vertically without covering neighboring tracks.
2. **Focus clip:** A single selected clip's property hierarchy stays open while the rest of the sequence remains visible. Selecting a different clip switches the focus rather than opening another massive stack of drawers. Used for complex edits, many effects, or short clips.
3. **Graph Editor:** The selected parameter opens in a dockable or floating editor. First provide interactive value curves and the existing speed graph. Changes use the exact same command/state as inline diamonds and Effect Controls. No cloned keyframe data.

The right-hand Properties / Effect Controls panel **remains usable**. It is the inspector for precise values, while the timeline supplies spatial/time context. Neither panel may become a separate truth for animation.

### Deliberately not part of V7 implementation

A new JavaScript expression runtime, general AE parenting/null-controller system, an AE-style composition layer replacement, a universal OpenFX plugin host, a brand-new project serialization format, or arbitrary 2D tangent interpolation without first changing the underlying model. These may be future phases.

## 2. User-facing hierarchy

```text
Sequence: Main Edit
  Video 2:  Misty_Forest.mp4 [collapsed]
  Video 1:  Ocean ... City_Night_Drive | Desert_Dunes.mp4 ▾ | Misty_Forest | Neon
    Desert_Dunes.mp4  [focused instance / animation header]
      ▾ Motion                     [eye / bypass] [key count]
          Position X     960.0 px  ◇  ⌁    ◆ ─── ◆ ─────◆
          Position Y     540.0 px  ◇  ⌁
          Scale          100.0 %   ◇  ⌁
          Rotation       0.0°      ◇  ⌁
      ▾ Opacity                    [eye / bypass]
          Opacity        65.5 %    ◇  ⌁    ◆ ── ◆ ──────◆
      ▾ Lumetri Color
          Exposure       +0.60 EV  ◇  ⌁    ◆ ── ◆ ──────◆
      ▸ Gaussian Blur
      ▸ Wave Distortion
  Audio 1:  synced clip audio; independent audio automation as needed
  Audio 2:  Ambient_Score.wav ▸
```

Disclosure levels: **track** (optional track automation), **clip instance**, **effect group**, **property**, and for existing masks **mask group**. Preserve actual effect order (intrinsic vs ordered added effects), bypass, mask priority, and image-processing semantics. Do not reorganize effects alphabetically or turn their presentation order into a silent processing-order change.

## 3. Discovery, filters and shortcuts

**Default:** all drawers closed when opening a project or entering the ordinary Clean Editing workspace, unless a prior saved workspace/view-state explicitly restores them. Newly selected or newly added effects can auto-expand if that was the current editing action. Never open every effect automatically.

**Visibility filter:** `Animated` / `Modified` / `All properties` / `Selected property`. Filter applies to *display only*, not to effect evaluation, keys, bypass, or stored project data.

**AE-inspired shortcuts:** `U` reveals animated properties; fast `U U` reveals modified properties; `P` Position, `S` Scale, `R` Rotation, `T` Opacity, `E` Effects. These must be **context-aware**: only in timeline/animation focus, never while typing, never if conflicting with FilmCraft's existing active editing shortcuts. A discoverable command/shortcut settings panel takes precedence over hardcoded keys. Exact mappings are proposals pending shortcut audit.

Expanded state should be stable per clip **identity**, effect **identity**, parameter ID and workspace. Do **not** use track index + filename or effect array index alone: duplicated clip names, splits, effect reorders and track moves would otherwise break disclosure state.

## 4. Core interaction workflows

### Workflow A — Add and edit a keyframe

1. Select a clip, expand it (`▸`) or choose **Focus clip**.
2. Expand an effect, e.g. Opacity, and its property row.
3. Put the sequence playhead on the desired frame. Numeric field shows interpolated value at the current time.
4. Click `◇` beside the property or click its stopwatch if animation is not yet enabled. If the playhead already coincides with a keyframe, the diamond should reflect that (filled/highlighted); a separate delete action or explicit toggle removes it. Do not accidentally create duplicates at the same exact frame.
5. Change the numeric value to adjust/create the keyframe at the playhead **only when animation is enabled**. When animation is disabled, change the static property value without silently creating animation.
6. Drag diamond horizontally to retime (frame snapping by default, modifier for subframe precision where semantics support it). Select multiple keys with Shift/Ctrl and drag as one operation. Arrow-key nudging, copy/paste, and Delete should work using existing commands when supported.
7. Right-click diamond for Linear, Bezier, Auto Bezier, Continuous Bezier, Hold, Ease In/Out, or delete; menu must correctly describe current mode.
8. Inspector, inline row, monitor, and Graph Editor update from the same committed project state.

### Workflow B — Edit a curve

1. Click `⌁` beside the property or use **Graph Editor** in timeline toolbar.
2. Open it docked (when possible) or as floating; it can be resized, scrolled, and navigated without changing the main sequence layout.
3. Select **Value** (actual property values) or **Speed / Velocity** (derivative, with units). Support clip-fit, selected-key-fit, pan/zoom and numeric range; retain current key selection.
4. In Value mode, drag a point to change time/value and drag the existing Bezier influence handles. Hold/Linear/Bezier results must match the engine's renderer, not just a hand-drawn curve.
5. Display X/Y components of vector parameters independently. Where independent tangent control is *not* supported by the current underlying interpolation schema, keep existing influence semantics and state this limitation until the schema is extended and tested.
6. A more advanced graph in a later phase may add independent spatial/temporal tangent handles, influence percentages, velocity input, easing presets, axis isolation, fit selection, and multi-key transform.

### Workflow C — Modify a motion path over Program

Once motion-path overlay is implemented for real (in V7 prototype it is **illustrative only**), selecting Motion → Position reveals keyframe anchors and an overlaid spatial path in the Program monitor. Each anchor corresponds to an existing position keyframe. Dragging an anchor edits the X/Y values under one undoable gesture. Path display should update for rotation/scale changes and clip speed. Spatial Bezier tangents are a future model change; do not pretend a temporal easing handle is a spatial tangent.

### Workflow D — Collapse / switch / preserve layouts

Closing the Animation drawer never deletes or disables keyframes. Switching between Inline and Focus never duplicates animation rows or projects. Selecting a neighboring clip in Focus keeps the chosen filter, updates the inspector, and loads that clip's own disclosure state. A separate Graph Editor tracks the selected property by stable IDs and closes gracefully if the source clip or effect is deleted. Reset to default affects **UI layout**, not clip animation.

## 5. Timeline and timebase contract (non-negotiable)

Existing source facts: FilmCraft uses exact sequence ticks; clip keyframes are stored in **source/media time**, while timeline positions are sequence ticks. `TrackItem` owns ordered effects and their parameters; clips on the same track do not overlap as ordinary `TrackItem`s. Existing `effect_controls.rs` maps keyframe times to the timeline and supports the current interpolation menu and graph rows.

- Draw a key at **sequence position** derived from the clip's source-to-sequence mapping. Do not store duplicate sequence-time keys merely to make UI alignment easy.
- A keyframe's physical lane X coordinate must use the *same ruler transform* as clips, playhead, markers and transitions, including timeline zoom, scroll, RTL where supported, and high DPI.
- Trim: preserve media-time keys and only show/enable those visible within the current clip's displayed source interval. Do not delete hidden out-of-range keys without explicit command.
- Clip move: change the sequence position of the drawn keyframes without changing their media times.
- Slip: the visible keyframe subset changes because the source interval changes.
- Split: follow existing project semantics, not assumed duplication; keyframe evaluation must remain visually continuous at split boundary, and source-time identity must be correct.
- Speed change and reverse: implement mapping for each rendered segment and ensure keyframe navigation orders keys by **display sequence time**, not necessarily ascending source time. The existing mapping/tests must be reused or extended.
- Nonlinear speed ramps and time-remap ambiguity: avoid assuming that a source time has one unique timeline inverse. Where it does not, define displayed crossings by the applicable clip segment or disable ambiguous drag and show clear feedback until a correct mapping exists.
- Nested sequences: distinguish the nested instance's own effect stack from effects inside the nested sequence. Editing child sequence effects requires deliberate context switch.
- Clip transitions may sample frames outside simple visible regions; preserve rendering semantics; UI disclosure must not change effect evaluation.
- Audio automation: track and clip automation use different time domains as documented in `docs/architecture.md`. Do not conflate their keyframe lanes.
- Always use deterministic snapping and stable tick math, not floating-point percentages as the final Rust implementation.

## 6. Layout collision and performance rules

**Track organization:** The main clip rows remain visible at their standard heights. A clip drawer is inserted **after the containing track's clip row**, not absolutely painted on top of the next video/audio track. For multiple expanded clips on one track, either stack drawers by selected clip identity or use an explicit focused chooser when horizontal/vertical room is limited. Limit auto-expanded drawers; never explode the height of a large timeline without a click.

**Short clip:** The drawer is allowed to span the timeline's full viewport with property names in a fixed left label column, while the animated region highlights only that clip's real horizontal time span. Do not force labels inside tiny clip blocks.

**Scrolling:** The ruler and playhead stay synchronized. Property rows scroll vertically within the timeline, while timeline horizontal scrolling and zoom apply identically to clips and keys. A narrow viewport should retain editable labels and keys; side labels may be resizable.

**Performance:** Virtualize off-screen property/clip rows where warranted; evaluate only visible UI rows; reuse text/layout buffers; draw keyframe diamonds in a batched layer; avoid copying entire sequences per repaint; cache graph samples keyed by project revision, clip, effect, property, time window, zoom and value range. Animation focus must not add render workload simply because controls are expanded. Coalesce in-progress parameter drags and preserve one clean undo action on release.

**Accessibility:** All disclosure buttons expose clear labels and open/closed state, keyboard activation and visible focus. Property values must be editable by keyboard. Different interpolation modes should not be conveyed only by shape or color.

## 7. State and engine integration

- Reuse `TrackItem`, `EffectInstance`, `Param`, `KeyframeRef` and the existing commands: `effects.addKeyframe`, `effects.moveKeyframes`, `effects.setKeyframe`, `effects.setInterpolation`, `effects.deleteKeyframe`, plus selected-effect/parameter commands.
- Introduce a **view-state layer** for `expandedClipIds`, `expandedEffectIds`, `expandedPropertyIds`, `filter`, `viewMode`, `graphDockState`, `activeClipId`, `activeParameterRef`, ruler zoom/scroll, and curve selection.
- Put global UI preferences in existing Preferences; keep layout state separate from project content unless an explicit project-local view-state option is defined.
- Track object lifecycle: relink, rename, duplicate, split, delete, reorder effects, undo/redo, project reopen, workspace switch.
- Avoid duplicated keyframe ownership between Effect Controls, inline timeline and floating graph. Re-render from one immutable/cached snapshot as feasible and send engine commands through existing session routes.
- Dirty/unsaved project flag should reflect animation edits, not disclosure-only or graph zoom changes.
- Maintain native desktop and browser parity without introducing HTML UI into egui; HTML file is a **reference artifact**.

## 8. Exact V7 prototype scope and limitations

The HTML prototype demonstrates:

- Flat FilmCraft-style timeline, source monitor placeholder and inspector, light/dark themes.
- Inline clip expansion and an optional single-clip focus mode.
- `Animated / Modified / All / Selected` filters.
- Parametric numeric fields, static-versus-keyed edit behavior, keyframe addition, drag to retime, right-click interpolation, selected-key sync.
- Detachable graph with interactive **value curve**, Bezier influence handles, and read-only **speed preview**.
- AE-inspired contextual shortcut demonstrations and an optional decorative motion path overlay.
- The earlier Concept UI's project hub, menus and settings in their existing non-production state.

The prototype does **not** decode actual media, evaluate real project animation in Rust, implement live full-featured Bezier tangent mathematics or expression scripting, persist data to FilmCraft project files, or establish real parent/child rendering. Some controls are demonstrative and do not implement production semantics. In particular, the sample HTML uses simplified numerical interpolation, while the real implementation must use FilmCraft's canonical keyframe evaluation.

## 9. Phased implementation for Codex

### Phase 0 — Preserve verified base and test inventory

Create a fresh branch from `performance-upstream-integration`. Read performance handoffs and actual rendering/keyboard/animation code. Save screenshots and baseline tests. Keep original Unified layouts, native builds and already-verified performance optimizations.

### Phase 1 — Safe disclosure and filters (ship-quality MVP)

Add per-clip expand/collapse with keyframe dots aligned to sequence ruler, effect hierarchy, Animated/Modified/All filters and selectable property rows. Keep right inspector synchronized. No new animation schema. Test trim, split, move, reverse, speed changes, undo, layout persistence and large projects.

### Phase 2 — Full inline edits

Enable add/delete, multi-select drag, keyboard nudge, numeric edit and interpolation context menu through existing engine commands. Unify key selections with Effect Controls. Respect static parameter changes versus animated edits and selected-key identity. Stress-test rapid scrubbing and many opened drawers.

### Phase 3 — Focus and Graph Editor integration

Add focused view and dock/float graph. Adapt existing `effect_controls::graph_rows` value and velocity logic where appropriate; avoid parallel graph implementations. Preserve existing Bezier influence semantics, zoom, multi-selection and project undo. Test roundtrip selection between all views.

### Phase 4 — Motion overlays and precision

Add real spatial path overlays and refined XY property editing only after validating existing spatial interpolation. Improve graph handles and numeric easing only when underlying model can represent them correctly. Ensure image/output parity across CPU/GPU browser/desktop paths.

### Phase 5 — Future After Effects-style authoring (separate roadmap)

Optional controllers/nulls, parent-child transforms, property links/pick whip, safe visual expressions, animated text/shape internals, exposed properties of nested sequences, richer graph and preset libraries. Each is a separately scoped engine feature requiring serialization/export decisions, not just new timeline UI.

## 10. Acceptance criteria

- [ ] Ordinary Clean Editing with all animation collapsed looks and behaves like current FilmCraft.
- [ ] Opening two clips doesn't overlap, occlude or silently resize other tracks incorrectly.
- [ ] Selected clip change updates inspector, inline effects, graph and Program without stale values.
- [ ] Keys appear on the same frame of the main ruler at any zoom/scroll and through normal/reverse/speed-adjusted clips.
- [ ] Static parameter edit does not silently turn on animation.
- [ ] Add/remove/move keyframes, interpolation and graph influence edits produce correct output and undo/redo.
- [ ] Animated/Modified filters never change output or destructive project data.
- [ ] Split/trim/slip/reverse/nested sequence tests pass and no keyframes vanish from project unexpectedly.
- [ ] Graph and inline values match canonical engine interpolation, paused render and export.
- [ ] Large-project UI scrolling, selection and playback are no worse than baseline.
- [ ] Browser and native build/test gates pass; independent performance test uses existing `studio-review.mjs`.
- [ ] Dark/light theme, keyboard, pointer, narrow browser, high DPI and floating-dock interactions verified.
- [ ] No protected branch merged, no deployment, and verified commits/handoff pushed on development branch only.

### Suggested acceptance demonstration

Select `Desert_Dunes.mp4`, press **U**, expand Motion and Opacity, add a position keyframe, drag it, open the value graph, change easing influence, switch to speed preview, modify opacity from right Effect Controls, undo, reverse/speed-adjust the clip, collapse back to normal timeline, reload saved workspace and verify every frame and key remains consistent.

## 11. Additional targeted performance work (separate workstream)

The actual recent performance run measured about 29–30 distinct presented FPS on a specific 720p30 AVC fixture for no effects and Blur+Lumetri, while the procedural demo was still around 11 FPS. These results do **not** extend to the user's Wave/Wiggle/Glow observations. Benchmark those independently and investigate GPU eligibility, entire-chain CPU fallbacks, transition dual-source costs and motion-resampling allocations. Do not intertwine a major renderer rewrite with timeline-animation UI in one unverified change. Retain all existing GPU initialization and decoder-policy fixes.

---

**Recommended product decision:** Implement Phases 1–3 as FilmCraft's animation-workspace foundation. Keep Phase 4 selective and treat Phase 5 as a future After Effects-style authoring roadmap. The key competitive advantage is professional animation **inside** a fast, multi-clip editor—not making every clip a separate After Effects layer.
