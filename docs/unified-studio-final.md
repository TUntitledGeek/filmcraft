# Unified Studio integration

## Starting point and boundaries

Development branch: `unified-studio-final`, based on Antigravity's
`044d19752338bf06e028759d5a20702b8cc635da`. This is a descendant of Codex's
`1aad5038`; the engineering optimizations and icon attribution remain in its history.
`visual-design-v1` was absent from GitHub when inspected. It was fetched from the existing
local repository at `Desktop/antigravityfolder/filmcraft`, whose visual worktree was clean.
The original worktrees and the protected source/deployment branches were left intact.

The user-supplied HTML and image are visual references, not executable product requirements.
Their segmented viewer selector, Design Lab controls and conceptual overlays are not copied
into the editor. The application continues to render through Rust/egui and its real engine.

## Audit and decisions

Chrome captures use 1656×894, DPR 1. Editing is also checked at 1280 and 1024 px.
The first full visual audit used a read-only copy of Antigravity's existing browser build.
The reproducible release baseline was then built from `044d1975` and frozen separately.
Intermediate captures are visual checks, not performance evidence.

| Finding | Classification | Resolution |
| --- | --- | --- |
| Large Program viewer, compact folders and contextual dual viewers | Working foundation | Retained |
| Vertical and horizontal editing tools appeared together outside Clean Editing | Usability | One toolbar in factory layouts; a saved docked Tools panel takes precedence |
| Horizontal bar omitted grouped alternatives and narrow overflow | Functionality | Shared flyout implementation, including right-click, drag/release and long-press; all tools remain available through More |
| Source toggle could leave loaded footage visible when toggled off | Usability | Explicit open/close behavior; closing stops Source auditioning and preserves divider geometry |
| Workspace visits discarded unsaved divider/docking changes | Usability | Bounded session drafts; explicit save/reset retains control over persistent baselines |
| Track badges filled entire lanes with saturated blue | Visual hierarchy | Compact outlined source patches and tinted targets, with distinct explanations |
| Opening a sequence replaced compact track defaults with 60/56 px heights | Integration | Defaults are now 40/36 px; existing saved sequence views retain their heights |
| Position/anchor fields collided with keyframe controls | Usability | Responsive label/value columns, separate bounded X/Y fields and reserved navigator space |
| Source timecodes overlapped below 250 px | Usability | Independent timecode rows and narrower settings layout; geometry regression coverage extended |
| Fusion used dark hard-coded surfaces in light mode | Accessibility | Token-based canvas, toolbar, cards, text, inspector and connections |
| Persistent focus outline competed with clip and track selection | Visual hierarchy | Local panel-header indicator; actual keyboard-focused widgets retain visible rings |
| Caption-options button had no action | Functionality | Opens a real captions menu with transcript access and undoable subtitle-track creation |

## Design system

| Rule | Implementation |
| --- | --- |
| Typography | Existing licensed Inter UI and monospace timecodes; body/button 12 px, secondary 11 px, restrained semibold hierarchy |
| Spacing | 4 px base rhythm; 5 px docking gutters, 32 px panel strips, 24–28 px compact controls, 44 px folder rows |
| Geometry | 6 px panels, 4 px controls; timeline clips, rulers and plots retain precision geometry |
| Dark surfaces | Slate header/panels with distinct recessed fields and data plots |
| Light surfaces | White panels, pale slate chrome and readable secondary text; no dark-themed toolbar text hidden on dark surfaces |
| Accent | FilmCraft blue: `#6489eb` dark, `#3e67bd` light; soft tinted selections rather than large solid blue blocks |
| Focus | Small panel-header context marker; keyboard focus is drawn around the actual control |
| Media evaluation | Neutral `#0c0d10` viewing surface in both themes; footage is not theme-tinted and keeps its display aspect ratio |
| Semantics | Colored clip labels, node types/sockets, audio meters and grading gradients keep their specialized meanings |
| Disclosure | Subtle Source control, contextual Inspector, grouped editing tools and overflow; advanced operations remain accessible |

Shared changes flow through `theme::Tokens`, egui visuals and reusable widgets. Fusion, grading,
mixers, scopes, captions/graphics, Hub, Export, menus and dialogs keep their specialized layouts.
Folder rows remain compact in Clean Editing; List, Icon and Freeform media views retain their
existing selection, sorting, rename, drag, thumbnail and browsing behavior.

Workspace drafts keep up to 128 visited layouts in memory. They do not silently overwrite saved
workspace preferences. Use Save Changes or Save Layout As for restart persistence. A factory-layout
preview is reversible; explicit Reset restores the saved layout. Source closing is a rendering state,
not destruction of the stored dual-view split. Custom docking trees and floating groups remain intact.

## Verification and evidence

### Priority change checkpoint

The user redirected this run toward playback, reliability and actual editing workflows. The
visual overhaul is frozen at this checkpoint; additional work is limited to functional integration.
The latest completed production WASM packaging passed (42.9 MB WASM). Native UI library tests
passed (228), as did the new Source/divider/tool tests, narrow monitor geometry, project browsing,
Color, keyboard and sequence-tab suites. The wider regression run exposed two legacy docking
tests that assumed Info was already present in the default workspace; their setup now explicitly
opens Info. Those corrections and the final workspace/focus cleanup await the next complete rerun.
Workspace-wide Clippy is still running. This is a recovery checkpoint, not final acceptance.

Final results and commit IDs are recorded here after the acceptance runs. Local logs and captures
live under `target/unified-*` and `target/unified-review/`; generated media and screenshots are not
bundled into the application. No GitHub Pages deployment or merge into `studio-pro` is authorized.

## Reproducing the browser review

Use Rust 1.99.0 explicitly on this machine; do not let a changing `stable` alias replace the
compiler during a build. Build first, then stop compilation before performance runs.

```powershell
cargo +1.99.0 xtask web
node serve.cjs 8766
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --workspace "Clean Editing" --seconds 12 --audit all --out target/unified-review/after
```

The harness uses real Chrome/CDP, isolated temporary profiles, the control API and real pointer
events for grading. It records browser/hardware details, raw timing samples, frame queues,
render scale, final values, undo and workspace save/switch/reload results.

For imported media, generate an external test fixture (never a product dependency):

```powershell
ffmpeg -f lavfi -i "testsrc2=size=1280x720:rate=30:duration=16" -f lavfi -i "sine=frequency=440:sample_rate=48000:duration=16" -c:v libx264 -preset veryfast -crf 23 -g 30 -pix_fmt yuv420p -c:a aac -b:a 128k -shortest -movflags +faststart target/unified-fixtures/avc720p30.mp4
# Copy only this generated test fixture into each local test server's distribution directory.
node apps/filmcraft-web/tests/studio-review.mjs --url http://127.0.0.1:8766/ --workspace "Clean Editing" --media avc720p30.mp4 --benchmark-only true --seconds 12 --out target/unified-review/imported-after
```

The synthetic AVC/AAC clip is 1280×720, 30 fps, 16 seconds. Each comparison must use the same
fixture hash, browser, hardware, sequence, resolution, display size and cache state. Procedural
demo playback and imported-video playback are separate workloads.

## Interpreting performance

`playback.shown` is the existing count of exact pictures that met their sequence-frame deadlines.
It is not the count of every image that appeared on screen: late fallback pictures can be visible.
The new `playback.presented` counts changed sequence images presented during playback, including
late pictures. Repainting the same picture does not increase it. Deadline drops remain unchanged.
The original baseline does not expose this additional counter; compare its deadline counts with
the same metric in the final build, not with the new presentation count.

Browser requestAnimationFrame pacing, sequence-frame deadlines, changed image presentations,
source generation time and GPU submission time answer different questions. Seek and grading
timings include control/API latency and are upper bounds. Demo sources are procedural: WebCodecs
availability alone does not demonstrate imported-video acceleration. Use the final WebCodecs
decoder/transfer counters in `afterInfo` to verify that path.

WASM rendering remains cooperative on the main thread. Yielding between jobs cannot preempt an
expensive individual generator/effect job. Deeper worker/GPU-generator changes require a separate,
image-correctness-tested architecture effort. No 60 FPS guarantee is made.
