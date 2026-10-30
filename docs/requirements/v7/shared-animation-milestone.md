# Next V7 milestone — user priorities, 30 October 2026

Continue from pushed `filmcraft-v7-animation-and-ux` checkpoint `bfb47546`, preserving V7.5/V7.6.
The full master prompt, animation plan and later hierarchy correction remain requirements.

First fix the reported clip context menu interruption: moving over an animation drawer,
disclosure or clip resize edge must not hide/intercept an already open foreground menu.

Then prioritize:

1. Stable effect-instance identity with compatibility-safe duplicate/reorder/remove/undo handling.
2. Shared inline/Effect Controls key selection: individual, Shift/Ctrl multiselection and marquee.
3. Real interactive key dragging: snapping, preview, collision protection, cancel, undo/redo and
   accurate supported time domains.
4. Selection synchronization with Effect Controls and the eventual Graph Editor.
5. Compact Animated/Modified/All/Selected filters and animation toolbar where safe.

Preserve normal clip editing and the progressive disclosure hierarchy. Exercise real Chrome
key drags, duplicate effects and multi-track projects; capture actual interactions. Keep one
canonical keyframe store. Do not enable unverified reverse/hold/retime operations. Proceed
toward Focus Clip and dockable Value/Speed Graph after this milestone. All original UX and
performance requirements remain binding. Push verified checkpoints; no merge or deployment.
