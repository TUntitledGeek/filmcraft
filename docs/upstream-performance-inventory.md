# Upstream performance integration inventory

Development starts at fork `825226cc` on `performance-upstream-integration`. Upstream was fetched
at `8d925e9df32d8755f22bd2635e400a6b3e93f69c` on 2026-10-28 (America/Toronto). No wholesale
merge, UI replacement, protected-branch update or deployment is intended. PR status comes from
GitHub metadata checked during this run; an open PR's synthetic merge SHA does not mean merged.

## Verified existing work

| Source | Status / scope | Problem and solution | Evidence / local status | Decision / risk |
| --- | --- | --- | --- | --- |
| [#2440](https://github.com/storytold/filmcraft/pull/2440) | Merged; both | Unbounded thumbnail textures; cap and LRU eviction that protects the current pass | Merge `0fd5c333` is an ancestor of our baseline | Preserve; low |
| [#2436](https://github.com/storytold/filmcraft/pull/2436) | Merged; desktop | Whole preview-file retention; positional readers | Merge `f4b7486b` is an ancestor | Preserve; low |
| [#1193](https://github.com/storytold/filmcraft/pull/1193) | Merged; both | Deep sequence copies and transcript rebuilds per redraw; snapshot borrowing and derived-content cache | Merge `eb37bea4` is an ancestor | Preserve; low |
| [Performance MEM1–MEM4](https://github.com/storytold/filmcraft/blob/main/docs/performance.md) | Implemented upstream; desktop | File readers, idle decoder eviction, global GOP budget and recycled planes | Local FileReader, MAX_LIVE_DECODERS=4, global 1 GiB GOP budget and frame plane pools are present. Large-frame retention still needs separate review | Preserve architecture; do not add redundant caches |
| Fork `947c8eb0` | Local; browser | Pending WebGPU validation incorrectly disabled supported effects | Asynchronous validation retained; previous 12 GPU tests and new 41-test GPU suite pass | Preserve; low |
| Fork `f2464e12`, `aed3b68b` | Local; both/browser | Prefetch revision cancellation, generator cache collisions, grading feedback, asynchronous decode starvation | Current source and handoffs reviewed; fresh browser baseline captured | Preserve unless a measured replacement wins |

## Initial compatible integrations

| Source | Status / scope | Bottleneck and actual solution | Upstream evidence | Local status / decision / risk | Local verification / outcome |
| --- | --- | --- | --- | --- | --- |
| [b4a15c603e / #3244](https://github.com/storytold/filmcraft/commit/b4a15c603e8654b320a98bad05b4f039612c7df2) | Merged; native | Replaced frame servers leave workers parked with old caches/media. Drop marks stop under queue lock, cancels in-flight work and wakes all workers; a taken-before-drop job also observes stop | Source identifies 2–6 retained threads and up to a 768 MiB frame cache per replacement | Missing in baseline; adapted while retaining our coalescing/yield behavior; low | New weak-ownership and cancellation-race tests pass. They prove release of caches/media/projects, not immediate OS RSS reduction |
| [Performance: web audio pending issue](https://github.com/storytold/filmcraft/blob/main/docs/performance.md) | Known unresolved issue; browser/export | Audio pull advanced cursors/limiter before a pending read was retried. Stage all interleave mixes before committing any audio state | Explicitly documented upstream; local source confirmed it | Adapted transactional single- and multi-block pulling; medium | 123 export library tests pass, including repeated startup/mid-stream pending reads, limiter history and a later block failing after an earlier successful mix |
| [#3053](https://github.com/storytold/filmcraft/pull/3053) | Open; native/GPU | Adapter ordering can pick slow integrated GPU; scalar reads from uncached staging memory are expensive. Honor power preference and copy mapped rows before conversion | Upstream Linux/Intel measurements, including 650→85 ms mapped-row read; not a local speed claim | Source diff reviewed and adapted without touching asynchronous effects initialization; medium | 41 GPU tests pass, including effects, HDR input, alpha, prepared uploads, compositing and readback. Discrete-GPU selection still requires suitable hardware |
| Local regression | Both/UI | Egui's smoothed Command/Ctrl-wheel zoom was mistaken for pinch despite raw wheel routing. Only touch or explicit Zoom events enter pinch handling | Three existing scroll regressions reproduced locally | Fixed; low | All 11 scroll tests pass, including explicit pinch preservation |

## Browser architecture candidates

| Source | Current status / scope | Actual proposal and evidence | Dependencies / local difference | Decision and risk |
| --- | --- | --- | --- | --- |
| [#764](https://github.com/storytold/filmcraft/pull/764) | Open; browser | Optional Rayon Web Workers; upstream eight-worker demo reported 193 shown / 1 dropped versus 102 / 98 inline in an 8 s run | wasm-bindgen-rayon, atomics-built std, isolation headers. Our frame queue and coalescing must remain independent | Adapt optional inner-loop parallelism first; medium/high. User explicitly approved the runtime and pinned nightly toolchain. Regular Chrome, not a custom browser |
| [#1836](https://github.com/storytold/filmcraft/pull/1836) | Open; browser | SIMD conversion, optional shared-memory packaging and adaptive pump budgets; headless/software-rendering results | SIMD128 is already enabled locally. Do not equate Rayon worker count with simultaneous frame jobs or port scheduling changes before runtime validation | Adapt packaging/runtime selectively; defer budget/scheduler changes until measured |
| [#1019](https://github.com/storytold/filmcraft/pull/1019) | Open; both | Fallible CPU access to lazily downloaded GPU surfaces | Prerequisite for #1021 and #1114; broad frame-consumer API migration | Investigate, high. Never introduce a GPU-only frame whose CPU consumers silently receive fabricated pixels |
| [#1021](https://github.com/storytold/filmcraft/pull/1021) | Open; Windows | Shared D3D11/DX12 decoded surfaces with lazy original-plane download | Depends on #1019; portable CPU fallback and format/lifetime parity required | Defer until shared API and Windows adapter interoperability are validated; high |
| [#1114](https://github.com/storytold/filmcraft/pull/1114) | Open; browser | Copy eligible WebCodecs frames into private GPU textures and materialize CPU planes lazily | Contains #1021/#1019; GPU-to-GPU copy, not zero-copy | Investigate, high. Current profile confirms copyTo is a major cost; do not import the dependent API stack blindly |
| [#3218](https://github.com/storytold/filmcraft/pull/3218) | Open; both | Source-aware timing, a short source-bound queue and latency-bounded directional lookback; upstream slow 4K50 HEVC report | Our revision coalescing and pending-job backpressure are newer local behavior to retain. Source timing must distinguish active clips and queued waits | Investigate after threading validation; medium/high |
| [#3217](https://github.com/storytold/filmcraft/pull/3217) | Open; native/software codecs | Pin frames requested by live waiters, ignore cancelled waiters, and avoid useless GOP re-walks | Source/lock-order diff reviewed; needs current cancellation flag accessor and concurrent codec tests | Candidate adaptation; medium |
| [#3059](https://github.com/storytold/filmcraft/pull/3059) | Open; both | Throughput planning for independently decodable frames | No in-tree upstream source advertises independence yet; long-GOP sources must not use sparse planning blindly | Defer broad planner replacement; high |

The inventory will be expanded with remaining GPU/LUT, codec, memory, scopes/import and export
candidates as their source and local compatibility are reviewed. Raw API metadata and benchmark
profiles are retained locally under `target/upstream-review/`; they are evidence, not instructions.
