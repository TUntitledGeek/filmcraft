# Browser playback and editing stabilization

The Unified UI is frozen for this phase. The recovery checkpoint is `dcbe5ccc` on
`unified-studio-final`, based on Antigravity `044d1975` and retaining Codex `1aad5038`.
No deployment, protected-branch update or merge was performed.

## Recovery handoff at the user's 5% usage warning

Source is pushed through `6e8c68d5` (`947c8eb0` GPU effects, `aed3b68b` Source/decoder fixes,
`6e8c68d5` Fusion and evidence). The working tree was clean after remote-hash verification.
The final optimized production WASM compile **and wasm-bindgen packaging passed**: 42.9 MB WASM,
43.1 MB distribution, recorded in `target/performance-production-final.log`.
The earlier production package including the GPU,
decode and Source-drop fixes passed and was used for the browser measurements below.

Verified in this phase: 228 UI library tests; Clean Editing 10; keyboard 7; monitor geometry 8;
shared docking 12; existing Source-to-timeline 13; layer and asset checks. Actual Chrome pointer
checks passed media double-click after warm-up, Project-to-Source replacement, timeline-to-Source
without timeline edits, trimming with undo/redo, and keyboard split/undo.
Workspace formatting also passed. The preview returned HTTP 200 after final packaging.

The browser track-control assertion initially inspected sequence contents instead of editor
targeting state; the harness now queries `state.inspect`, but that corrected assertion, normal
move check and rapid-seek extension await a final browser rerun. Initial demo warm-up can delay
double clicks; the browser gesture checks wait for that warm-up and do not prove cold-start
responsiveness. The native media-drop fixture now isolates drag loading/replacement from browser
double-click coverage; its final rerun is pending. Its timeline-drop and unsupported-bin cases
passed. Do not describe these pending checks as passed.

Background commands already queued (do not start duplicate jobs):

- `target/performance-editing-final.log`: remaining Source/playback/workspace/timeline suites.
- `target/performance-gpu-parity.log`: native GPU/CPU parity; then media tests write
  `target/performance-media-final.log`.
- `target/performance-clippy-final.log`: workspace/all-target Clippy, waiting for the build lock.
- `target/performance-production-final.log`: final optimized WASM package, **passed**.

Use pinned Rust 1.99.0. No compilation may overlap any resumed performance benchmark. The
preview server remains local at `http://127.0.0.1:8766/`. Keep `studio-pro`, `visual-design-v1`
and `gh-pages` untouched. This handoff deliberately records incomplete acceptance checks rather
than spending the remaining usage on further profiling or redesign.

## Findings and changes

The existing GPU effects implementation was effectively disabled in Chrome. Its constructor
polled the validation future once and discarded the effects stage if that future was pending.
WebGPU validation resolves asynchronously. Supported effects therefore ran through the CPU
reference inside GPU presentation, outside the frame server's reported rendering budget.
Profiles showed frame conversion, Blur passes and Lumetri power functions on the UI thread;
small frame-server render timings did not describe this work.

The browser now retains the stage and awaits validation, then installs it on the next compositor
pass. Actual validation failure retains the existing CPU fallback. No shader math, effect
parameters, export settings or export quality were changed. `perf.stats.gpu` exposes
`effectsStatus`, `cpuEffectLayers` and `gpuEffectLayers`. All matched effect runs reached
`ready`, processed effect layers on the GPU and reported zero CPU effect layers.

A second problem affected paused seeking: after a pending asynchronous decode, the cooperative
pump could start unrelated background jobs that repositioned the same decoder before its
output arrived. It now yields on the first pending job. An inactive monitor requests frames
below the active transport's priority, and Source playback receives the same foreground budget
as Program. These changes apply backpressure without replacing the media engine or caches.

Source now accepts the existing Project Bin media payload, including replacement of an already
loaded Source. Timeline-to-Source consumes the existing move gesture before a timeline command
can commit and resolves the grabbed timeline point through the clip's source-time mapping.
Unsupported drops produce feedback and leave the project unchanged. Existing Source-to-timeline
payloads retain their own semantics. There is no second drag architecture.

Fusion is absent from all primary header navigation densities. Its saved workspace identity,
node data, processing and rendering remain intact. Workspace menus retain **Fusion
(Experimental)** for legacy access.

## Controlled measurements

Measured sequentially after compilation finished, on Windows, Intel Core Ultra 7 155H (22 logical
CPUs, 31.4 GiB RAM), Intel Xe-LPG WebGPU, Chrome 155.0.8059.86 headless, 1656 × 894, device scale 1.
Each case starts a fresh Chrome profile, imports the same file, warms the paused view, then plays
for 12 seconds. No CPU profiler was enabled for these comparisons. Both loopback servers used
the same no-store policy and were not cross-origin isolated. One run per condition: these are
directional measurements, not statistical guarantees or foreground-browser hardware coverage.

The fixture is a generated 16-second AVC/AAC MP4, 1280 × 720, 30 fps, GOP 30, CRF 23.
SHA-256: `0c6b57c1504f5a86f16c9ce2e3bee004163da0c8b6e9043a72e44fe38ce42cb5`.
Blur uses blurriness 8. Lumetri uses exposure 0.6, contrast 12, saturation 110. Stacked means both
effects are enabled. The existing 1/2 playback setting and automatic viewer-fit render scale
were identical. Paused quality remains independently controlled by the existing monitor setting.

| Workload | Presented fps before → after | Deadline drops before → after | Browser rAF p95 ms before → after | Pause round trip ms before → after |
| --- | ---: | ---: | ---: | ---: |
| Imported, no effects | 21.8 → 21.2 | 91 → 110 | 75.0 → 75.2 | 22 → 15 |
| Imported, Blur | 9.0 → 21.2 | 329 → 115 | 150.0 → 75.1 | 204 → 20 |
| Imported, Lumetri | 2.6 → 20.7 | 341 → 129 | 250.0 → 83.2 | 473 → 22 |
| Imported, Blur + Lumetri | 2.4 → 19.7 | 331 → 150 | 291.7 → 83.4 | 140 → 20 |
| Procedural demo | 9.2 → 9.1 | 275 → 278 | 133.5 → 133.6 | 87 → 80 |

Presented fps is changed sequence images actually drawn divided by elapsed wall time. Repaints
of the same image do not count. `playback.shown` instead counts exact images that met their
sequence deadline; `dropped` is missed sequence deadlines. Late images can be presented, so
presented + dropped is not the number of sequence frames. Browser rAF cadence describes the
whole page, not just video. Pause/seek/parameter timings include automation round trips and
are upper bounds. GPU `submitMs` measures CPU submission/fallback work, not GPU execution time.
The stacked run's browser long tasks fell from 29 (5,891 ms total) to zero in the measured window;
its rAF p95 still exceeded 83 ms. The latter prevents a claim of consistently responsive pacing.

The matched frozen baseline WASM SHA-256 is
`c81d1dd3ee848d05d1ca10c4ba64723b21ecddafb67dd74ce672634ce28d8ecb`.
Raw reports and screenshots are in
`target/performance-review/matched-{before,after}-{none,blur,lumetri,stack,demo}/`.
The compact counters, hardware details and hashes are preserved in
[`verification/browser-performance.json`](verification/browser-performance.json).
The baseline contains the Unified checkpoint and presentation instrumentation, before the
performance fixes. Earlier CPU-profiled runs are diagnostic evidence only, not table inputs.
An interrupted host-sleep diagnostic was excluded.

In the final stacked-effects seek smoke test, five paused seeks to 1, 5, 2, 8 and 3 seconds all
displayed the requested frame, with preview upper bounds of 262, 211, 252, 19 and 268 ms. The
earlier baseline diagnostic failed to display all five requested positions within its polling
window. Treat those as a functional regression check, not a matched latency distribution.

## Applicable established patterns

- [WebGPU error handling](https://gpuweb.github.io/gpuweb/explainer/): error scopes return
  promises. Preserve a potentially valid pipeline until validation finishes; pending is not
  unsupported. This directly explains and fixes the observed CPU fallback.
- [Chromium WebCodecs guidance](https://developer.chrome.com/docs/web-platform/best-practices/webcodecs):
  decoding is asynchronous, but frame callbacks and frame handling can still overload the main
  thread. Keep queues bounded and close frames. The browser counters confirm native-plane
  transfers and frame closing here; the current path still copies planes into WASM and uploads
  textures. Codec availability alone does not prove a zero-copy path.
- [GStreamer scheduling](https://gstreamer.freedesktop.org/documentation/application-development/advanced/threads.html)
  and [clocks](https://gstreamer.freedesktop.org/documentation/application-development/advanced/clocks.html):
  separate streaming work with explicit queues and synchronize presentation to timestamps.
  FilmCraft already has exact ticks, deadlines and a foreground queue. Yielding a pending job
  and lowering inactive-monitor priority fit that architecture.
- [FFmpeg filtergraph documentation](https://ffmpeg.org/ffmpeg-filters.html): model processing as
  an ordered graph with explicit frame formats. FilmCraft already has plans and GPU effect
  chains; fixing their activation is safer than introducing a parallel effects implementation.
  Only documentation was studied; no GPL/LGPL source or runtime dependency was imported.

## Reproduction

Build before testing and stop all compilers during measurements. On this host pin Rust 1.99.0;
the shared `stable` alias changed during the earlier run. See `unified-studio-final.md` for build
and fixture-generation commands. Keep two frozen distributions served with identical headers.

```powershell
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --media avc720p30.mp4 --effects gaussian_blur,lumetri --seconds 12 --benchmark-only true --seeks false --out target/performance-review/repeat-stack
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --media avc720p30.mp4 --effects gaussian_blur,lumetri --seconds 8 --benchmark-only true --out target/performance-review/repeat-seeks
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --seconds 2 --benchmark-only true --seeks false --editing-tests true --out target/performance-review/repeat-editing
```

Omit `--media` for the procedural demo. Add `--cpu-profile true` for diagnosis, then compare only
similarly profiled conditions. The full harness also tests an actual Lumetri pointer adjustment,
release quality, undo, themes, narrow viewports and saved-workspace reload.

## Remaining limits and next work

This is a large effects-path improvement, not smooth 30/60 fps acceptance. No-effects playback
remained around 21 fps on this machine and both effect-heavy cases still missed many deadlines.
The procedural demo did not improve: source generation averaged about 64 ms per job, with a
106 ms p95, and is still cooperative WASM on the UI thread. Yielding between jobs cannot preempt
one expensive generator, CPU fallback or readback.

Next work should isolate procedural generation and frame conversion in bounded workers or
existing GPU stages, with image parity and cancellation tests. A measured, visibly indicated
adaptive interactive resolution is another option; no silent quality reduction was introduced
here. External-texture/transferable-frame integration needs explicit lifetime handling and a
CPU/export fallback; it is not implemented. Intermediate caches remain revision/scale keyed,
and this run does not redesign them.

Long-duration memory-growth testing, broad real-camera codec coverage, foreground Chrome
comparisons on additional GPUs, disabled-WebGPU fallback performance and export image
comparisons remain outstanding. No claim of smooth 60 fps or complete full-workspace CI is made.
