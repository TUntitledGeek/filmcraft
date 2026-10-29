# FilmCraft V7 implementation status

## Checkpoint: verified partial implementation, 29 October 2026

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
| 3 | Collision-safe inline hierarchy, Focus, Graph | Pending; resolve effect identity, then implement progressively |
| 4 | Animated/Modified/All/Selected, search, shortcuts | Pending; command/shortcut conflict audit first |
| 5 | Numeric/keyframe editing and undo | Pending new inline controls; reuse canonical commands |
| 6 | Value/velocity curves and influence | Pending shared Graph Editor; retain canonical interpolation |
| 7 | Exact trim/split/slip/reverse/speed/nested domains | Pending full animation mapping matrix |
| 8 | Geometry, focus, virtualization, view persistence | Pending; collapsed timeline must remain unchanged |
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
| 21 | Phases, usage and checkpoints | Current verified slices preserved separately; no new large architecture started |
| 22 | Tests, builds, performance | Partial: above gates pass; full native suite, remaining effects and platforms pending |
| 23 | Full deliverables / handoff | Partial checkpoint delivered; animation, complete UX, full evidence still required |

## Exact next steps

1. Verify this branch/remote and read this ledger, the complete master prompt and both references.
   Retain the current standard browser build and all previous engineering work.
2. Run the deferred full native regression suite with bounded parallelism before substantial new work.
3. For the next V7 animation slice, first settle stable per-effect identity/lifecycle without reusing
   filenames or array indexes. Clip IDs and parameter IDs already exist. General EffectInstance UIDs
   do not; existing EffectSel type/occurrence and inspector index strings are insufficient for all
   same-type duplicate/reorder/remove/undo cases. Do not silently change clip IDs or project semantics.
4. Implement one small end-to-end disclosure/animation slice using canonical Params, Tick time,
   commands and undo. Reserve drawer geometry in track extents and hit testing. Keep ordinary
   collapsed editing unchanged. Then cover the master acceptance matrix before widening scope.
5. Continue compact Effect Controls, shared selection/Graph/Focus, header/browser/layout/Hub/storage
   work and measured Wave/Glow investigation as tracked above. No merge or deployment without review.

## Reproduction

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
