# V7.5 real Chrome hierarchy captures

Verified source: `7ad34e1b`. These are actual FilmCraft Rust/egui screenshots with the original
procedural multi-track demo. Main viewport 1656×894; narrow viewport 1280×894.

| State | Dark | Light |
| --- | --- | --- |
| collapsed | [Capture](hierarchy-collapsed-dark.png) | [Capture](hierarchy-collapsed-light.png) |
| effect-headers | [Capture](hierarchy-effect-headers-dark.png) | [Capture](hierarchy-effect-headers-light.png) |
| motion | [Capture](hierarchy-motion-dark.png) | [Capture](hierarchy-motion-light.png) |
| opacity-keys | [Capture](hierarchy-opacity-keys-dark.png) | [Capture](hierarchy-opacity-keys-light.png) |
| selective-effects | [Capture](hierarchy-selective-effects-dark.png) | [Capture](hierarchy-selective-effects-light.png) |
| another-clip | [Capture](hierarchy-another-clip-dark.png) | [Capture](hierarchy-another-clip-light.png) |
| narrow | [Capture](hierarchy-narrow-dark.png) | [Capture](hierarchy-narrow-light.png) |
| short | [Capture](hierarchy-short-dark.png) | [Capture](hierarchy-short-light.png) |

Additional captures: [same-track disclosures](hierarchy-same-track.png), [audio disclosure](hierarchy-audio.png).

The previous fixed inspector is preserved in [V7.4 captures](../v7-inline/inline-animation-dark.png).

Compared with the HTML workflow, clips now disclose compact effect headers, then properties and
actual key lanes. Rows use FilmCraft tokens and its existing track header/ruler, rather than the
prototype artwork or mock data. Opening anchors the clip; deliberate shared track scrolling can
move it out of view to reach lower groups. Short views scroll the hierarchy instead of nesting
an inspector scrollbar. Focus/Graph/filters and broader domain editing remain pending.
