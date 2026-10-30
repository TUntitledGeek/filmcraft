# V7.8 — real Chrome interaction evidence

Captured from the production Rust/egui application in ordinary Chrome, with the procedural demo.
The fixture enlarges the real timeline dock through its divider. Rows scroll in the existing
video/audio halves; an intentionally scrolled owner can move offscreen. Screenshots are not
HTML prototypes. [Actual results and canonical key data](../../verification/v7-shared-animation.json).

| Interaction | Capture |
| --- | --- |
| Independently disclosed Gaussian Blur instances, with distinct real values | [Duplicate effects](shared-duplicate-effects.png) |
| Three selected canonical Opacity keys, before movement | [Before](shared-keyframes-before.png) |
| Actual held mouse drag; the project is still unchanged | [Preview](shared-group-drag-preview.png) |
| Release commits one exact 12-frame group move | [After](shared-keyframes-after.png) |
| Real marquee selecting two keys | [Marquee](shared-marquee-selection.png) |
| Existing Effect Controls retains shared selection | [Inspector](shared-inspector-selection.png) |
| Keys on V1/V2 selected and moved together in their source domains | [Multiple tracks](shared-multitrack.png) |
| Narrow desktop, 1280×800 | [Narrow](shared-narrow.png) |
| Light theme, 1656×894; the viewing surface remains neutral | [Light](shared-light.png) |

The production sequence is 24000/1001 fps.
The group move is 127135008000 exact timeline ticks, preserving key values,
interpolation and influence. Browser events perform individual/Shift/Ctrl selection, instance
menus, drag preview/release, Escape cancel, collision rejection, marquee, filters and a cross-track
drag. Setup and assertions read canonical engine data. Source names are never identity.

These are functional checks, not playback benchmarks. Known boundaries and all remaining V7
requirements are retained in the [implementation ledger](../../v7-implementation-status.md).
