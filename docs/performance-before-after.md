# Performance results and handoff

Verified source: `53b796fa` on `performance-upstream-integration`, based on Unified `825226cc`.
Protected branches were not merged, modified or deployed. Final standard production WASM passed
(42.9 MB WASM, 43.1 MB packaged). Ordinary Chrome needs no custom browser. The optional threading
experiment was withdrawn from active source and preserved separately.

## Controlled ordinary-Chrome measurements

Three fresh-profile, 12-second runs per condition, interleaved before/after for each workload.
No compilation, CPU profiling, unsafe-WebGPU flag or disabled-hardware-decode flag ran during
this final batch. Both versions used matching isolation headers, WebGPU, 1656×894 DPR 1,
the existing 1/2 playback setting and viewer-fit render scale. Hardware: Windows, Core Ultra 7
155H / Intel Xe-LPG; Chrome 155.0.8059.86. Baseline: frozen `825226cc`; after: standard artifact
of `53b796fa`, stable Rust 1.99.0 release with thin LTO and SIMD128.

Values are medians of three runs. rAF p95 is computed within each run, then medianed.
Presented images are distinct from UI FPS and exact-deadline shown images.

| Workload | Presented images/s, before → after | Deadline drops, before → after | rAF p95 ms, before → after | rAF stalls >50 ms, before → after |
| --- | --- | --- | --- | --- |
| Imported AVC 720p30 | 23.09 → 29.82 | 81 → 0 | 66.64 → 16.68 | 62 → 0 |
| Same AVC, Blur + Lumetri | 24.57 → 29.51 | 67 → 0 | 58.51 → 16.69 | 64 → 0 |
| Procedural demo | 11.34 → 11.22 | 275 → 273 | 142.01 → 133.35 | 121 → 118 |

Imported presentation improved about 29% without effects and 20% with the tested effects.
Procedural playback remains slow; no meaningful improvement is established there. These short
local fixture runs do not establish universal 60 FPS, arbitrary footage, 4K/8K or other hardware.

Automatic prefer-software applies only to eight-bit AVC profiles 66/77/100 at 720p or smaller
on the measured Intel/Windows Chrome combination, after a support probe. Larger frames,
ten-bit AVC, other codec families and other platforms retain browser defaults. Eight-bit AVC
with HDR metadata is not separately classified or benchmarked. The API reports the requested
policy, not Chrome's actual decoder implementation. Diagnostic overrides: `?decoder=default`,
`?decoder=hardware`, `?decoder=software`.

All three after-effects runs reported GPU effects ready, zero CPU-effect layers, zero decoder
or Canvas fallbacks, native planes and all 402 delivered VideoFrames closed. Six before/after
first-frame Program crops (three plain, three effects) were pixel-identical: 573×322 RGB at
x=647,y=75 in the fixed viewport. This checks those images, not exhaustive decoder equivalence.
Resolution, effect parameters and output settings were not reduced.

Machine-readable results, hashes, per-run data and UI evidence:
[`verification/upstream-performance.json`](verification/upstream-performance.json).
Raw reports/screenshots remain locally in `target/upstream-review/normal-*`.

## Profiling and deferred threading

Separate diagnostic profiles sampled about 4.94 s in VideoFrame.copyTo over a 12-second imported
play. Procedural costs concentrated in generation loops, fbm, expf and sinf. Preliminary Chrome
runs disabling hardware decode reached 29.34/29.12/28.86 images/s; they used the old unsafe-WebGPU
flag and are not mixed into the final comparison.

The approved optional atomics artifact compiled with nightly-2026-10-28 and wasm-bindgen-rayon
1.3.0. It was not runtime-benchmarked or enabled in final source: parallel audio/multicamera
source traversal can reach event-loop-local Blob/WebCodecs state. That needs an explicit I/O
boundary and audible/output tests before worker activation. See
[`experiments/README.md`](experiments/README.md): the full patch applies cleanly to its recorded
base. Final active source has no threaded runtime dependency or build option. The separate
installed toolchain does not change the normal compiler or require a custom browser.

## Verification

| Check | Result |
| --- | --- |
| Standard production WASM | Passed: 42.9 MB WASM / 43.1 MB package |
| Workspace Clippy, all targets, `-D warnings` | Passed |
| Formatting / layering / attribution | Passed; 174 assets attributed |
| `xtask wasm` | Passed, 49 crates |
| Export library | 123 passed, including pending-audio/limiter transactions and export oracles |
| GPU library | 41 passed |
| Native GPU export integration | Passed; eight PNG frames at 320×180, 17×13, 1×1, 257×129 matched CPU RGB exactly; GPU counter assertion passed |
| UI library | 230 passed |
| Source drop / playback | 3 / 9 passed |
| Timeline OS drop / escape / option drag | 11 / 6 / 2 passed |
| Timeline scroll-pinch / track heights | 11 / 2 passed |
| Timeline trim | 16 passed; 2 screenshot-generation tests intentionally ignored |
| Keyboard focus / workspaces | 2 / 3 passed |
| Decoder policy boundaries and overrides | 2 passed |
| Final ordinary-Chrome timing batch | 18 completed runs; zero runtime errors |

Ignored tests: `hover_bracket_screenshots`, `trim_waveform_screenshots`. The broad
`cargo test --workspace` rebuild was stopped during compilation for remaining allowance; it is
not a passed workspace run. The planned desktop release build was not reached. Native GPU
export correctness is verified; a current desktop release executable, native playback speed,
hardware-decode usage, macOS/Linux, discrete GPUs, long memory stress, 4K/8K media, live
collaboration under isolation, audible A/V sync and export speed remain unverified.

Earlier failures were resolved: drop fixtures open Info before floating it; new-track audio
drags maximize Timeline to land in empty space instead of Mix. Three scroll failures exposed
smoothed Ctrl/Command-wheel zoom entering the pinch path; wheel and real touch/Zoom now route
separately.

An ordinary-Chrome functional run passed Source double-click, Project/Timeline source drops,
trim/split/move undo, targeting/source-patch toggles, five exact sequential seeks and rapid seek.
Its timings overlapped compilation and are excluded from the table. The first attempt hit a
fixture 404 during repackaging; restoring the ignored fixture fixed setup.

Final UI review after compilation captured dark/light, independent Source/Program, Inspector,
Color and Project Hub. A real Lumetri drag changed temperature, updated the preview, returned
from drag scale 0.25 to released scale 0.75, and undo restored the value. Single-sample control
and preview upper bounds were 60.3/79.4 ms, including control round trips, not comparative
latency benchmarks. Custom layout switching and page reload both restored the saved dock.
Screenshots/report remain in `target/upstream-review/final-ui-review/`.

## Reproduce

Use stable Rust 1.99.0, wasm-bindgen CLI 0.2.129 and `cargo xtask web`. Reuse the fixture command
in `docs/unified-studio-final.md`; SHA-256:
`0c6b57c1504f5a86f16c9ce2e3bee004163da0c8b6e9043a72e44fe38ce42cb5`.
Copy it into dist after packaging. Keep baseline/final distributions separate with identical
COOP/COEP headers. Final local preview: <http://127.0.0.1:8771/>; frozen baseline: port 8772.
Repeat three times, with fresh profiles, interleaving versions and all compilation stopped:

```sh
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8771/ --media avc720p30.mp4 --effects none --seconds 12 --benchmark-only true --seeks false --out target/repeat/plain
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8771/ --media avc720p30.mp4 --effects gaussian_blur,lumetri --seconds 12 --benchmark-only true --seeks false --out target/repeat/stack
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8771/ --media none --seconds 12 --benchmark-only true --seeks false --out target/repeat/demo
```

The harness records hardware/browser/flags, counters, rAF pacing, queue samples, transfers and
screenshots. CPU profiles must be separate. Focused UI tests used
`profile.dev.package.filmcraft-ui-egui.opt-level=0` and `debug=0` overrides; codec/render and
production profiles were unchanged. UI test targets: `source_drop_ui`, `source_playback_ui`,
`timeline_drop_ui`, `timeline_escape_ui`, `timeline_option_drag_ui`, `timeline_scroll_ui`,
`timeline_track_height_ui`, `timeline_trim_ui`, `ui_keyboard_focus`, `workspaces_ui`.
Exact commands/results are retained in local `target/upstream-review/` logs.

## Preserved commits and next priorities

- `b1770082`: retire replaced frame workers and release caches/media.
- `9ad4e523`: preserve export audio cursors/limiter across pending retries.
- `350f65c7`: honor GPU export preference and bulk-copy mapped readback.
- `99771295`: separate timeline wheel scrolling from actual pinch zoom.
- `7c65b0cb`: upstream review and initial verified checkpoints.
- `1a2ab681`: correct new-track audio regression geometry.
- `bb788f02`: recoverable, explicitly deferred threading experiment.
- `53b796fa`: measured Chrome decoder preference and normal-browser test defaults.

Unified layouts/icons, track controls, editing commands, exact ticks, Source independence,
prefetch/revision coalescing and asynchronous GPU-effect validation were preserved. Replacements
are limited to demonstrated worker lifetime, export transaction, mapped readback, wheel routing
and measured browser decoder-selection issues.

Next establish safe browser media I/O before worker activation; then isolate procedural
generation, GPU-resident transfer and scope costs. Review GOP byte ceilings with waiter pinning
before relaxing large-frame caches. Complete native release/playback/audio and long memory/export
stress checks. Dependent HDR/LUT/codec SIMD/API migrations are inventoried, not silently integrated.
