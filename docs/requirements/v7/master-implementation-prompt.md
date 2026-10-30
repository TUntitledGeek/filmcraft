# FILMCRAFT — V7 ANIMATION WORKSPACE, PROFESSIONAL UX REFINEMENT, EFFECT PERFORMANCE & FUNCTIONAL POLISH

## Mission and source material

**Repository:** https://github.com/TUntitledGeek/filmcraft  
**Base branch:** `performance-upstream-integration`  
**Last verified performance checkpoint at the time this instruction was prepared:** `a7c98a5e` (verify the actual current remote/local HEAD before branching).  
**Create a NEW implementation branch:** `filmcraft-v7-animation-and-ux` (choose a nonconflicting suffix if needed).  
**Do not merge, deploy, publish a release, or alter protected branches without explicit approval.**

I am attaching two substantial references:

1. `filmcraft_v7_animation_workspace.html` — a clickable prototype demonstrating the desired animation interaction model, inline clip/effect disclosure, clip focus, graph editor, timeline hierarchy, and several surrounding UX ideas.
2. `FilmCraft_V7_Animation_Workspace_Plan.md` — the detailed animation workspace design, data/timebase constraints, implementation phases, tests, and deliberate limitations.

**READ BOTH ATTACHMENTS IN FULL BEFORE MAKING ARCHITECTURAL CHANGES.** Open and interact with the HTML in an ordinary browser; inspect its expanded/collapsed clips, Animated/Modified/All/Selected filters, inspector synchronization, keyframe actions, graph modes, workspace arrangements, dark/light appearance, and narrow layout. Read the Markdown specification carefully, including its edge cases and non-goals. If the files are not available in the Codex workspace, locate the uploaded attachments rather than guessing from this prompt.

**Reference hierarchy and design intent:**
- The **actual latest FilmCraft Rust/egui application, its screenshots, established Unified v2 visual language, and existing functional behavior** are the source of truth for visual identity, precise layout density, rendering semantics, keyboard bindings, and what already works.
- The **V7 HTML** is a CLOSELY FOLLOWED DESIGN AND INTERACTION REFERENCE for the desired outcome, particularly how animation is exposed inside the timeline and accessed in a floating/docked Graph Editor. It is **not** production code, a replacement UI framework, a pixel-perfect mandate, a full mathematical implementation, or a request to rebuild FilmCraft as a web HTML app.
- The **V7 Markdown plan** is the primary architectural/behavioral blueprint. Treat its timebase, ownership, data-consistency, edge-case and acceptance requirements seriously. Where its illustrations conflict with the real engine, adapt the design to the engine's correct existing behavior and document the decision.
- Neither attachment is perfectly styled nor complete. Use sound product and engineering judgment to improve them, while staying VERY close to FilmCraft's existing appearance. **Do not invent a fresh design system or radically redesign the app.**

The goal is a FilmCraft that remains an extremely capable, fast, familiar multi-track video editor when everything is collapsed, yet can expand into a powerful, After Effects-inspired animation editor **inside the same timeline**, with optional separate precision tools. At the same time, resolve the specific effect lag, confusing navigation, inconsistent layouts, fullscreen, and project/storage UX reported below. This work MUST NOT quietly abandon any of those subjects just because V7 animation is ambitious.

## 1. Non-negotiable protection of existing work

Inspect and preserve the existing work from `studio-pro`, `unified-studio-final`, and `performance-upstream-integration`. Read:

- `docs/studio-pro-review.md`
- `docs/unified-studio-final.md`
- `docs/performance-reliability.md`
- `docs/performance-architecture-review.md`
- `docs/performance-before-after.md`
- `docs/upstream-performance-inventory.md`
- `docs/verification/upstream-performance.json`
- `docs/architecture.md` and relevant tests.

Preserve the Unified UI, responsive Source/Program behavior, docking/layout persistence, input and drag/drop corrections, project operations, decoder selection, prior WebGPU async-validation repair, playback scheduling/prefetch, cache and worker-lifetime improvements, audio-export transactional behavior, native GPU-export/readback improvements, and CPU/GPU rendering correctness.

Previous Codex code is a **presumptive baseline, not untouchable**. You MAY replace it if investigation reveals a faster, cleaner, or more correct implementation; explain the original purpose, evidence of deficiency, proposed alternative, tests and measured outcome. Do not blindly revert measured improvements, import upstream UI wholesale, create competing animation stores or redundant schedulers, or confuse an experimental archived threading patch with an enabled production feature.

The latest verified browser performance milestone involved a specific Windows/Intel Chrome 720p30 fixture: median 23.09→29.82 distinct frames/s without effects, 24.57→29.51 with Blur+Lumetri; median deadline drops fell to zero. Procedural demo remained ~11 fps. These are **device/fixture-specific observations**, not universal 30/60fps guarantees. Keep baseline comparison conditions and diagnostics intact. Do not merge unverified threading code merely to make the prototype feel more advanced.

Start with a clean-tree/worktree/branch inspection. Preserve any user's local uncommitted changes. Create the new development branch from the verified current performance branch. Commit and push cohesive **verified checkpoints**. Never edit `gh-pages`, `main`, or other protected work directly.

## 2. Visual design: subtle refinement, not a new product

Use real screenshots of the current application (from the project and live Chrome inspection) as the styling reference. In particular, preserve:

- Compact menus, flat toolbars and modest control heights.
- Thin panel borders, normal panel tabs, subdued accents, restrained blue selection and limited corner radii.
- Dense professional track headers, recognizable clip color categories, ruler, transitions, audio waveforms and Source/Program tools.
- Large Program monitor, optional contextual Source monitor, accurate video aspect ratio and independent timecodes/playheads.
- Right-hand Properties / Effect Controls tabs, left-hand project/media tabs, full-width timeline below.
- Coherent but restrained light and dark themes and readable neutral monitoring surfaces.

Do NOT turn every item into a big rounded card. Do NOT introduce unnecessary second header rows, giant toolbar icons, saturated visual decorations, fake cloud controls, oversized padding, a completely new header, or wide empty spaces. Match the genuine FilmCraft/Unified v2 typography, row heights, docking system and visual vocabulary more closely than the standalone HTML does. Where the HTML makes a simplifying visual choice, preserve the more sophisticated real editor behavior.

The UI should be pleasant and modern *because it is consistent and thoughtfully organized*, not because it looks like a standalone web-dashboard mockup. Take matching before/after real-app screenshots in light/dark at 1656×894, a typical narrower laptop width, and at least one more responsive width; assess actual clipping and click targets.

## 3. V7's central feature: a professional, expandable timeline animation hierarchy

Deliver the following architecture using the existing Rust/egui UI and canonical project animation model. **Do not copy the mock HTML controls into production.** The existing FilmCraft track, clip, effect and property structures must remain the owners of editing data.

Hierarchy:

- Sequence and track — normal video/audio clip lanes stay visible at conventional heights.
- Clip instance — a disclosure arrow on an individual clip opens its animation drawer **beneath the containing track's main clip row**, without covering the next track or pushing neighbouring clips out of their sequence time positions.
- Effect group — e.g., Motion, Opacity, Time Remapping, Lumetri, Gaussian Blur, Wave, Wiggle, and applicable audio groups, respecting **actual** effect processing order and intrinsic effects.
- Property — e.g. Position X/Y, Scale, Rotation, Opacity, Exposure, Blur Radius; editable numeric values, enable/animation state, keyframe navigation and a Graph Editor affordance where supported.
- Keyframes / masks — diamonds and graph data tied to existing project keyframe IDs and time semantics, plus existing effect masks and animated mask parameters where supported.

All disclosure levels must be nested and independently collapsible. The initial normal editing timeline must remain essentially the current FilmCraft interface; no giant auto-expanded stack when a project opens. Persist appropriate view/disclosure preferences for the current workspace without changing or dirtying project animation data.

**Three synchronized views of ONE project animation state:**

**A. Inline animation:** Expand a specific clip, then its effects/properties to edit inline in the overall sequence timeline. Keyframe diamonds use the same horizontal axis as clips, markers and playhead. The main video and audio editing operations remain available.

**B. Focus Clip:** A more concentrated presentation of the selected clip's hierarchy when a short clip has too little horizontal width or a complex clip has many properties. Selecting another clip updates focus without duplicating data. The rest of the sequence remains visually accessible. Focus is a UI choice, not a new type of sequence or standalone project.

**C. Dockable/floating Graph Editor:** Open from a selected property. Make it resizable and compatible with FilmCraft's docking/layout persistence; support actual value curves, existing interpolation controls and an appropriately described speed/velocity view. Current prototype speed graph is read-only: do not falsely present derivative handles as editable until the engine supports them correctly. All three views, plus the existing right inspector, must share selection and project-command state.

The default collapsed UI remains extremely compact. Support multiple expanded clips cautiously: drawers for separate clips may coexist and stack underneath their owning track; ensure their labels, timeline ranges, clip context and vertical sizes never collide with one another or other track rows. Limit **automatic** expansions, allow user-requested expansions, and support a Focus mode when many clips are expanded or time spans are short. Never draw an expanded drawer over the main clips or adjacent tracks. Preserve stable horizontal ruler alignment even when the drawer's property names use a fixed-width column.

Implement a small, discoverable animation view toolbar: expand/collapse, Focus Clip, Show filter, open Graph Editor, and convenient Fit/Zoom controls. Integrate with the existing track toolbar rather than building a competing control system.

## 4. Property discovery and sane defaults

Adopt After Effects-inspired productivity, but adapted to a multi-clip editor:

- `Animated`: show currently animated properties.
- `Modified`: show properties whose values differ from their relevant defaults (including appropriately animated/masked changes).
- `All`: show all valid properties.
- `Selected`: show the selected effect/property without irrelevant content.
- Direct search for effect/property names, and quick access to recently edited properties when feasible.

These filters MUST only affect what is displayed; they must not enable/disable effects or alter rendering, keyframes, serialization, or saved content.

When navigating between clips, preserve a sensible identity-based expanded/collapsed state. Do not index solely by filename, track order or effect index: duplicate names, split clips, moved tracks and reordered effects should not cross-wire the state. Use stable existing identifiers where available and handle deletion/reorder/undo/reopen explicitly.

Newly selected or newly added effects may open automatically *in the context of the user's action*; do not auto-expand every effect. Provide Expand All / Collapse All with bounded behavior. When an effect is collapsed, display useful minimal summaries (e.g., Opacity 100%, Blur Radius 8px), not a huge parameter list. Make the panel efficient for everyday cutting and advanced animation alike.

After Effects-inspired `U` (Animated), quick `UU` (Modified), and contextual P/S/R/T/E property shortcuts are desired where compatible, but **audit the existing keyboard registry before assigning them**. Never intercept typing in text/numeric inputs, steal core editing commands, or rely on timing-sensitive shortcuts without tests and visible shortcut discoverability. Preserve the existing shortcut editor/preferences system and context precedence.

## 5. Real keyframe editing: no mock or disconnected state

Reuse the real model and commands: `TrackItem`, `EffectInstance`, `Param`, `KeyframeRef`, relevant masks and existing engine routes such as `effects.addKeyframe`, `effects.moveKeyframes`, `effects.setKeyframe`, `effects.setInterpolation`, `effects.deleteKeyframe`, plus existing selection/numeric/undo commands. Inspect actual signatures before reuse.

Required workflow:

1. Select/expand clip → effect → parameter.
2. Scrub to a precise sequence frame; value field reads the canonical interpolated value at the mapped media time.
3. Clicking the add-key diamond or enabling animation produces the correct keyframe, without duplicate keys at the same exact frame.
4. An unanimated property edit changes its constant value. When animation is enabled, editing at the playhead updates/adds the appropriate keyframe under existing engine semantics. Make animation state obvious.
5. Draw multiple keys in the correct sequence positions, drag one/many keys to retime (frame snap by default), with multiselection and correct modifier rules. Preserve existing select-all/marquee behavior and cancel/Escape semantics. Support keyboard nudge/copy/paste/Delete where the engine already provides it; otherwise add safely in a later verified step rather than faking it.
6. Right-click a keyframe for actual supported interpolation: Linear, Bezier, Auto Bezier, Continuous Bezier, Hold, Ease In, Ease Out, plus delete and relevant selection actions.
7. Numerical property editing, inline keys, inspector keys, Graph Editor values, Program render and export must agree on the *same* evaluated project state.
8. Coalesce drag gestures into meaningful undo operations, preserve undo/redo in all views and ensure no hidden keyframes are deleted by disclosure/filter changes.

A keyframe changed from anywhere must be reflected in the other views. A graph closing, a clip deselecting, or a workspace switching must not remove animation. The project dirty flag should reflect actual animation edits but **not** disclosure, graph zoom, filter or other view-only actions.

## 6. Value curves, speed graph, easing and future precision

The code already contains keyframe/graph functionality in `crates/ui-egui/src/panels/effect_controls.rs`, including `graph_rows`, selection helpers and interpolation controls. Inspect and **reuse or refactor** it rather than creating a second unrelated graph math implementation. The real renderer's interpolation, not arbitrary sample HTML curves, is authoritative.

Graph Editor MVP:

- Value mode: horizontal = correct time; vertical = numeric parameter value. Support selected-key indication, drag keys to adjust values and, where correctly defined, time; precise numeric editing; panning, zoom, fit clip/selected, units and sensible range behavior.
- For scalar and Vec2 parameters, show proper scalar or X/Y component curves; allow component selection while preserving model semantics.
- Display and edit the existing Bezier *influence* handles as actually represented by FilmCraft. Changing handles must produce the same interpolation as the render engine and be undoable.
- Speed/Velocity mode: show a genuinely derived derivative with meaningful units. It may be read-only initially, clearly indicated. Do not claim independent speed/temporal tangent editing when the model cannot represent it.
- Keep multi-key selection consistent between graph and inline rows.
- Avoid UI curve computations that copy entire project/sequence data on each paint; cache/virtualize appropriately.

Advanced features requiring separate engineering, NOT fake UI promises: independent incoming/outgoing temporal handles, true spatial tangents, rich curve presets, axis separation semantics beyond the existing model, velocity numerical controls, multi-key easing transformations, and full motion-path editing. These are optional subsequent milestones only if the model can be extended safely, with serialization, CPU/GPU evaluation and export parity tests. The illustrative HTML motion path is **not** production functionality. If implementing it, use real position keys and correct Program-monitor coordinate transforms, including rotation, scale and source mapping.

## 7. Exact timebase, trim, split, reverse and sequence invariants

This section is non-negotiable. FilmCraft stores many clip-effect keys in **media/source time** and clip positions in **sequence ticks**. Do not create separate sequence-time keyframe data simply to draw them inline. Use the existing canonical mapping, and audit its assumptions:

- Sequence-tick precision, exact snapping, frame rate/drop-frame handling, timeline zoom/scroll and high-DPI transforms must match clips, playhead, markers and transitions precisely.
- Clip move: keys' **displayed** positions move with the clip while key times stored in media/source domain stay correct.
- Trim: show the relevant visible subset; do not delete offscreen keys automatically.
- Slip: the selected source region and visible key subset change appropriately.
- Split: follow actual source/clip semantics and maintain animation continuity; don't assume every key is duplicated indiscriminately.
- Speed change and reverse: derive correct displayed time and order from the real speed mapping; previous/next keyframe navigation should use sequence presentation order when needed.
- Nonlinear time remapping and speed ramps: a media time can map to multiple sequence positions. Do not use an incorrect single inverse mapping. Handle displayed crossings per segment or constrain unsupported drag operations with explicit feedback until a correct implementation exists.
- Nested sequences: distinguish the enclosing clip instance's effects from child-sequence internal effects, and only enter nested editing by explicit context change.
- Transitions may request overlapping/out-of-range media; keyframe UI visibility must not change rendering semantics.
- Audio clip and track automation have different time domains; respect existing mixer/clip model rather than displaying incorrect shared key offsets.
- Any new keyframe/property model changes must preserve project load/save, existing projects and export correctness.

Build specific regression tests for every case above, including source In/Out and duplicated clip names, track moves, selected effect deletion/reorder, project reload and edit-history operations.

## 8. Timeline geometry, focus management and large-project performance

- Keep ordinary clip rows and professional track controls intact. Insert drawer height into timeline layout calculations; no overlap and no invisible hit zones over other tracks.
- A short clip may have a *full-width drawer* for readable labels and controls; the actual editable key region must still highlight/align with that clip's true ruler span.
- Maintain synchronized main horizontal ruler and shared scroll/zoom for clips and animation keys. Vertical track scrolling and animation-row scrolling should be predictable; avoid accidental outer-versus-inner scroll traps.
- Provide sensible disclosure limits and targeted Focus instead of expanding everything when selection changes.
- Correct pointer hit areas, drag thresholds, selection vs trim precedence, tool compatibility, transitions, track resize/collapse, focus and keyboard accessibility.
- For hundreds of clips and many animated properties, avoid enumerating/drawing every offscreen key every frame; virtualize visible rows, cache sampled curves using project revision/clip/property/window/range where beneficial, and avoid redundant cloning/layout allocations.
- Expanding animation controls must not itself trigger high-cost frame recomputation. Interactive value drags must not block the browser or regress playback. Existing adaptive preview quality may be retained and improved, but precise paused/final render must be correct.
- Maintain deterministic UI state across dock changes, light/dark themes, page reload, session restore and window resize. Verify correct behavior at common laptop widths.

## 9. Effect Controls: a compact inspector, not a competing animation application

The current right-hand Effect Controls UI is overloaded: many parameters start expanded, an always-present keyframe lane occupies space in narrow layouts, and an orange **Fusion Nodes** button is prominent in ordinary effect editing.

Implement:

- Compact expandable effect rows, with restrained visual separators rather than giant cards.
- Collapsed by default unless the user opened them, the current action selected/added an effect, or deliberately saved relevant disclosure state.
- Helpful compact values on collapsed headers, a selected-effect indicator, intrinsic vs user-applied effect distinction, and genuine processing order.
- Expand All / Collapse All; convenient search/find, enable/bypass, reset, masks, keyframe navigation, animation toggles, preset/save and context menus.
- Optional keyframe lane or shared external Graph Editor, not an always-visible narrow-right miniature timeline. If retaining an in-panel lane, make it genuinely hideable and restore existing professional users' access.
- Predictable vertical scrolling, parameter labels/value fields that fit a narrow dock, clean resize handles and keyboard access.
- Right inspector stays synchronized with the expanded inline property and Graph Editor. The same actual engine command is used for editing.
- Remove the orange **Fusion Nodes** button from the ordinary Effect Controls header; preserve Fusion through appropriate advanced/experimental workspace/menu access, without deleting node features.

Do not remove existing animation or effect features merely to make the list cleaner. This is organization of the same capability, not a simplification of the engine.

## 10. Actual performance defects: Wiggle, Wave and Glow

The following are **user-observed symptoms**, NOT established benchmark conclusions:

- Lumetri + Blur: smooth in the tested clip.
- Lumetri + Blur + Wiggle: stutters.
- Lumetri + Blur + Wave: stutters.
- Glow transition between `Desert_Dunes.mp4` and `Misty_Forest.mp4`: visibly laggy.

Investigate actual rendering paths before optimizing. Benchmark plain clip, Lumetri only, Blur only, each of Wave/Wiggle/Glow alone, Lumetri+Blur, Lumetri+Blur+Wave, Lumetri+Blur+Wiggle, and the named Glow transition with/without competing effects. For transitions test overlap lengths and both frames. Use the current demo where appropriate plus deterministic exported fixtures.

Measure real distinct presented images/s, deadline misses, rAF p95 and long tasks, render and effect time, decoding, allocations/copies, cache revision churn, CPU fallback and actual GPU-path status. Ensure compiler/profiler work is not running during the final browser benchmark. Use normal Chrome and identical hardware/preview conditions. Do not confuse UI FPS with actual sequence presentation FPS.

Specifically determine whether one unsupported effect forces an entire otherwise GPU-eligible chain to CPU, whether Wave requires costly full-image resampling, whether Wiggle causes repeated geometry or cache invalidation, and whether Glow causes expensive dual-source rendering, recomputation or intermediate allocations. These are hypotheses to test, not assumptions.

Implement the safest high-impact solution(s): GPU implementation when accurate, more efficient CPU kernels, correct mixed CPU/GPU segmentation, resource reuse, bounded caching, and/or avoiding redundant processing. Respect effect order, masks, alpha, HDR/color math, time-varying values and exact final export. Keep CPU reference fidelity and fallback. A draft preview mode can be used deliberately, but must not silently reduce paused/final/export quality. **Do not declare a feature hardware accelerated based only on API support or a pipeline's existence.**

**Workstream isolation:** Do not combine a major effects-renderer rewrite, a new animation data model and new UI geometry in one enormous commit. Implement and verify engine optimizations separately from UI changes. If time/usage is constrained, preserve checkpoints and honestly mark unresolved stutter.

## 11. Consolidate the THREE competing top-bar systems

Current UI puts traditional menu commands (File/Edit/Clip/Sequence/Markers/Graphics and Titles/View/Window/Help), a second primary mode bar (Project Hub/Edit/Color/Export), and additional workspace/project controls (Home icon, Clean Editing dropdown, status capsule). They compete visually; Home and Project Hub duplicate navigation.

Preserve **all existing professional menu commands** and shortcuts, but create **one understandable navigation hierarchy**:

- One app identity/entry point on the left; no redundant Home action if Project Hub already exists.
- ONE clear primary workspace navigation: Project Hub, Edit, Color, Export (and other working specialized modes in the appropriate workspace selector), with a discoverable Edit-layout menu for Clean Editing/Classic/other layouts.
- Project name, truthful unsaved/storage status, settings, theme and functional fullscreen grouped unobtrusively on the right.
- Traditional File/Edit/Clip/Sequence/Markers/Graphics/View/Window/Help remain accessible, potentially as a compact Menu dropdown with cascading categories and a selectable classic visible-menu presentation where useful. Do not destroy familiar pro commands or require a long extra navigation row to find them.
- Responsive behavior: at narrower widths condense labels and move advanced items to one menu, instead of overlapping mode buttons, project identity or controls.
- Optional command search/palette should reuse the command registry, never be a duplicate command implementation.

Visually keep the header **flat, compact, subtle**. Do not create a second breadcrumb/navigation toolbar unless it proves necessary and is visually modest. Do not simply rename three competing systems; reduce their redundancy and ensure functional clarity. Compare screenshots and interaction tests before declaring it improved. Keep saved workspace names discoverable without a permanently competing high-level selector.

## 12. App Settings / Preferences must be visible and real

The repository already implements Settings in `crates/ui-egui/src/panels/settings.rs` and registered Edit → Preferences commands, including General, Appearance, Playback, Media Cache, Memory, Audio and other categories.

Expose the **existing** Preferences dialog through a clear gear/app menu, preserving traditional menu navigation and shortcuts (`Ctrl+,` or platform-appropriate equivalent). Improve category organization and helpful explanations as needed. Do not build a disconnected second settings store. Respect native-only vs browser-capability limitations; disabled controls should explain why.

Consider a playback/performance diagnostics section presenting *actual* measured available stats (sequence FPS, deadline drops, WebGPU/CPU fallback, decoder preference/status, buffer usage), hidden unless requested. Clearly distinguish a configured preference from evidence of actual hardware usage.

Review UI theme controls, icon legibility, responsive sizing, keyboard settings/shortcuts and save/reset behavior.

## 13. Distinguish Project Assets from Browse Files (Media Browser)

The **Project** panel contains already-imported clips, sequences, graphics, bins and project structure. The **Media Browser** browses external/host-accessible directories before import. Both are valuable, but their difference is currently unclear.

Preserve both functions and their existing command IDs; make the names/introductory messages/tooltips clearer, e.g. Project Assets and Browse Files. Improve left-panel interactions without making them card-heavy.

Media Browser requirements: clickable breadcrumbs, visible and labeled Back/Forward/Up affordances, keyboard navigation, intuitive folder double-click, consistent navigation history, visible current location, editable path when necessary, clear Import and Open in Source operations, correct error states, and correct handling of inaccessible/nonexistent paths. The up arrow is already present in existing code: improve discoverability rather than introducing two unrelated navigation mechanisms.

Browser filesystem permissions are not desktop filesystem access. Do not render empty Local Drives/Network groups as though the browser can inspect the machine. Provide truthful Open Files/Link Folder guidance only if the current host supports it. Preserve drag/drop from Media Browser to Project/Timeline, import media, favorites, filters, list/grid and thumbnail behavior. Confirm folders, recent directories and file selections remain functional on native hosts too.

## 14. Reliable Clean Editing workspace and docking

Investigate why screenshots of ostensibly the same Clean Editing workspace show different panels/tabs. Distinguish user-selected tabs, automatically contextual panes, persisted saved layouts, and actual unintended layout drift. Do not overwrite intentional saved customizations.

Define/verify sensible **factory Clean Editing**:

- Project Assets tab to left, with Browse Files and Effects readily accessible.
- Large Program monitor central by default; Source opens contextually and preserves independent transport/timecode/in/out/pinning and resizable split.
- Right dock holds Properties / Effect Controls; their role and switching are clear.
- Multi-track timeline stretches below with normal professional controls.
- Animation is **collapsed** by default; opening/closing drawers does not break docking.
- No redundant tool palettes or advanced experimental buttons.

Provide an understandable distinction between **Preview Factory Layout**, **Restore Saved Layout**, **Reset to Default**, and **Save Layout As**. Layout resets must not affect clips, edits or keys. Verify restoring layouts after workspace switching, closing/reopening contextual Source, page reload, floating/docking Graph Editor, theme changes, and responsive resize.

## 15. Light/dark polish without abandoning established styling

Light mode is promising but currently shows some weak boundaries, low-contrast controls, inconsistent text hierarchy, busy selection colors and mismatched panel densities. Review actual Chrome screenshots in BOTH themes.

Fix inconsistent borders, control alignment, disabled vs active state, hover/focus cues, cramped text, icon style, toolbar grouping, animation row colors and readability. Keep the viewer neutral for color work. Avoid saturated blue blocks, heavy focus outlines across whole panels, huge rounded cards, extra header height or arbitrary visual effects. Ensure animation drawers, graph backgrounds, keyframe handles and inspector affordances are recognizable in both themes; do not rely solely on color to communicate interpolation or selected state.

## 16. Turn Project Hub into a useful entry point

Existing Hub prioritizes oversized sequence presets and then leaves much of the page empty. Reorganize without removing existing project creation and sequence presets:

- Continue Editing / recent real projects and honest saved state.
- Open Project; New Project.
- Start from a chosen sequence template/preset.
- Load Demo.
- Linked local folders or accessible documents when the host supports them.

Use real project data and genuine actions. Do not fabricate recent thumbnails, cloud sync results, folder permissions or user documents. Provide informative empty states and good keyboard navigation. Keep it compact, clean and recognizably FilmCraft—not a marketing dashboard. A current project should not require repeatedly going through Hub to continue editing.

## 17. Storage & Collaboration: truthful, compact, testable

The title/status capsule currently opens an oversized, visually inconsistent panel mixing local storage, Google Drive folder ID, cloud sync, collaboration room and supposed shareable URL. Redesign as an appropriately sized, well-organized **Storage & Collaboration** dialog with clearly identified sections and states:

- Unsaved/in-memory versus saved-local project, file location, save and save-as.
- Linked local/synced folder versus authenticated cloud account and verified actual sync status.
- Live collaboration connection, participants/room when actually available, reconnect/error state, and permissions.
- Actionable explanatory errors and supported host capability indicators.

Audit actual underlying functionality BEFORE changing labels. Preserve working collaboration/Drive behavior; do not implement fake providers, fake synced indicators or nonfunctional share buttons. A URL beginning `127.0.0.1`/`localhost` is NOT externally shareable—never label it as a universal public collaboration link. Distinguish a room token, local preview link and genuinely reachable collaboration URL. Do not treat entering a Google Drive folder ID as proof of authenticated sync. Test local save/reload, linked folder errors, relevant cloud statuses, collaboration and disconnect/reconnect; document unsupported scenarios rather than claiming success.

## 18. Fix the web application's fullscreen button

The header's existing `egui::ViewportCommand::Fullscreen` does not work in the user's ordinary web environment. Investigate actual web host plumbing and implement an appropriate browser Fullscreen API path from a genuine user gesture, with fallback behavior when unsupported or denied. Native desktop fullscreen must remain functional.

Handle current fullscreen state, request result, Escape/external exit, button state synchronization, canvas resizing, DPI/viewer geometry, and permitted browser restrictions. Test normal Chrome without flags and ensure keyboard/mouse interaction survives entering/exiting. Clearly distinguish fullscreen of the *whole editor* from maximizing just a Program panel; preserve existing monitor behavior.

## 19. Other actual usability and reliability issues to inspect

After the priority features and tests, evaluate and address defensible improvements in:

- Timeline track header density/alignment, V1/V2 and A1/A2 patch/target distinction, lock/mute/solo/visibility and readable hover states.
- Panel resizing and handles, layout at narrow browser widths and high-DPI screens.
- Effect search, effect application, ordering, presets, context menus, masks and enable/bypass controls.
- Keyboard shortcuts, command discovery, context-aware menu behavior and no interception while typing.
- Import progress/error messaging; media thumbs, waveform generation and browsing responsiveness.
- Empty states, accessibility labels/tooltips, visible focus and predictable scroll regions.
- Project save/unsaved indicator and recent-project state.
- Source/Program independently functional interaction and snapshot correctness.
- Responsive parameter dragging and preview updates, including after-effects-style motion work.
- Optional honest GPU/CPU diagnostics on effect/transition headers or performance pane.

Do not spend the entire implementation period on low-value cosmetic changes. Keep serious editing functionality and correctness ahead of polish.

## 20. Future After Effects-inspired capabilities: plan, don't pretend they're in V7

The V7 animation foundation should make future work possible without obliging you to implement it all now:

- Controllers/null-type non-rendering elements and optional parent/child motion relationships.
- Property-link/pick-whip UI.
- Deterministic, sandboxed visual expressions (Loop, Ping-Pong, Follow, Delay, Wiggle) before any general scripting engine.
- Spatial motion paths and improved tangent control.
- Reusable easing presets and richer value/speed graph editing.
- Per-character text/shape animation, advanced mask/path animation.
- Selected exposed properties for nested sequence instances (reusable titles/graphics).
- Effect groups and reusable animation templates.

These are **future roadmap items**, not permissions to add placeholder UI or insecure JavaScript evaluation now. They need explicit engine, serialization, project compatibility, export and performance design. Your goal is the solid Phase 1–3 foundation, not an unfinished full clone of After Effects.

## 21. Execution order, phases and usage discipline

This assignment is deliberately broad, but **all categories above remain in scope** even if implementation takes several sessions. Do not silently drop the older UX/performance requests in favor of V7, or vice versa. Track completion status honestly.

1. **Phase 0 — Audit and baseline:** Verify Git state and latest branch; inspect HTML and plan; inspect actual UI, effects, keyboard, animation/timebase code; screenshot both themes; run relevant baseline tests and matched effect benchmarks. Produce a prioritized issue/implementation checklist, not a research-only report.
2. **Phase 1 — Low-risk correctness and performance:** Fix reproducible interface defects (such as fullscreen and discovered navigation/scroll issues), and investigate/implement a defensible Wave/Wiggle/Glow bottleneck fix or isolate it with actionable evidence. Keep renderer work separately committed.
3. **Phase 2 — V7 disclosure MVP:** Per-clip expansion, collision-safe drawers, effect/property hierarchy, appropriate view states and filters, selected parameter/inspector synchronization, current key diamonds. Production-quality defaults and stable timeline geometry.
4. **Phase 3 — Fully functional inline animation:** Real numeric edits, enable/keyframe changes, retiming/multiselect, interpolation context menu, frame mapping, undo and project persistence using existing commands. Test tricky trim/split/reverse/speed cases.
5. **Phase 4 — Focus + graph:** Focus Clip, detachable dockable Graph Editor, value/velocity modes and existing influence handles. Unified selection, proper graph evaluation and performance.
6. **Phase 5 — Professional UX integration:** Compact Effect Controls, consolidated top-bar hierarchy, discoverable settings, clarified Project/Media Browser, deterministic layouts, light/dark, Hub and Storage/Collaboration improvements; make all reachable and useful from the actual application.
7. **Phase 6 — Final verification and handoff:** Real interaction test suite, production WASM and available desktop validation, effects benchmarks, screenshots, documentation, known limitations, clean pushed implementation branch.

The exact internal order may change where dependencies demand it. In particular, lightweight header fixes might precede major animation UI, and keyframe views may be implemented in cohesive commits. BUT keep both effects performance and UX requirements visible throughout the project. Prefer vertical end-to-end working slices over enormous untested integration.

If there are insufficient resources to complete all phases in one session, **do not present partial work as full completion**. Implement as much verifiable functionality as possible, commit/push completed pieces and provide a precise next-session handoff. Stop starting large risky changes when remaining usage is low. Preserve a clean standard browser build as a fallback at all times. Never promise background work.

## 22. Regression, browser/native testing, and acceptance

Use the project's real test tools and exact pinned toolchain. Extend Rust tests, UI interaction tests and Chrome automation where necessary; browser performance tests belong to actual optimized builds. Use `apps/filmcraft-web/tests/studio-review.mjs` for controlled playback where appropriate. Don't benchmark while compiling or while CPU profiling. Maintain fixture hashes and matched render scale/backend/browser flags. Clearly label skipped/unverified test coverage.

**Animation acceptance:**

- Collapsed ordinary Clean Editing looks and behaves materially like baseline FilmCraft.
- Expand two clips on one track and another on a different track: no drawn overlap, misaligned keyframes or destructive track height collisions.
- Short clip focus, selected/modified/animated filters, dock/float Graph Editor and right inspector work together.
- Edit static property without enabling animation, then intentionally enable animation, add/move/delete keys; undo/redo and real frame rendering agree.
- Drag several keys, change interpolation, show X/Y and switch to speed graph: underlying data and Program/export agree.
- At different timeline zoom/scroll positions, keyframe X coordinates follow the exact main ruler.
- Trim, move, slip, split, reverse, varied speed, nested clips, transition overlap and audio-domain tests preserve actual rendering/time semantics.
- Keys/effects persist correctly after save/reopen; expand/collapse-only actions don't dirty project.
- Reorder/delete/duplicate effects or clips without stale selection or wrongly attached view state.
- Arrow/keyboard access works and property shortcuts don't fire in inputs or conflict with existing edit tools.
- Large projects with many animated clips remain scrollable and responsive; animation UI does not regress standard playback.

**Original UX/performance acceptance:**

- Quantified before/after Wave/Wiggle/Glow results or precisely documented unresolved bottleneck; no invented acceleration claim; Blur+Lumetri baseline preserved.
- Effect Controls compact by default, no intrusive Fusion Nodes button, full professional operations retained.
- Home/Hub duplication resolved; 3 competing header systems harmonized, no lost File/Edit/etc commands, responsive narrow layout.
- Existing Preferences visible and usable; no duplicate settings system.
- Project Assets and Browse Files clearly differentiated, breadcrumbs/back/up functional and browser capabilities truthfully described.
- Clean Editing defaults and custom layouts consistently restored; optional Source behavior intact.
- Light/dark theme contrast and subtle FilmCraft identity preserved.
- Project Hub usable, real recent/linked data, empty states and presets preserved.
- Storage & Collaboration dialog truthful about save, auth, linked directory, sync, real room and shareable URL; working providers preserved.
- Browser fullscreen enters/exits successfully where allowed; graceful failure otherwise; native desktop remains compatible.
- Project import, drag/drop, timeline controls, effects, source/monitor operations and export maintain focused regression coverage.

Run available formatting, Clippy, architecture/layers, asset attribution, WASM workspace and optimized web packaging; run relevant GPU parity, export and UI suites, with actual results. Test in normal Chrome with light/dark at 1656×894 and narrower widths. Test desktop on available hardware; never claim other OS/hardware tested when not. Verify project files survive both old and new builds where appropriate. Do not omit tests simply because an HTML demo appears to work.

## 23. Deliverables and final report

Push verified commits to `filmcraft-v7-animation-and-ux`, keeping original branches untouched. Include:

1. Real Rust/egui implementation, not a static HTML embedding or replacement application.
2. An implementation checklist/status document mapping **every section of this prompt** and both attachments to `completed / partial / deferred`, with reason and associated commits. Never lose tasks between sessions.
3. Brief architecture notes covering drawer geometry, shared animation state/commands, exact time mapping, selected key identity, curve semantics, view-state persistence, and compatibility.
4. Correctness and performance report: precise matched Wave/Wiggle/Glow results, pre/post browser responsiveness where measured, standard playback regression comparison, and any native performance evidence.
5. Real before/after screenshots for ordinary Clean Editing, 2+ expanded clips, Focus Clip, Graph Editor value/speed, dark/light, Hub, Preferences, Storage dialog and narrow mode as completed.
6. Exact test/build commands, passing/failing/skipped tests, baseline hashes, true GPU/CPU paths, and any unverified platform or scenario.
7. Concise handoff explaining what was genuinely finished, remaining issues, recommended order and reproducer steps for the next Codex session.

Report categories separately: (a) confirmed bugs fixed, (b) measured performance improvements, (c) actual animation functionality, (d) usability/layout refinement, (e) deferred After Effects-inspired features, (f) known defects and verification gaps. Do not claim full V7 implementation from a mocked graph, functioning controls with no engine command, synthetic benchmarks, or a standalone prototype.

**Hard boundaries:** Do not silently merge/deploy, force threaded WASM, delete user work, reduce export accuracy, break existing projects, remove essential menu/editing operations, fabricate cloud features or GPU claims, or sacrifice verified playback improvements. Changes should be reversible, tested and attributable to commits.

**Final product objective:** FilmCraft remains a familiar, professional, high-performance multi-clip editor but gains a deeply integrated, progressively disclosed, After Effects-inspired animation workflow. The V7 HTML and Markdown plan define the direction; the real FilmCraft implementation defines correctness and subtle visual fidelity. Produce functioning, testable code and preserve the complete scope of this request across checkpoints.
