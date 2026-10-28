# Studio Pro implementation and verification

Starting point: `studio-pro`, `69236d85266d8a7656531b2576ddd7e8cb45b445`.

## Architecture findings

- The engine owns project edits and undo; UI changes continue to use its existing commands.
- Dock trees and floating panels are serialized separately per workspace in `workspaces.json`.
  Saved entries take precedence over presets. Clean Editing is a new preset; existing Editing
  and custom saved layouts retain their geometry. Contextual hiding changes rendered rectangles,
  never the stored tree or split paths. Saving Clean Editing makes its geometry explicit.
- The GPU preview consumes frame plans; software rendering consumes RGBA frames. WASM processes
  these jobs cooperatively on the main thread; budgets apply **between** jobs, not within them.
- Generator sources already have a revision-independent source-frame cache. Reusing the source
  and GPU texture is preferable to adding another revision-dependent cache.

## Changes

- Clean Editing prioritizes Program and a full-width timeline, with a compact media/effects area.
  Source appears for loaded footage; clip selection reveals Properties/Effect Controls. Both can
  be pinned. Classic Editing and the old Editing preset remain available. Save Layout As and
  Reset to Saved Layout remain accessible.
- Fusion places the graph beside Program and the timeline, retaining the graph's existing node inspector.
  Color prioritizes Program, scopes, grading controls and timeline without an irrelevant project bin.
- The Project Hub opens actual projects, creates blank sequence presets, loads the genuine demo,
  and browses real folder entries. It no longer offers unavailable catalog files or a fabricated
  collaboration link. Link Folder invokes the host picker and reports unsupported hosts truthfully.
- Icons adapt upstream PR #2308, pinned at `d3f0ba48dd008fd6baac5d1b879f5931adcae59c`.
  SVGs, attribution sidecars, MIT notices and provenance are included. Parsing is cached once.
  Six original editing-operation glyphs preserve recognizable bracket and insert/overwrite semantics.
  The public Icon/paint/button API is unchanged. Header and empty-state emoji are replaced.
- Playback prefetch requests no longer cancel one another. WASM revision coalescing preserves
  the due frame and future frames, including reverse playback and preroll. One completed job is
  processed per pump before yielding; this is not a guarantee against long single-job stalls.
- Grading's quarter-resolution preview applies to Lumetri edits in Program, not unrelated effect edits or Source.
  On release, the paused resolution and exact final values return through the existing undo path.
- Procedural noise reuses exact lattice hashes. Aurora and dunes reuse features that are constant
  down each column. Pixel-equivalence tests cover full and draft quality. The redundant global
  generator cache (which collided between different color mattes) and the unconditional WASM
  half-resolution cap are removed; full-quality rendering honors the requested resolution.

## Reproducing browser evidence

`apps/filmcraft-web/tests/studio-review.mjs` drives real Chrome using CDP and `window.filmcraft`.
It writes screenshots and raw reports under the requested output directory, at 1656×894, DPR 1.
Use the same Chrome, hardware, sequence, resolution and effects for comparisons. Compile before
measuring, and run without other builds. Browser FPS, actual displayed sequence frames, and
decode/render job time are different metrics. Demo sources are procedural, so their generation
appears in source/decode time; WebCodecs availability is not evidence of codec use.

```powershell
cargo +1.99.0 xtask web
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --workspace "Clean Editing" --out target/studio-review/after
node apps/filmcraft-web/tests/studio-review.mjs --url https://filmcraft.online/editor --out target/studio-review/original-editor
```

## Verification status

The user requested a scope freeze and source preservation with the remaining usage allowance.
No GitHub Pages deployment has been performed. The local preview is `http://127.0.0.1:8766/`,
serving `target/web/dist`; Project Hub → Load Demo → Clean Editing opens the review layout.

Source recovery points (pushed to `origin/studio-pro`):

| Commit | Contents |
| --- | --- |
| `a72fcaa2` | Baseline formatting and mechanical Clippy cleanup |
| `8fa633d2` | Licensed SVG icon integration and original editing glyphs |
| `684b9623` | Contextual Clean Editing, Fusion/Color, truthful Hub and browser layout persistence |
| `f2464e12` | Prefetch/revision fixes, exact procedural reuse and grading preview policy |

Completed verification:

All final focused native suites passed: **349 tests, zero failures**. Detailed results are in
`target/essential-core.log` and `target/essential-ui.log`.

- Native media library: 78 tests passed, including exact procedural pixels, cache isolation and reuse.
- Collaboration library: 7 tests passed in the final pinned-compiler rerun.
- Native UI library: 227 tests passed, including SVG parsing and playback queue coalescing.
- Focused UI suites: Clean Editing 3, Color 7, Studio Pro 25, workspace persistence 2 passed.
- Desktop debug build, asset attribution (174 assets) and dependency-layer checks passed.
- Workspace-wide release Clippy with warnings denied passed. Formatting passed.
- Final pinned-compiler production WASM build and wasm-bindgen packaging passed: 42.9 MB WASM,
  43.1 MB total distribution (`target/production-verified.log`).
- Real Chrome loaded that final release WASM, captured six workspace/theme screenshots, and verified
  a real Lumetri pointer drag, release quality restoration, exact undo, workspace switching and
  saved-layout restoration after page reload, with no recorded runtime exceptions. Final smoke
  evidence is under `target/studio-review/preservation/`; the earlier 12-second run is in `final/`.
  The two-second final run is a smoke check, not a performance benchmark.

The full workspace test run did **not** finish. The shared `stable` toolchain changed from Rust
1.99.0 to 1.99.1 during compilation, causing missing/incompatible standard-library errors in
dependencies. Final essential checks use the installed, pinned 1.99.0 compiler; this is not a claim
that the entire CI matrix passed. Logs are local under `target/`.

Final focused commands use Rust 1.99.0, `--locked --offline -j 4`: media/collaboration library tests,
then UI library plus `clean_editing_ui`, `color_ui`, `workspaces_ui` and `studio_pro_refinements_ui`.
UI test compilation uses the existing development profile with the UI package optimization/debug
overridden to zero; the production WASM build uses the normal optimized release profile.

Browser playback stutter remains unresolved. Measurements made while builds were running are
not comparable: they ranged from 15 to 184 displayed demo frames in 12 seconds across local
iterations. No stable FPS, latency improvement, or performance acceptance is claimed. The demo
uses procedural sources; imported-video/WebCodecs playback and export need separate validation.
Further performance work and the full test matrix are deferred rather than consuming the remaining
allowance. Native unit tests alone do not establish browser smoothness.
