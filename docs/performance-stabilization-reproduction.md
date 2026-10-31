# Reproducing the independent-worker checks

Run from the repository root on the source branch. Keep the before/after packaged
directories immutable and serve them on separate loopback ports. Do not compile,
profile, or run a second browser benchmark at the same time as accepted timings.
Record both WASM SHA-256 hashes and the exact Chrome version, GPU, CPU and viewport.
The local test server used ports 8782 (V7.9) and 8783 (worker); it sent no isolation
headers. These addresses are local previews, not a published release.

## Single imported-media run

Use ordinary installed Chrome. The script creates a separate temporary profile,
closes only its own browser and validates the owned profile path before cleanup.
It does not operate the user's editing tab. Fixtures are local synthetic media;
FFmpeg is an external fixture generator only, never linked or bundled.

```powershell
node apps/filmcraft-web/tests/stabilization.mjs `
  --url 'http://127.0.0.1:8783/?fresh&norecover' `
  --dist target/stabilization/worker-dist `
  --out target/stabilization/example `
  --workload imported --effects stack `
  --media /fixtures/avc1080p30.mp4 --seconds 6
```

Repeat against the frozen baseline directory/URL. Use three fresh profiles per
case/build, interleaving before/after and reversing the order on the second repeat.
Keep both builds at 1656×894, DPR 1, dark Clean Editing, Half resolution, Program
zoom 0.5, timeline 60 pixels/second and scroll zero. The script checks WebGPU; no
unsafe-WebGPU or disabled-hardware-decoding flag is supplied. Automated audio uses
Chrome's autoplay-policy flag. No custom browser, atomics or shared WASM memory is
required. Browser focus/host load can still introduce measurement variance.

Cases: 720p30 and 1080p30 AVC/AAC, each plain, Lumetri + Gaussian Blur, and
Lumetri + Blur + Wave Warp + Spherize. Grade values are Exposure 0.6, Contrast 12,
Saturation 110; Blur 8; Wave Height 12/Width 120; Spherize Radius 160. Values are
applied through canonical commands and persistent effect IDs.

`--workload multitrack --effects none` uses the procedural demo with its video,
transitions and audio. `--project <local.fcproj>` loads a private local copy,
retains its complete effect/keyframe state and stores only its hash, not project
contents. The Ocean benchmark limits each playback pass to 2.2 seconds so it stays
inside the first clip. Do not commit a private project or recovery snapshot.

`--workload large --media /fixtures/avc720p30.mp4` creates 14 distinct imported
sources and 1008 video clips. `--loop true --seconds 60` requests sustained looping
playback. These switches are test capabilities, not evidence that a run passed;
consult the ledger for actual executed cases.

Each run writes a JSON report, before/after real Chrome screenshots and a stage
log. It exercises real pointer selection during cold seeks, clip-edge trimming,
undo, move/Escape cancel, canonical effect parameter edits/undo, and an actual
Color-panel Exposure drag with exact canonical undo. The shared V7 harness covers
keyframe/duplicate-effect/menu interactions separately.

## Interpret the measurements carefully

* `presented` counts changes of the monitor's sequence-frame key, including late
  pictures. It is distinct from rAF and exact-deadline `shown`/`dropped`. Revisited
  keys can be counted again, so presentation changes/second are not a proof of
  globally unique frames or smooth source-rate playback. Clock discontinuities
  must be considered when inspecting deadline counters.
* Cold and warm are the first and second passes over the same interval in one
  profile. Warm does not imply the whole interval fits in every bounded cache.
  Report elapsed wall time, changed pictures, deadline counters and lag separately.
* Pointer-selection timing includes CDP mouse events, selection inspection and
  two animation frames. It is an end-to-end **upper bound with automation overhead**,
  not direct input-to-next-paint/INP. It cannot certify the strict 50 ms feedback
  target. Parameter-command UI acknowledgement and updated-picture waits are
  separate fields. Drag timing includes the scripted gesture itself.
* Long Tasks are browser main-thread entries. Their absence does not prove that
  all per-frame work is cheap; repeated sub-50 ms work can still hurt pacing.
* AudioWorklet output-missing frames and starved quanta start after priming and
  reset with audio generation. For looping tests, sum each generation's maximum
  across samples. Mixer underruns are different. Automated output counters do
  not prove physical audibility or sample-accurate A/V synchronization.
* WASM bytes are allocated linear-memory capacity, not resident system RAM.
  Browser/driver allocations are additional. Logical cache bytes may count shared
  planes more than once; this is not a single total process-memory ceiling.

## Functional checks and build

```powershell
node apps/filmcraft-web/tests/audio-output.mjs
node apps/filmcraft-web/tests/studio-review.mjs `
  --url 'http://127.0.0.1:8783/?fresh&norecover' `
  --out target/stabilization/functional --seconds 1 `
  --workspace 'Clean Editing' --wave-sphere-tests true `
  --popup-tests true --shared-animation-tests true --host-tests true
cargo +1.99.0 build -p filmcraft-web --target wasm32-unknown-unknown --release --jobs 2
wasm-bindgen --target web --no-typescript --out-dir target/stabilization/worker-dist `
  target/wasm32-unknown-unknown/release/filmcraft_web.wasm
```

The build also needs the matching files in `apps/filmcraft-web/web/`, including
`frame-worker.js` and the audio modules. Follow the repository's web packaging
workflow so versioned glue, WASM, bootstrap and worker assets agree. Do not publish
a bare WASM replacement or infer runtime correctness from compilation alone.
Build/check results and omitted checks are listed in the ledger. Full-workspace
CI, additional workloads and release review remain separate gates.
