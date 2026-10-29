# Performance architecture review — integration in progress

The engine remains shared between native and browser hosts. Project snapshots, exact ticks,
command-based edits, Source/Program independence, revision-keyed frame caches and the Unified UI
are preserved. Native decoding uses platform factories where supported and software fallbacks;
browser decoding uses the existing WebCodecs media source with native planes and CPU fallback.

Browser frame jobs are currently cooperative on the UI thread. WebCodecs decoding is asynchronous,
but native plane copyTo, allocations, texture uploads and procedural generation still consume
UI-thread time. Native frame workers must terminate when their FrameServer is replaced; the
adapted upstream lifetime fix now cancels queued/in-flight jobs and wakes parked workers.

The optional threaded application artifact parallelizes existing Rayon inner loops. It does not
replace the cooperative frame queue, does not make a synchronous join preemptible, and does not
require a custom browser. It requires shared WASM memory and appropriate site headers. Stable
1.99.0 builds the normal artifact; approved nightly-2026-10-28 builds only the optional artifact.
Both startup and the original single-threaded fallback still require real browser verification.

The measured Intel/Windows Chrome 720p30 path spends substantial sampled CPU time in copyTo.
Disabling accelerated video decoding in a controlled Chrome experiment improved presentation;
an application-level prefer-software hint for this measured small-AVC case is being tested.
No hardware-residency, zero-copy or actual decoder implementation claim follows from a hint.
Large/HDR/other-platform sources keep browser defaults. A GPU-resident path needs the fallible
frame API and explicit resource lifetimes described in upstream #1019/#1021/#1114.

Exports retain the existing renderer/encoder overlap and fixed interleave groups. Pending audio
mixes are now staged transactionally before any cursor or limiter history advances. This also
protects batches with multiple audio groups. Export quality, encoder decisions, color math,
sample rate, duration and final settings are unchanged by that fix. Bulk staging reads preserve
the existing half/float conversion and are covered by GPU parity tests.

Further analysis and final build/benchmark status will be appended after validation. Unsupported
platforms and unfinished experiments must not be described as verified.
