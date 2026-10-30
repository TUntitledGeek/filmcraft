# Ocean Sunset: keeping the complete effect stack enabled

2026-10-30. Scope: the V7 demo clip's Lumetri, Gaussian Blur, Wave Warp and Spherize,
with its existing Motion/Opacity animation. This is a rendering change; it does not rewrite
project effects, introduce animation storage, or remove V7 editing controls.

## Diagnosis

The captured clip had Lumetri temperature 73.7, Gaussian Blur 17.75, Wave Warp height 10 /
width 40 / direction 90 / speed 1, and Spherize radius 0. All were enabled, followed by
Motion (three Position keys), Opacity (four keys) and normal Time Remap. The sequence was
1920×1080 at 24000/1001 fps. Its `.mp4` display name belongs to a procedural demo generator;
no imported-video decoder was involved.

The planner previously rejected the entire GPU effect chain when it encountered either
Wave Warp or Spherize. Spherize's zero-radius CPU function did nothing, but its mere presence
still moved Lumetri and Blur to the CPU. In the ordinary browser build, these frame jobs run
cooperatively on the UI thread; an expensive job can exceed its budget and delay interaction.

Ten isolated Chrome measurements (two per condition) confirmed both independent blockers.
Median uncached 960×540 Program-frame waits were 1,120 ms with everything enabled, 1,040 ms
with only Wave Warp bypassed, 1,124 ms with only Spherize bypassed, and 264 ms with both
bypassed. Removing Lumetri and Blur as well gave 265 ms. Bypassing effects was a diagnostic
ablation, not the implemented solution.

## Implementation

- Wave Warp and Spherize now have evaluated GPU operations and ordered WGSL compute passes
  in the existing float32 effect pipeline. A normal/nonzero Spherize works as well as radius 0.
- Wave phase retains the CPU's elapsed-time semantics; animated parameters use the existing
  mapped effect time. Parameters, stable instance IDs, effect order and undo history are unchanged.
- Wave sampling retains transparent edges, premultiplied alpha and bilinear reconstruction.
  Split phase coefficients, cycle reduction and a range-folded degree-13 sine polynomial avoid
  native GPU sine error being amplified into pixel displacement. Existing strict parity
  tolerances were retained.
- A zero-radius Spherize needs no compute pass, while remaining an enabled canonical effect.
  Animated radii are evaluated each frame, so subsequent nonzero values still render.
- Unsupported devices, masks, other unsupported effects and extreme/non-finite evaluated
  parameters keep the existing CPU fallback. This does not make every FilmCraft effect or
  every combination GPU accelerated.
- Hostile-value tests exposed integer overflow in CPU bilinear sampling of infinite warp
  coordinates. The sampler now rejects non-finite/fully out-of-bounds coordinates before
  integer conversion and uses saturating neighbor arithmetic; a regression covers the panic.

## Verification and reproduction

Native tests on the pinned Rust 1.99.0 toolchain: **245 passed, three existing ignored**
(41 GPU and 204 render). Tests cover CPU-reference preservation, transparent/HDR samples,
zero and nonzero radii, automatic/explicit centres, negative wave directions/heights/speeds,
long-running phase, animated parameters, reordered/repeated effects, the complete planned
chain with source-in and Motion/Opacity keys, and malformed-coordinate safety. Affected
Clippy with warnings denied, formatting, dependency layers and asset gates passed.

```text
cargo test -p filmcraft-render -p filmcraft-gpu --lib --jobs 2 -- --test-threads=2
cargo clippy -p filmcraft-render -p filmcraft-gpu --all-targets --jobs 2 -- -D warnings
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8773/ --out target/ocean-stack-review --wave-sphere-tests true --seconds 1 --benchmark-only true --seeks false
```

The browser acceptance fixture uses real demo project data, canonical parameter commands,
persistent effect IDs and acknowledged Program pictures. It checks GPU execution with all
four effects enabled, edits each effect (including nonzero Spherize), and verifies exact
effect-stack restoration through undo/redo. The separate before/after benchmark uses a
private local copy of the captured project; the project and recovery/UI snapshots are not
published. Compilation is stopped before quantitative browser measurements.

The [verification record](verification/v7-ocean-stack-gpu.json) contains two fresh-profile
runs per build using the actual captured stack. Fixed-frame parameter edits with a cached
source improved from median **877 ms to 57 ms** (15.4×), with a 51–79 ms new-build range.
The median uncached half-resolution picture wait improved from **921 ms to 256 ms** (3.6×).
All effects and keys remained enabled. Native/GPU preparation time excludes GPU execution;
the reported waits include control acknowledgement and polling overhead.

The standard production WASM build passed, and the checked-in acceptance fixture passed
both locally and on the [public editor](https://tuntitledgeek.github.io/filmcraft/). It uses
ordinary Chrome WebGPU with no experimental threaded build or unsafe-WebGPU flag. The live
runtime source is `6fdb2b69`; the completed Pages artifact commit is `93090067`.
The release marker at `release-info.json` records the source and artifact digest.

[Real browser captures](screenshots/v7-ocean-gpu/README.md) show the unchanged authored
picture at frame 75. The Program-surface comparison, including clip outlines, had p99
maximum channel difference 1, mean 0.077 and worst 17 in 8-bit channels. The native strict
effect-stage checks passed at the existing 1e-4 tolerance. The whole native workspace sweep
was not repeated; the affected render/GPU suites, production build and browser tests were run.

## Remaining performance work

The source generator is an independent CPU cost. Restoring the GPU effect chain does not
by itself guarantee real-time first-pass playback of the procedural ocean. Cached-source
parameter edits and cold generation are different workloads and must be reported separately.
GPU submission/preparation counters are not GPU execution time, and UI FPS is not the count
of displayed sequence pictures. No smooth-60-FPS claim follows from these tests.

Optimize/profile procedural generation and broader ordered hybrid effect fallback separately,
with image correctness and the full V7 requirements retained. Focus Clip, Graph Editor,
remaining time-domain semantics and the other deferred UX requirements are unchanged.
