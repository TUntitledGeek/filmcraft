# FilmCraft — Performance First: Full Editing and Playback Stabilization

## Mission

Continue work on `TUntitledGeek/filmcraft`, branch `filmcraft-v7-animation-and-ux`, from the latest pushed V7.9 checkpoint.

We are temporarily freezing nonessential feature additions and visual redesigns.

**The current FilmCraft editor is not consistently usable because rendering and playback latency can cripple ordinary editing interactions. Your mission is to fix this at the application-architecture level.**

This is not another small effect optimization or general performance audit.

The objective is to make FilmCraft a responsive, reliable video editor in ordinary Chrome, even when editing projects containing several video and audio tracks, animated properties, color correction, blur, distortion and other expensive effects.

You have substantial implementation freedom, including larger architectural improvements when supported by evidence.

Do not stop after identifying one bottleneck. Trace the whole editing and playback experience and implement a coherent solution.

## Essential product behavior

The following principle should govern every decision:

**Rendering can take time. The interface cannot become unusable while rendering takes time.**

Users must be able to select, move, trim and rearrange clips, change effects, navigate the timeline, pause playback, undo changes and save their projects without the editor freezing behind expensive frame jobs.

When a frame cannot be processed in real time, the application should respond intelligently through existing quality controls, asynchronous rendering, cached previews or another verified solution.

A temporarily delayed video picture is preferable to blocking the application's controls.

The full-resolution, final-quality rendering path must remain correct. Do not silently modify effects, disable effects, lose keyframes or degrade export quality.

## Phase 1 — Establish realistic end-to-end performance baselines

First confirm the working branch, source integrity, existing optimizations and active browser configuration.

Read the existing architecture and verification documentation, especially:

- `docs/performance-architecture-review.md`
- `docs/performance-ocean-stack.md`
- `docs/performance-before-after.md`
- `docs/v7-implementation-status.md`
- Existing frame-server, media, GPU and browser performance tests.

Investigate the real application under representative workloads:

1. Imported 720p and 1080p footage with no effects.
2. Imported footage with Lumetri and Gaussian Blur.
3. Imported footage with Lumetri, Blur, Wave Warp and Spherize.
4. Ocean Sunset with all existing effects and animated properties.
5. Multitrack video with transitions and audio.
6. Rapid scrubbing and repeated seeking.
7. Timeline clip movement, trimming, ripple editing and undo/redo during render load.
8. Frequent effect parameter dragging while the preview updates.
9. Long playback sessions with cold and warm caches.
10. A substantial project containing many clips and media sources.

Where supported by the test environment, add 4K and memory-pressure workloads.

Measure more than frames per second:

- Actual distinct sequence frames displayed and missed deadlines.
- Cold versus warm preview latency.
- Input-to-visible-UI-feedback latency.
- Input-to-updated-preview latency.
- Browser long tasks and main-thread blocking.
- CPU and GPU rendering time.
- Decode and conversion costs.
- Cache hits, evictions and memory consumption.
- Audio underruns and A/V synchronization.
- Timeline drag/trim response during heavy rendering.
- Worker queue depth, cancellations and stale frame work.

Separate actual editing responsiveness from image-rendering performance.

Do not run compilation during accepted benchmarks. Preserve the baseline results.

**Limit initial profiling time. Begin implementation promptly once the primary bottlenecks are identified.**

## Phase 2 — Remove main-thread rendering bottlenecks

This is the highest architectural priority.

Inspect `crates/ui-egui/src/frames.rs`, browser host integration, frame scheduling and media source ownership.

The current browser build uses cooperative frame jobs on the UI thread. Its budget is checked between jobs, meaning one expensive job can block the interface far beyond the nominal frame budget.

Investigate and implement the safest effective architecture for separating expensive frame production from immediate UI interaction.

Consider dedicated Web Workers, message-based rendering services, transferable frame data, interruptible processing, or another technically sound approach.

Do not simply increase time budgets, remove expensive effects or hide the issue by reducing frame rates.

Respect WebCodecs and Blob ownership, WebGPU device boundaries, WASM constraints, audio safety and the existing unenabled threading experiment. Do not activate an experimental thread pool merely because it compiles.

The intended outcome is that expensive render jobs no longer monopolize UI event processing.

Cancellation and revision handling must prevent outdated rendering jobs from delaying newer edits.

The application should remain responsive while decoding, rendering and preparing previews.

## Phase 3 — Fix procedural rendering and first-pass playback

Profile `crates/media/src/generators.rs`, particularly Ocean Sunset.

The recently completed Wave Warp/Spherize GPU changes have substantially improved effect editing, but cold procedural frame generation remains slow.

Investigate the per-frame noise evaluation, repeated computations, allocation behavior, rendering resolution, caching and opportunities for parallel or GPU execution.

Evaluate solutions such as:

- Faster mathematically equivalent generation.
- More efficient noise calculation and reuse.
- GPU-based procedural generation where appropriate.
- Asynchronous frame preparation.
- Predictive frame caching and bounded pre-rendering.
- Incremental caching at suitable preview resolutions.

Preserve visual correctness and deterministic outputs where required.

The goal is to improve initial playback and scrubbing, not merely repeated playback of already cached frames.

## Phase 4 — Make playback resilient to heavy effects

The editor must support real projects with complicated effect chains.

Build on the successful GPU implementations of Wiggle, Wave Warp and Spherize.

Investigate the remaining scenarios in which unsupported effects, masks, transitions or compositing paths force large amounts of work onto the CPU.

Consider ordered hybrid GPU/CPU execution, smarter caching and render-plan partitioning where worthwhile.

Avoid unnecessary GPU readbacks, intermediate allocations and duplicate conversions.

However, recognize that not every arbitrary effect combination will be real-time on every device.

Implement a robust fallback strategy for workloads that cannot meet playback deadlines.

This may include background preview rendering, bounded preview caches, explicit preview-quality modes and reusable precomputed frames.

These mechanisms must be useful during actual editing, not only during export.

A user should not need to disable their entire effect stack just to move the playhead or manipulate a clip.

## Phase 5 — Intelligent caching and preview rendering

Inspect the existing frame cache, project revision keys, media caches, prefetching and render-preview architecture.

Preserve all existing optimizations that already work.

Implement or improve the mechanisms that will make repeated editing fast:

- Cache correctly reusable decoded and rendered data.
- Invalidate only affected frames or stages where safe.
- Prioritize current playhead and interaction requests over speculative work.
- Cancel stale requests during rapid scrubbing.
- Prevent unnecessary re-rendering of unchanged material.
- Manage memory and GPU resources within realistic browser limits.
- Investigate persistent, bounded preview caching where practical.
- Provide a predictable user-facing state when previews are being prepared.

Ensure cache invalidation remains correct for animated parameters, effect reordering, trims, splits and nested sequences.

Do not let caching introduce stale pictures, incorrect exports or unbounded memory growth.

## Phase 6 — Keep timeline interaction fast under load

Review Rust/egui layout, event handling, thumbnail generation, timeline drawing, hit testing, project snapshots, property controls and the V7 animation hierarchy.

Find unnecessary work performed on every UI redraw.

Optimize actual editing interactions, including:

- Clip selection and multiselection.
- Moving and trimming clips.
- Timeline zoom and scroll.
- Source/Program navigation.
- Parameter dragging.
- Keyframe selection and dragging.
- Undo/redo.
- Project save and recovery.
- Docking and workspace switching.

Editing must remain predictable even when video frame production is slow.

Do not break the recently implemented V7.8 shared keyframe selection, effect identity or undo history.

## Phase 7 — Acceptance criteria

This project should not be considered complete merely because Rust tests pass or one performance benchmark improves.

Use explicit responsiveness targets as engineering goals, then document what is achieved on the tested hardware.

Suggested initial targets:

- Ordinary timeline and UI interactions: p95 visible feedback within 50 ms.
- Heavy-load interaction: no sustained UI freezing; investigate every main-thread stall exceeding 100 ms.
- Commands such as Pause, Undo and selecting a different clip must remain responsive during expensive rendering.
- Representative 1080p24/30 editing workloads should reach their source frame rate during warmed playback when feasible on the test hardware.
- If a demanding effect stack cannot sustain playback, the editor must still remain interactive and provide a usable preview strategy.
- Memory usage must remain bounded during long editing sessions.
- Saving, undo/redo and project data must remain correct.

Targets are not permission to fabricate passing results. Record any missed criteria and their causes.

Run at least one realistic sustained editing test rather than relying entirely on scripted isolated operations.

Test both imported media and the procedural demo. Include real audio playback where possible.

## Phase 8 — Implementation and delivery

This is a substantial engineering effort. Treat it as one coherent performance-stabilization program, with safe intermediate commits rather than disconnected feature experiments.

Implement the highest-impact changes first.

Run the necessary native, GPU, codec, project, UI and WASM regression tests.

Use actual Chrome browser interaction tests and carefully matched before/after performance measurements.

Preserve the existing source branch and all verified V7 work.

Maintain a persistent performance ledger recording:

- Confirmed bottlenecks.
- Implemented architectural changes.
- Baseline measurements.
- Post-change measurements.
- Functional acceptance results.
- Remaining limitations and their severity.

Commit and push verified checkpoints to `filmcraft-v7-animation-and-ux`.

Do not merge or publish a new GitHub Pages build until the changes have been fully verified and release approval is provided.

Do not spend the entire session producing reports. Make real code improvements and preserve progress before usage limits are reached.

## Final success definition

FilmCraft should feel like an editor that can actually be used for sustained creative work.

The user should be able to open a project, play footage, scrub, trim, rearrange clips, stack effects, edit keyframes and save without repeatedly waiting for the entire application to catch up.

**I am not asking for a handful of isolated performance wins. I am asking for the editing and playback architecture to be stabilized so that performance is no longer the main obstacle to using FilmCraft.**

Preserve the V7 animation and UX roadmap, but treat performance stabilization as the immediate prerequisite for further feature development.