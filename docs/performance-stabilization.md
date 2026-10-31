# Editing and playback stabilization ledger

Date: 2026-10-31. Branch: `filmcraft-v7-animation-and-ux`.
Starting checkpoint: `aa2bf55565cf6835a53a2a76e140348a0843741a` (V7.9).
Requirements: [complete user brief](requirements/performance-stabilization-20261031.md).
The user also requests reviewing actual upstream issues/contributions and integrating
compatible improvements. All unfinished V7 requirements remain binding.
No new Pages deployment or merge without verification and release approval.

## Confirmed cause and preserved baseline

The cooperative browser frame server checks its budget **between jobs**. A source
decode, procedural picture or CPU effect fallback can occupy the UI thread for an
entire job. Reducing the budget does not bound input latency.
V7.9's GPU effect-chain improvements are preserved; they did not remove this
architectural limitation. See [the previous measured report](performance-ocean-stack.md).

V7.9 production artifacts are frozen locally in `target/stabilization/baseline-dist`.
WASM SHA-256: `1bcae9ffcd7a7f9f9280660c646f3b97a0163c80e2c76c3e0e04d00fbc461dff`.
Accepted before/after benchmarks require matched Chrome/hardware/project/quality
without compilation. Historical measurements are not a new responsiveness baseline.

## Architecture in progress — not yet runtime-verified

* Independent standard Web Worker, separate ordinary WASM memory, existing frame
  server/media/exact-time/render code. No atomics/nightly/shared-memory activation.
* UI owns canonical project, commands, undo and GPU compositor. Immutable snapshots
  and integer-tick tasks cross as JSON strings, not JS-parsed time numbers.
* Transferable binary pixel/prepared-upload planes preserve color, alpha, bit depth
  and GPU chains. A bounded receiver dictionary preserves immutable source identity
  across parameter edits, avoiding repeated copies and uploads.
* Worker owns Blob reads and WebCodecs; referenced files and render previews cross
  as immutable handles. Retired callbacks cannot reuse replacement-worker file IDs.
* Weak document completion and cancellation checks reject late work. The existing
  queue retains prioritization/coalescing. Paused obsolete work can be preempted;
  playback does not restart its decoder on every advancing frame.
* Unsupported GPU plans, including shader-startup fallbacks, execute in the worker.
  Explicit `?cooperative` remains available for matched diagnostics.

These describe implementation code/intent, not passing acceptance. Compile, test
and Chrome results will be appended after actual execution.

### Verified transport foundation

Native rendering/GPU library regressions passed: 204 render + 45 GPU tests,
0 failed, 3 existing render tests ignored. Four new transport tests cover binary
plane/prepared-upload round trips, exact integer ticks beyond JavaScript's safe
integer range, color/alpha/PAR preservation, unchanged-source identity reuse,
ordered transitions, fallback-image equality, invalid planes/references/dimensions
and bounded dictionary eviction/resend. Receiver consumes transferred byte planes
instead of cloning them again. Affected color/render/GPU Clippy, formatting,
dependency layers and attribution checks passed. Full-workspace CI is still pending.

The first independent-worker production build and Chrome functional smoke succeeded
(non-isolated ordinary Chrome); these are **intermediate** results. Playback smoke
exposed cancellation starvation when the clock advances independently of a slow
worker. The scheduling correction and final-source rebuild remain in progress;
no end-to-end performance acceptance is claimed for this intermediate build.

## Actual upstream review

Queried the fork and `storytold/filmcraft` on GitHub, including open/closed PRs and
issues. The fork had no PRs at review time. No bundle was blindly merged.

| Contribution | Assessment |
|---|---|
| [#2438](https://github.com/storytold/filmcraft/pull/2438), merged, head `022a4dbacd1dfe7f94f6bd5daa950d1608e81e53` | Adapting completion wakes/avoid native idle polling, rebuilt-server wake hook, cheap growing-file check; preserve fork cancellation/coalescing. |
| [#3729](https://github.com/storytold/filmcraft/pull/3729) | Growing-file enumeration overlaps #2438; do not apply twice. |
| [#3705](https://github.com/storytold/filmcraft/pull/3705), open, head `325467459e1232dcd9275291408ba88b9ab7c259` | Shorter preroll candidate, not architectural isolation. Pending startup/audio evidence. |
| [#3723](https://github.com/storytold/filmcraft/pull/3723) | Export reader ownership relevant; detailed compatibility review pending. |
| [#3763](https://github.com/storytold/filmcraft/pull/3763), open, head `12c018265fbbed7fcd09c70f77a59184991affed` | Work-area preview reuse useful; larger command/UI changes not imported. |
| [#3768](https://github.com/storytold/filmcraft/pull/3768) | Bokeh gather candidate; patch/correctness evaluation pending. |
| [#3769](https://github.com/storytold/filmcraft/pull/3769), [#3725](https://github.com/storytold/filmcraft/pull/3725) | Export scheduling candidates; native changes do not establish browser worker support. |

Earlier reviewed/adapted contributions remain in [the inventory](upstream-performance-inventory.md).
Deferred Rayon/atomics and broad lazy GPU-frame/residency changes remain deferred.

## Open acceptance work

1. Transport, cancellation, malformed-input regressions and production WASM build;
   Chrome bootstrap/imports/GPU and CPU-fallback parity.
2. Matched matrix: imported 720/1080 plain/grade+blur/full stack, animated Ocean,
   multitrack/transitions/audio, rapid seek and real trim/move/undo under load.
3. Distinct presented frames, long tasks, visible input feedback versus picture
   latency, memory, cancellation and audio underrun/A-V behavior.
4. Procedural Ocean cold-frame throughput without appearance changes. A worker
   alone does not make a slow source real-time.
5. Aggregate worker/UI/media/decoder/GPU budgets and large-project snapshot/layout
   costs. Bound decoded bytes without evicting requested frames during lookahead.
6. Export/background-preview `Session::pump_jobs` remains cooperative; complete
   main-thread isolation cannot be claimed while this remains true.
7. Sustained editing, save/recovery, V7 effect/keyframe/undo identity regressions;
   verified logical commits/push. Release review before deployment.
