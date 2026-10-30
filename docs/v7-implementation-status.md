# FilmCraft V7 implementation status

## V7.5 verified correction: progressive inline hierarchy

The user's [correction request](v7-inline-correction-request.md) supersedes V7.4's fixed-height layout.
V7.4 remains a recoverable verified code checkpoint, not an accepted final animation design.
Verified source is pushed as `7ad34e1b`; regression/API compatibility repairs are `be1e88d9`.
The correction replaces the embedded inspector with measured clip/effect/property/keyframe rows,
direct clip chevrons, shared track scrolling and explicit owner anchoring. No merge or deployment.

**Completed:** newly opened clips show compact effect headers, each group/property folds
independently, Lumetri uses actual schema subgroups, and multiple clips on the same/different
tracks allocate disjoint measured rows. The existing video/audio divider stays fixed. Opening
anchors the owning ordinary clip; resizing preserves a previously visible owner. Intentional
track scrolling reaches lower rows naturally. There is no internal scrollbar. Closing all
disclosures restores compact geometry and saved scroll positions without editing the project.

Reusable typed editors replace the special Opacity row. Actual Motion Position, static Opacity,
animated edits, add/remove keys, undo/redo and grouped Lumetri Exposure were exercised in Chrome.
Key positions retain the main ruler's zoom/scroll transform. Previous/next, stopwatch and key
click-to-seek use existing commands. No alternate animation values/history/interpolator exist.
Real animated parameter data survives save/new/open through canonical project serialization.

**Verification:** 231 UI library tests and 95 focused integration tests passed (326 total):
11 inline hierarchy, 10 Clean Editing, 8 Escape/drag, 17 keyframe, 8 menu, 12 docking,
25 Studio Pro, 2 track-height and 2 fullscreen-host tests. Optimized standard WASM packaging,
49-crate WASM compatibility, UI all-target Clippy with `-D warnings`, formatting and layers passed.
Attribution passed for 198 assets, including the 18 new original verification captures.
The full native workspace suite remains incomplete; the correction does not claim a fresh full
workspace Clippy pass. Earlier checkpoint results remain archived below.

Real Chrome 155 / WebGPU / ordinary single-thread build passed the multi-track acceptance run
with zero recorded browser errors and 24 captures. Required collapsed/header/Motion/Opacity-key/
selective-effect/another-clip/short/narrow states were captured in both themes. Viewports were
1656×894 and 1280×894. Expanded clip trim/move undo and targeting/patch controls passed; the
existing Source double-click/drop, timeline trim/split/move and undo/redo checks also passed.
Screenshot review confirms progressive hierarchy rather than an embedded scrolling inspector.
Some lower Motion/effect rows require the shared track scroll at this split, as intended.

Evidence: [machine-readable results](verification/v7-inline-hierarchy.json),
[18 attributed screenshots and comparison notes](screenshots/v7-hierarchy/README.md),
[implemented architecture](v7-animation-architecture.md). Production WASM is 43,041,231 bytes,
SHA-256 `c7383262d46e02befe71b8fb7070a420be3b6476d9ae370ec35fc7f246cf71c1`.
These are functional checks; their one-second playback samples are not performance benchmarks.

Existing regression setup issues were corrected rather than hidden: missing Effect Controls
was reproduced on unchanged V7.4, and those tests now request Classic Editing explicitly;
the mask test uses Motion's twirl. Legacy Hub automation IDs remain aliases. Chrome's test
gestures now move before wheeling, materialize implicit targeting defaults and separate
double-click pairs to avoid accidental triple-click classification.

**Still required:** stable model IDs and persistent duplicate-effect disclosure, broader time
domains, inline key selection/retiming/interpolation, filters, toolbar, Graph/Focus and all
original UX work. Duplicate instances are visibly closed; unique effect types are re-resolved
for each action, not identified by filename or stored index. Disclosure is transient.
Inline edits/key positioning are limited to speed 1, forward, non-held clips. Time Remapping
and text/curve/path summaries are read-only inline; specialized existing editors remain.
Reverse/speed/hold safety is covered by a disabled-control regression. Large-project planning
and other browser/OS interaction remain unmeasured. GPU Wiggle/render code is unchanged;
Wave Warp + Lumetri + Blur remains a severe unresolved issue. The full master and references
remain binding, with every unfinished requirement retained in the ledger below.

## V7.4 archived checkpoint: first functional, structurally rejected slice

This section supersedes the earlier checkpoint's statement that no inline UI exists.
The entire master prompt and both references remain required; all untouched ledger entries remain pending.

- Select a clip and use Clip > Show/Hide Clip Animation, or its timeline context menu. Close it independently with the drawer's close control.
- One actual clip drawer reserves track height, shares the main ruler transform, has its own property scrolling and excludes its visible area from clip gestures/new-track drops. Collapsing restores the compact geometry without editing the project.
- Actual ordered effects and canonical property values are displayed. The unique intrinsic Opacity control edits static values without implicit animation and adds/removes real keys through existing engine commands. Undo/redo is shared with the editor. Two real keyframes were created in Chrome.
- State is transient and scoped to clip ID, sequence and workspace. No filename or effect-index identity is stored. Project replacement and clip deletion clear stale state.
- Key positions/editing are limited to normal-speed, non-reversed, non-held clips. Other mappings explicitly disable the quick editor. Other properties are read-only; there is no per-effect disclosure state.
- Native: 230 existing UI unit tests plus 2 new geometry/lifecycle tests passed. Focused all-target UI Clippy, formatting, optimized standard WASM, layers and attribution passed. Real Chrome tested expansion, non-destructive view state, hit exclusion, static editing, key creation, undo/redo and close.
- Screenshots: [dark](screenshots/v7-inline/inline-animation-dark.png), [light](screenshots/v7-inline/inline-animation-light.png). Evidence: [v7-inline-slice.json](verification/v7-inline-slice.json). Full native workspace/other OS coverage remains incomplete.

Known limitation: in short timeline docks the expanded drawer can push the clip header partly above the visible video region; vertical track scrolling or enlarging the timeline is needed. Adaptive drawer height remains pending.

Next: add audio/narrow/scroll regression cases, stable effect identity before persistent per-effect state, then incremental property editors and key interactions. Multiple simultaneous drawers, filters, group disclosure, nonlinear/held/reverse domains, saved drawer state, Graph and Focus remain pending. All original UX and Wave/Glow requirements remain pending as recorded below. No merge or deployment.

## Earlier checkpoint: verified partial implementation, 29 October 2026

Branch: `filmcraft-v7-animation-and-ux`, based on verified local/remote
`a7c98a5ee7d34eda046930b12b291cb758f069b8` (`performance-upstream-integration`).
The starting tree was clean. No protected branch was changed; no merge or deployment occurred.

The entire **FilmCraft_Codex_V7_Master_Implementation_Prompt.md** remains the ongoing task.
Both references, **FilmCraft_V7_Animation_Workspace_Plan.md** and
**filmcraft_v7_animation_workspace.html**, remain applicable. This checkpoint completes a
small rendering/host slice, not V7 as a whole. The user's checkpoint-completion instruction
paused new feature work until the verified changes were committed and pushed.

## Preserved commits and completed features

- `5ae4d07a` — V7.0: verified base and complete scope ledger.
- `259f7d52` — V7.1: real browser fullscreen, visible existing Preferences, host regression tests.
- `a7bdb7b6` — V7.2: GPU-eligible Wiggle through the existing affine resampler, with CPU/GPU parity.
- The evidence/status commit following these preserves measurements, screenshots and next steps.

**Rendering:** Wiggle's evaluated transform is shared with its CPU reference. Eligible Wiggle
chains now use the existing GPU resampler instead of forcing the otherwise eligible chain to
CPU. Effect order, alpha/HDR behavior and the existing minification/mip CPU fallback remain.
No shader approximation, new dependency, threaded build or project schema was introduced.

**Host/UI:** The header fullscreen button uses the browser Fullscreen API through HostHooks;
actual active state and asynchronous failure are reported. Native viewport fullscreen remains
available when no host hook exists. The new gear opens the existing General Preferences dialog
and existing settings store. The browser harness also fixes stale source IDs when combined
editing tests switch from the imported project to the demo.

**Animation:** No new inline animation UI, Focus Clip or detached Graph Editor is implemented.
No project/keyframe data model was replaced. The source audit is recorded in
[v7-animation-architecture.md](v7-animation-architecture.md); its future integration notes are
not claims of implemented functionality.

## Actual Chrome measurements

Three successful fresh-profile 12-second runs per row/build; figures are **median distinct
presented sequence images/s [min–max]**, not UI FPS or frames meeting their deadline.

| Condition | Frozen base, contemporary controls | Verified checkpoint |
| --- | ---: | ---: |
| Plain imported clip | 24.96 [24.66–25.37] | 24.94 [24.26–25.43] |
| Lumetri + Blur | 25.12 [24.86–25.47] | 24.99 [24.35–25.00] |
| Wiggle | 19.06 [18.99–19.27] | 25.26 [24.39–25.52] |
| Lumetri + Blur + Wiggle | 6.43 [0.47–7.91] | 24.92 [24.71–25.26] |

Wiggle improves about 1.33x; the combined chain improves about 3.87x by median. Lumetri+Blur's
median differs by approximately -0.5%, with overlapping ranges: no material regression observed
in this experiment. These are local Windows/Chrome results, not a universal smoothness guarantee.
GPU effect-layer submissions were observed during the new Wiggle runs, alongside native GPU
parity tests. `cpuEffectLayers == 0` alone is **not** proof of acceleration: the counter misses
whole-chain CPU results emitted as FramePlan::Image.

The original eight-condition baseline is retained in
[verification/v7-phase1.json](verification/v7-phase1.json), together with every reported sample,
frame-deadline counts, rAF p95, long tasks, render/decode cost, GPU submissions, upload/cache data,
fixture/build hashes and hardware/browser conditions. Original plain and Lumetri+Blur medians
were about 29.5 images/s; later browser pacing changed from roughly 120 to 30 UI updates/s.
Repeating the frozen base under the later conditions prevents attributing that system-wide
change to this patch. Its cause was not established. Strict frame-deadline misses still occur;
no smooth 60 FPS claim is made.

Conditions: ordinary Chrome 155.0.8059.86, Intel Ultra 7 155H / Intel xe-lpg WebGPU, 1656x894,
Editing workspace, 1/2 preview, standard single-thread WASM, fresh browser profile per run.
No compilation or CPU profiling ran during accepted measurements. Fixture: 16-second generated
1280x720 30 fps AVC/AAC clip, SHA-256
`0c6b57c1504f5a86f16c9ce2e3bee004163da0c8b6e9043a72e44fe38ce42cb5`.
Lumetri: exposure .6, contrast 12, saturation 110; Gaussian Blur: blurriness 8; Wiggle: catalog defaults.

Original Wave Warp medians were 8.23 alone and 1.14 with Lumetri+Blur, with large run variance.
Wave Warp remains a CPU path. No Wave or Glow fix or completed transition benchmark is claimed.
Some launch attempts produced no report and were replaced by named successful retries. A later
CPU-chain run stalled on an optional post-measurement screenshot and is excluded. Its final two
retries disable only that result screenshot; warmup and measurement remain unchanged.

## Verification actually completed

- Production optimized standard WASM package: **passed**, 42.9 MB WASM; exact hashes in JSON.
- `cargo test -p filmcraft-render --lib`: **199 passed, 3 ignored** (existing effect-cost/contact-sheet/lens-blur timing tools).
- `cargo test -p filmcraft-gpu --lib -- --nocapture`: **41 passed**, including actual Wiggle and Lumetri+Blur+Wiggle GPU parity.
- `cargo test -p filmcraft-ui-egui --lib`: **230 passed**.
- `cargo test -p filmcraft-ui-egui --test fullscreen_host_ui`: **2 passed**.
- Workspace/all-target Clippy with `-D warnings`: **passed**.
- Formatting: **passed**. Architecture/layers: **passed**. WASM compatibility: **49 crates passed**.
- Asset attribution: **passed, 178 attributed assets**, including the four new verification captures.
- Real Chrome combined audit: **passed, zero recorded browser errors, 26 captures**. Covered dark/light
  workspaces, 1656x894 and narrower 1280/1024 layouts, custom layout switching/reload, Source opening,
  project/timeline drag/drop, trim/split/move undo-redo, source patching and track targeting.
- Fullscreen: actual entry, header exit, external document exit and successful retry **passed**.
  Synthetic rejected Promise handling **passed**. Preferences gear opened the existing dialog;
  host-only controls left engine/project state unchanged.
- Physical Escape exit: **manual check still required**. Headless CDP Escape did not invoke Chrome's
  browser accelerator. Actual permissions-policy denial/embedding and other browsers remain unverified.
- Full native workspace test suite: **not completed**. An over-parallelized build was stopped for
  contention; its queued restart was canceled at the user's checkpoint request. This is not a passing
  full-suite result. Desktop application interaction and other OS/hardware validation remain pending.

Real captures preserved in [screenshots/v7-phase1](screenshots/v7-phase1): matched ordinary
Clean Editing before/after, actual fullscreen and Preferences. The Preferences capture intentionally
retains the status from the synthetic denial test. The 26-workspace before/after audit sets and raw
browser reports remain locally under `target/v7`; only the selected captures and compact numerical
summary are committed. No prototype or third-party artwork was incorporated.

## Complete master scope ledger

Pending/deferred here means **still required**, not removed from scope.

| Section | Requirement | Checkpoint status and exact remaining work |
| --- | --- | --- |
| 1 | Preserve engineering / branch | Completed for checkpoint; original branches untouched |
| 2 | Subtle Unified visual refinement | Pending; baseline design retained, no broad redesign claimed |
| 3 | Collision-safe inline hierarchy, Focus, Graph | Partial: measured multi-clip hierarchy verified; persistent duplicate identity, Focus and Graph pending |
| 4 | Animated/Modified/All/Selected, search, shortcuts | Pending; command/shortcut conflict audit first |
| 5 | Numeric/keyframe editing and undo | Partial: typed edits, add/remove/nav/seek and undo verified; inline selection/retiming/interpolation pending |
| 6 | Value/velocity curves and influence | Pending shared Graph Editor; retain canonical interpolation |
| 7 | Exact trim/split/slip/reverse/speed/nested domains | Pending full animation mapping matrix |
| 8 | Geometry, focus, virtualization, view persistence | Partial: shared measured geometry, clipping, scroll and compact restore verified; persistent disclosure, broader focus/scaling matrix pending |
| 9 | Compact Effect Controls, optional lane, no Fusion CTA | Pending; do not remove existing professional operations |
| 10 | Wave/Wiggle/Glow performance | Partial: Wiggle measured/fixed; Wave and Glow investigation still required |
| 11 | One header hierarchy, all menus | Pending consolidation; only host utilities changed |
| 12 | Visible existing Preferences / diagnostics | Partial: gear complete; diagnostics/category/capability review pending |
| 13 | Project Assets / Browse Files | Pending naming, navigation and truthful host-capability audit |
| 14 | Factory/saved/reset layout distinction | Partial regression coverage; UX distinctions and full matrix pending |
| 15 | Light/dark polish | Partial screenshot coverage; requested new visual refinements pending |
| 16 | Useful real-data Project Hub | Pending; preserve real presets and avoid fabricated recent data |
| 17 | Truthful Storage & Collaboration | Pending save/auth/sync/room/link audit and dialog work |
| 18 | Browser fullscreen / native compatibility | Partial: host API and native fallback verified; physical Escape/denial/other hosts pending |
| 19 | Editing/usability reliability | Partial focused browser regressions; remaining integration scenarios pending |
| 20 | Future AE authoring | Deferred by design: expressions, parenting, spatial tangents, rich templates |
| 21 | Phases, usage and checkpoints | Verified slices preserved separately; V7.4 rejected layout corrected before additional architecture |
| 22 | Tests, builds, performance | Partial: above gates pass; full native suite, remaining effects and platforms pending |
| 23 | Full deliverables / handoff | Partial checkpoint delivered; animation, complete UX, full evidence still required |

## Exact next steps

1. Verify this branch/remote and read this ledger, the complete master prompt and both references.
   Retain the current standard browser build and all previous engineering work.
2. Run the deferred full native regression suite with bounded parallelism before substantial new work.
3. Settle stable per-effect identity/lifecycle before persistent duplicate disclosure or detached graphs, without reusing
   filenames or array indexes. Clip IDs and parameter IDs already exist. General EffectInstance UIDs
   do not; existing EffectSel type/occurrence and inspector index strings are insufficient for all
   same-type duplicate/reorder/remove/undo cases. Do not silently change clip IDs or project semantics.
4. Extend the verified hierarchy with shared key selection, canonical preview/commit/cancel
   retiming and supported interpolation. Keep exact-time/unsupported-domain guards; add
   trim/split/slip/reverse/speed/hold/nested mapping cases before widening domain support.
   Add filters/toolbar and then Focus/Graph using the same canonical selected property.
5. Continue compact Effect Controls, shared selection/Graph/Focus, header/browser/layout/Hub/storage
   work and measured Wave/Glow investigation as tracked above. No merge or deployment without review.

## Reproduction

Corrected hierarchy acceptance (ordinary build, real procedural multi-track demo):

```text
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8771/ --out target/v7/hierarchy-browser-final --inline-tests true --media none --workspace "Clean Editing" --seconds 1 --benchmark-only true --seeks false --editing-tests true
```

The inherited `--benchmark-only` flag skips the broad workspace tour; this invocation is a
functional acceptance run, not an accepted performance measurement. Capture metadata includes
disclosure geometry, project revisions, actual edited values/keys and ruler transforms.
Current focused native suites:

```text
cargo test -p filmcraft-ui-egui --no-fail-fast --lib --test inline_animation_ui --test clean_editing_ui --test keyframes_ui --test timeline_track_height_ui --test escape_drag_ui --test shared_docking_ui --test menus_ui --test studio_pro_refinements_ui --test fullscreen_host_ui --jobs 2 --config profile.dev.package.filmcraft-ui-egui.opt-level=0 --config profile.dev.package.filmcraft-ui-egui.debug=0 -- --test-threads=2
cargo clippy -p filmcraft-ui-egui --all-targets --target-dir target/gates --jobs 2 -- -D warnings
```

Use installed Rust **1.99.0-x86_64-pc-windows-msvc**, setting RUSTC/RUSTDOC and PATH to that
specific toolchain, not the mutable stable alias. Standard packaging: `cargo xtask web`.
UI native test commands above used these compilation overrides:
`--config profile.dev.package.filmcraft-ui-egui.opt-level=0 --config profile.dev.package.filmcraft-ui-egui.debug=0`.
Clippy used `--workspace --all-targets --target-dir target/gates -- -D warnings`.

For each of plain (`none`), `lumetri,gaussian_blur`, `wiggle`, and
`lumetri,gaussian_blur,wiggle`, run three fresh-profile Chrome measurements against both builds:

```text
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8771/ --out target/v7/reproduce --media avc720p30.mp4 --effects wiggle --seconds 12 --benchmark-only true --seeks false
```

Use distinct output directories; the frozen base is served on port 8780 locally. Restore the fixture
to the dist directory after packaging. Fixture generation is documented in `unified-studio-final.md`.
`--result-screenshot false` disables only the optional post-measurement screenshot on slow CPU
references. For host controls, use `--host-tests true --seconds 1 --benchmark-only true` (functional
verification only). The local preview is **http://127.0.0.1:8771/**; nothing was published.
