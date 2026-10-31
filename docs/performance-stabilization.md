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

## Architecture implemented — acceptance measurements in progress

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

Transport is pushed as `af3cf658`; the browser integration is pushed as
`563e36b5`. Functional Chrome checks have exercised the independent worker.
The broader performance acceptance criteria remain open; measurements below
must not be read as a claim that the complete stabilization mission is finished.

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
| [#2438](https://github.com/storytold/filmcraft/pull/2438), merged, head `022a4dbacd1dfe7f94f6bd5daa950d1608e81e53` | Adapted completion wakes/avoid native idle polling, rebuilt-server wake hook and cheap growing-file check; preserved fork cancellation/coalescing. |
| [#3729](https://github.com/storytold/filmcraft/pull/3729) | Growing-file enumeration overlaps #2438; do not apply twice. |
| [#3705](https://github.com/storytold/filmcraft/pull/3705), open, head `325467459e1232dcd9275291408ba88b9ab7c259` | Shorter preroll candidate, not architectural isolation. Pending startup/audio evidence. |
| [#3723](https://github.com/storytold/filmcraft/pull/3723), merged 2026-10-31, head `9208a3053e108101f34cc6aa5be162e30c1f4c8a` | Reviewed actual patch: separate per-job file readers/decoders for export, previews and proxies; pending/whole-file inputs retain safe fallbacks. Relevant follow-up for native and remaining cooperative jobs. Not imported during the frozen browser benchmark. The browser frame worker already has its own media pool; this patch alone would not move browser export off the UI thread. Upstream Mac timings are not measurements of this fork. |
| [#3763](https://github.com/storytold/filmcraft/pull/3763), open, head `12c018265fbbed7fcd09c70f77a59184991affed` | Work-area preview reuse useful; larger command/UI changes not imported. |
| [#3768](https://github.com/storytold/filmcraft/pull/3768) | Bokeh gather candidate; patch/correctness evaluation pending. |
| [#3769](https://github.com/storytold/filmcraft/pull/3769), [#3725](https://github.com/storytold/filmcraft/pull/3725) | Export scheduling candidates; native changes do not establish browser worker support. |

Earlier reviewed/adapted contributions remain in [the inventory](upstream-performance-inventory.md).
Deferred Rayon/atomics and broad lazy GPU-frame/residency changes remain deferred.

## Worker integration verification in progress

The isolated-worker implementation now exists in the browser host, rather than
only as a transport foundation. It preserves the existing GPU compositor and
uses the original frame server inside the worker. Important integration fixes:

* Dispatch after the shared UI queues monitor requests; completion wakes the UI.
* Replace the entire worker/cache namespace when the document changes. Results
  hold weak completion handles and cannot resurrect retired documents.
* Explicit playback mode, independent of decoder catch-up margins: forward and
  reverse Source/Program playback can finish useful late frames; paused scrubs
  replace obsolete work. Full job cost drives lookahead, including late results.
* Paused obsolete jobs can be preempted without continually restarting playback.
* GPU initialization, unsupported stages/transitions and oversized valid plan
  metadata can use the CPU reference path **inside the worker**. GPU presentation
  retains float values/alpha; invalid planes or geometry remain errors.
* Worker media pools retire idle sources (12 recent media items, including their
  full/proxy handles); native pools retain their existing default behavior.
* Browser budgets: plan cache 256 MiB, RGBA cache up to 128 MiB, generator cache
  64 MiB/source, Blob chunks 128 MiB, transported source dictionary 128 MiB,
  WebCodecs 40 frames/96 MiB per source. The currently requested decoded frame is
  protected even if it alone exceeds the decoded-cache budget. Frame-plan byte
  accounting is conservative and can count shared backing planes more than once.
* An async WebCodecs copy owns a buffer lease until consumption, preventing
  overlapping `copyTo` operations from sharing writable scratch storage. At most
  one idle scratch buffer (up to 16 MiB) is retained; pending copies own theirs.
* `filmcraft.info()` reports main/worker allocated WASM linear-memory capacity
  (not OS resident RAM), decoder/cache bytes and actual AudioWorklet starvation
  counters. Output starvation is distinct from mixer/decode underruns. Output
  counters reset with audio generations; sustained-loop analysis must group them.
* Real Lumetri drag verification uses the active Color panel's existing parameter
  row target and checks canonical effect state/undo. No grading UI redesign is added.

Actual checks so far: 204 render + 47 GPU tests passed (3 existing render ignores),
3 engine cache/manifest tests passed, 13 browser-host policy tests passed, audio
worklet output/clock/counter tests passed. Latest frame-server suite: 16 passed,
including forward/reverse Program/Source behavior and paused-scrub cancellation.
The V7 UI regression run passed 287 tests before the final explicit-mode refinement;
the additional mode cases passed in the latest focused run. Affected native Clippy,
browser-target Clippy, layers and attribution checks passed. Final-source standard
stable release WASM build and browser-target checks passed; accepted measurements
are being completed. `xtask wasm` checked 49 crates. Full-workspace CI was not run.

The measured production artifact is version `9b84dda0f9a7dd0e`, 44,271,566 bytes,
SHA-256 `26b288fa346d52b61fcec0a12626d90914177ce905b839d2b17fdd9b2f6dec18`.
Local preview: `http://127.0.0.1:8783/`; the comparison baseline is port 8782.
Both use ordinary Chrome without cross-origin isolation. No deployment was made.
See [reproduction and measurement definitions](performance-stabilization-reproduction.md).
Presentation changes/second are not rAF or globally unique-frame FPS. Long-task
observations in the matched matrix cover playback; cold-seek editing has separate
gesture/command latency measurements. The 50 ms visible-feedback target requires
a direct input-to-paint check beyond the automation upper bound reported here.

Intermediate ordinary-Chrome checks passed menu ownership over clip edges/drawers,
shared key selection/real drags/marquee, duplicate instances, inspector sync,
fullscreen/Preferences and the complete enabled GPU stack. The worker ran with
`crossOriginIsolated=false`, standard WASM, no unsafe-WebGPU or custom-browser flag.
Imported AVC also played and completed real selection, trimming/move cancel and
undo. Intermediate runs taken during compilation are **excluded from performance
claims**. A test-harness warm-run lexical-scope error was corrected before the
accepted set. Final accepted runs must identify their exact WASM hashes.

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
