# FilmCraft V7 implementation status

Branch: `filmcraft-v7-animation-and-ux`, created from verified local/remote
`a7c98a5ee7d34eda046930b12b291cb758f069b8` on 29 October 2026. The starting tree was clean.
No protected branch, deployment or release is authorized by this task.

## References and authority

The user explicitly authorized following `FilmCraft_Codex_V7_Master_Implementation_Prompt.md`.
The matching HTML and Animation Workspace Plan remain design references. The entire prototype's
styles, markup and four scripts have been inspected from source, with vector coordinates omitted
from the inspection transcript. No prototype icons/assets, JavaScript runtime, sample keyframe
store or illustrative interpolation are being incorporated into FilmCraft.

Interactive prototype inspection was blocked by Computer Use's inability to identify Chrome's
current URL. The user then explicitly approved source-based investigation. This is not recorded
as a successful interactive prototype test. Real-app verification uses the project's test tools.

The prototype uses filenames and floating 30 fps percentages as identities/time, a separate
history stack, illustrative curves and mock cloud/export controls. Production must instead use
the existing project/command model, exact source/sequence mappings, dock persistence and tokens.

## Complete scope ledger

Section numbers refer to the master prompt. Pending means in scope, not silently dropped.

| Section | Requirement | Status / next verification |
| --- | --- | --- |
| 1 | Preserve prior engineering and clean branch | Base verified; new branch created; prior optimizations retained |
| 2 | Subtle Unified visual refinement | Pending matched real-app light/dark/narrow screenshots |
| 3 | Collision-safe inline hierarchy, Focus, Graph | Pending; reuse canonical effects/keys and shared ruler |
| 4 | Animated/Modified/All/Selected, search, shortcuts | Pending; audit registry before assigning keys |
| 5 | Real numeric/keyframe editing and undo | Pending; no second animation store |
| 6 | Canonical value/velocity curves and influence | Pending; reuse Effect Controls graph; derivative view read-only |
| 7 | Exact trim/split/slip/reverse/speed/nested domains | Pending mapping regression matrix |
| 8 | Geometry, focus, virtualization, view persistence | Pending; collapsed timeline must remain unchanged |
| 9 | Compact Effect Controls, optional lane, no Fusion CTA | Pending; retain operations and advanced node access |
| 10 | Wave/Wiggle/Glow performance | Pending controlled baseline and isolated renderer change |
| 11 | One header hierarchy with all menu commands | Pending; avoid duplicate Home/Hub and overlapping controls |
| 12 | Visible existing Preferences and diagnostics | Pending; route existing command/settings store |
| 13 | Project Assets / Browse Files clarity | Pending; preserve IDs, navigation and truthful web capabilities |
| 14 | Factory/saved/reset layout distinction | Pending; do not overwrite customized docks |
| 15 | Light/dark polish and precision controls | Pending actual screenshots and keyboard checks |
| 16 | Useful Hub with genuine project/recent data | Pending; no fake recent projects or thumbnails |
| 17 | Truthful Storage & Collaboration | Pending save/auth/sync/room/link capability audit |
| 18 | Browser fullscreen, native compatibility | Confirmed desktop-only viewport route; implement browser host API |
| 19 | Other editing/usability reliability | Pending focused regressions; retain track semantics and shortcuts |
| 20 | Future AE authoring | Deferred by design: parenting, expressions, spatial tangents, rich templates |
| 21 | Phases, usage discipline, checkpoints | Active; keep standard build and verified checkpoints recoverable |
| 22 | Real tests, builds, benchmarks | Pending V7 baseline and per-slice verification |
| 23 | Code, complete status, architecture, evidence, handoff | This ledger started; final evidence pending |

## Implementation boundaries found in source

- `EffectInstance` has no general stable UID; the engine selects by effect type and occurrence.
  A V7 identity strategy must explicitly handle same-type reorder/removal/undo rather than
  assuming the prototype's filenames or the inspector's current array-index strings are stable.
- Existing Effect Controls owns key selection and graph interactions, and uses source time.
  Reuse/refactor that machinery; do not transplant HTML graph math.
- Browser media I/O is event-loop-local. The archived threading experiment remains disabled.
- The current browser fullscreen control emits a desktop viewport command. Host hooks offer
  an existing boundary for a real browser Fullscreen API implementation.

## Verification and evidence

No V7 runtime change has been verified yet. Existing performance evidence remains in
`performance-before-after.md` and `verification/upstream-performance.json`. Record tests,
failures, skips, screenshots and effect measurements here as each slice completes.
