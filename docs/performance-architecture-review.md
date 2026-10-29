# Performance architecture review

The engine remains shared between native and browser hosts. Project snapshots, exact ticks,
command-based edits, Source/Program independence, revision-keyed frame caches and the Unified UI
are preserved. Native decoding uses platform factories where supported and software fallbacks;
browser decoding uses the existing WebCodecs media source with native planes and CPU fallback.

Browser frame jobs are currently cooperative on the UI thread. WebCodecs decoding is asynchronous,
but native plane copyTo, allocations, texture uploads and procedural generation still consume
UI-thread time. Native frame workers must terminate when their FrameServer is replaced; the
adapted upstream lifetime fix now cancels queued/in-flight jobs and wakes parked workers.

The final active browser build remains standard single-threaded WASM, built with stable 1.99.0.
An approved optional Rayon/atomics artifact compiled with nightly-2026-10-28, but was withdrawn
before release. Its full source snapshot is preserved in
[`experiments/optional-wasm-threads.patch`](experiments/optional-wasm-threads.patch), with the
exact reconstruction base and limitations in the adjacent README. It targets ordinary Chrome
with site isolation headers, not a custom browser. It is not enabled in the final application.

Atomics exposed native GPU-device and egui Send/Sync boundaries. The experimental patch handles
those without unsafe assertions, but further review found a correctness risk: parallel audio
strip and multicamera source traversal can access event-loop-local Blob/WebCodecs state. A
worker pool must not be enabled until source I/O remains on the owning thread or uses an explicit
message bridge, with audible/output tests. Compilation alone was insufficient evidence. No
threaded runtime dependency or build option remains in active source; no speed claim is made.

The measured Intel/Windows Chrome 720p30 path spends substantial sampled CPU time in copyTo.
Disabling accelerated video decoding in a controlled Chrome experiment improved presentation;
the application-level prefer-software hint now improves the measured small-AVC case in three
ordinary-Chrome runs. See the matched conditions and limits in `performance-before-after.md`.
No hardware-residency, zero-copy or actual decoder implementation claim follows from a hint.
Larger frames, ten-bit AVC, other codec families and other platforms keep browser defaults.
Eight-bit AVC carrying HDR metadata is not separately classified or benchmarked by this policy.
A GPU-resident path needs the fallible
frame API and explicit resource lifetimes described in upstream #1019/#1021/#1114.

Exports retain the existing renderer/encoder overlap and fixed interleave groups. Pending audio
mixes are now staged transactionally before any cursor or limiter history advances. This also
protects batches with multiple audio groups. Export quality, encoder decisions, color math,
sample rate, duration and final settings are unchanged by that fix. Bulk staging reads preserve
the existing half/float conversion and are covered by GPU parity tests.

Final build/benchmark status and limitations are recorded in `performance-before-after.md`.
Unsupported platforms and unfinished experiments are not verified.

## Scheduling, ownership and memory

The frame server orders foreground monitor work before prefetch/thumbnails, caches by project
revision and source identity, coalesces compatible revisions, and retries asynchronous media
without caching incomplete pictures. The browser pump checks its budget **between** jobs: a
single generator/effect operation can exceed it. Rayon reduces work inside eligible operations;
it does not turn that pump into a preemptive scheduler. Native frame jobs remain on their
existing worker threads. Source preview retains an independent playhead and In/Out state.

Existing budgets are distinct: the frame-server cache starts at 768 MiB, the software GOP cache
shares a nominal 1 GiB budget, browser Blob reads have a 384 MiB chunk LRU, and WebCodecs retains
up to 40 decoded pictures per source. These are not a single process-memory ceiling. In
particular, this baseline's software GOP cache can enlarge its allowance for 64 frames without
the later upstream byte ceiling; a count-limited WebCodecs cache also grows with frame size.
Large-frame/many-source stress remains necessary. Do not promise that the sum fits an 8 GiB
machine or apply #3520's throughput relaxation without its preceding security ceiling.

Dropping a replaced native frame server now releases its queue, media, caches and worker-owned
state after cancellation. Ownership regression tests verify those lifetimes. This is distinct
from allocator/driver working-set reclamation and does not establish steady-state RSS or VRAM.
Browser VideoFrames retain the existing close-on-transfer/stale/error discipline; transfer,
closed-frame and fallback counters remain available through `filmcraft.info()`.

## Audio, desktop and export boundaries

Browser audio still uses an AudioWorklet supplied by UI-thread mixing ahead of device playback.
Main-thread stalls can starve that supply even when video decode itself is asynchronous. A
successful worker initialization or an audioWorklet flag does not prove audible A/V sync or
absence of underruns. The export transaction fix protects offline output cursor/limiter state;
it does not by itself solve live audio starvation.

Desktop platform factories remain behind the existing safe media interfaces: Media Foundation
on Windows, VideoToolbox on macOS, and VA-API/NVDEC on supported Linux hosts, with software
fallback. Registration means availability of a factory, not proof that a particular stream
used hardware. Hardware frame/session/fallback counters are the evidence to inspect.

Native GPU export shares one off-screen device and retains CPU fallback on unsupported plans
or device errors. It now requests the high-performance adapter unless `WGPU_POWER_PREF`
overrides it. Bulk copies of mapped rows precede scalar conversion, avoiding repeated uncached
reads without changing color arithmetic. The native end-to-end GPU export test actually
rendered eight frames (including odd dimensions) and matched CPU PNG pixels exactly. Browser
exports retain the stepped CPU renderer and do not register the native blocking GPU export
factory. GPU-resident decode/export, GPU YUV
readback and HDR/LUT shader migrations have not been introduced by this checkpoint.
