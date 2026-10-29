# Performance measurements — upstream integration

Baseline: fork `825226cc`, optimized WASM, Windows, Core Ultra 7 155H / Intel Xe-LPG,
Chrome 155.0.8059.86, 1656×894 DPR 1, existing 1/2 playback setting and viewer-fit render scale.
Three fresh-profile runs each: imported AVC 720p30 without effects, Blur + Lumetri, procedural
demo. No compilation ran during measurements. The fixture hash is unchanged from
`verification/browser-performance.json`. Raw reports live in `target/upstream-review/before-*`.

Separate CPU-profiled runs diagnose bottlenecks and are excluded from timing comparisons.
Imported playback sampled about 4.94 s in VideoFrame.copyTo over 12 s. Procedural playback's
largest sampled costs were generation loops, fbm, expf and sinf. This supports treating the two
workloads independently.

A diagnostic experiment disabled hardware video decoding in Chrome while retaining WebGPU.
Three imported 720p30 runs presented 29.34, 29.12 and 28.86 unique images/s, with 0, 3 and 6
deadline drops and rAF p95 of 21.3, 16.9 and 18.0 ms. This is evidence for investigating the
CPU-backed readback path, not an instruction that users disable browser acceleration and not
proof that an application-level preference achieves the same result. That preference awaits
ordinary-Chrome validation.

These initial runs used the prior harness's enable-unsafe-webgpu diagnostic flag, as recorded
by the invocation/source history. The harness now uses normal browser GPU defaults unless that
flag is explicitly requested. Any final comparison must match the GPU flags and isolation headers;
do not combine unmatched conditions into an improvement claim.

Correctness checkpoints so far: 123 export tests (including pending-audio transaction regressions),
41 GPU parity/readback tests, 230 UI library tests, Source drops 3, Source playback 9, timeline OS
drops 11, escape/cancel 6, option/plain moves 2, scroll/pinch 11, track heights 2. Additional suites,
the corrected new-track fixture, final WASM packaging and worker runtime verification are pending.
