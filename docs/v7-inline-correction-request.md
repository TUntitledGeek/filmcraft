# FilmCraft V7 — Correct and Complete the Inline Animation Workspace

Continue development on `filmcraft-v7-animation-and-ux` from the latest verified V7.4 checkpoint.

**This session's highest priority is fixing the V7 animation UI architecture to match the original HTML prototype and design plan.**

The existing V7.4 animation drawer is functional but structurally wrong. It resembles a miniature inspector inserted into the timeline, with a fixed 112-point height, an independent internal scrollbar, and nearly every property automatically displayed.

The resulting interface is cramped, confusing and visually disconnected from its owning clip. The associated clip can also disappear partly above the visible timeline when expanded.

I want these problems corrected at the architectural level, not hidden through cosmetic adjustments.

## Required references

Read:

- `FilmCraft_Codex_V7_Master_Implementation_Prompt.md`
- `FilmCraft_V7_Animation_Workspace_Plan.md`
- `filmcraft_v7_animation_workspace.html`
- `docs/v7-implementation-status.md`
- `docs/v7-animation-architecture.md`

Study the original HTML's hierarchy, disclosure interactions, effect rows, property rows, animation toolbar and relationship between ordinary clips and animation lanes.

Use the HTML as the strongly preferred workflow reference while retaining the actual FilmCraft Unified visual style.

Do not blindly copy the HTML's mock code or replace Rust/egui.

## Phase 1 — Fix the timeline layout first

Replace the existing fixed-height drawer with a proper, dynamically measured hierarchy.

Animation rows must belong to the owning clip and track, rather than functioning like an embedded inspector.

The ordinary timeline must retain its compact collapsed appearance.

**Critical requirements:**

1. Opening animation must keep the owning clip visible immediately above its animation hierarchy.
2. The video/audio divider must not unexpectedly move when animation is expanded.
3. No second independent vertical scrollbar inside ordinary expanded clip rows.
4. Expanded track groups must integrate with the timeline's existing vertical scrolling and scroll extents.
5. Expanding one clip must never conceal another clip or overlap its hit-testing area.
6. Opening a hierarchy must not automatically display every effect parameter.
7. The geometry must respond to exactly which groups and properties are expanded.
8. At short timeline heights, anchor the expanded clip visibly, scroll naturally and allow an optional Focus Clip presentation when necessary.
9. Track targeting, audio waveforms, clip dragging, trimming, scrolling and transitions must remain functional.
10. Keep existing ruler alignment, zoom and horizontal scrolling shared between clips and keyframes.

Do not replace the video/audio split with a different layout without explicit justification.

## Phase 2 — Implement the actual V7 hierarchy

Add a small disclosure chevron directly on each applicable clip.

The normal interaction must be:

Clip → Effect Group → Property → Keyframes.

When a clip first expands, display only its compact effect-group headers.

For example:

Motion — collapsed  
Opacity — collapsed  
Lumetri Color — collapsed  
Gaussian Blur — collapsed

Clicking Motion reveals its relevant Position, Scale, Rotation and other actual properties.

Clicking Opacity reveals its Opacity property.

Expanding Lumetri reveals its parameters, grouped appropriately.

Each effect and property must be independently collapsible.

Multiple expanded clips should be supported when the geometry is ready, without row collisions or incompatible disclosure state.

Do not show raw Rust debug values such as `Vec2(...)` and `Bool(true)`.

Use correctly typed controls, formatted numbers, units, labels and ranges derived from real parameter definitions.

Avoid a special separate Opacity row. The existing working Opacity editor should be integrated into the normal reusable property-row component.

Resolve stable effect-instance identity before attaching persistent disclosure state to duplicate or reordered effects.

## Phase 3 — Real animation interaction

Reuse canonical engine commands and existing Effect Controls behavior.

Implement a reusable editable property row supporting the applicable parameter types.

Support actual keyframes, addition/deletion, navigation, timeline positioning, selection, retiming, supported interpolation and undo/redo.

Keep static values distinct from animated properties. Editing an unanimated parameter should not automatically create keyframes.

Respect exact source/effect-time versus sequence-time mapping and correctly handle trims, splits, reverse, speed changes and frame holds.

If a mapping is not yet supported safely, communicate that explicitly without corrupting animation.

The right inspector and inline controls must remain synchronized.

Do not create a separate animation state or duplicate renderer interpolation logic.

## Phase 4 — Add V7 navigation and precision controls

Once the basic hierarchy works properly, continue with:

- Animated / Modified / All / Selected property filters.
- Compact animation toolbar.
- Focus Clip mode.
- Dockable/floating Graph Editor.
- Value and Speed/Velocity graphs.
- Existing Bezier influence controls.
- Convenient animation shortcuts without conflicts.

These features remain required, but should not be built on top of the incorrect fixed-height layout.

## Phase 5 — Continue the original UX roadmap

The complete original V7 master prompt remains required, including the professional Effect Controls redesign, header consolidation, layout improvements, Project Hub, browsing/navigation, Storage & Collaboration and theme consistency.

Do not abandon those requirements.

Wave Warp combined with Lumetri and Blur remains a severe unresolved performance issue. Preserve that problem in the ongoing ledger, but do not let unrelated rendering investigations consume this interface-focused session.

Preserve the previous verified GPU Wiggle optimization and all prior performance improvements.

## Mandatory visual verification

Before considering the corrected animation layout complete, launch the actual Rust/egui web application in Chrome.

Use a representative multi-track demo with visible video and audio, not only a simple isolated imported clip.

Capture screenshots in light and dark modes.

Verify all of the following:

- A normal collapsed timeline.
- One clip expanded showing only its effect-group headers.
- Motion expanded with readable properties.
- Opacity expanded with visible keyframes.
- Multiple effects selectively expanded.
- Another clip expanded.
- A short timeline panel with video/audio tracks still accessible.
- A narrower desktop viewport.

Compare the resulting real application directly with the original V7 HTML concept.

**The first accepted milestone is a visibly correct expandable hierarchy, not merely another successful Opacity unit test.**

Test actual editing, hit testing, undo/redo, project persistence, and animation consistency.

Use verified, logical commits and push each stable milestone.

Do not merge or deploy.

## Final objective

FilmCraft should remain a familiar professional video editor, but any clip should be able to reveal a progressively expandable animation hierarchy directly within the normal timeline.

The original prototype's overall structural interaction model must be closely followed.

**Correct the current implementation first, then expand its functionality. Do not repeat the V7.4 design mistake by stuffing more features into a fixed-height scrolling inspector.**