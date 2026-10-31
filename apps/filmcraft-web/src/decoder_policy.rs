//! Conservative policy for the current CPU-backed WebCodecs-to-WASM frame path.
//! Prefer software only on the small-AVC/Intel/Windows Chrome combination measured locally.

/// Retain the requested picture while bounding decoded bytes and lookahead.
/// Unsigned distance also handles hostile presentation times without overflow.
pub fn eviction_candidate(frames: impl Iterator<Item = i64>, wanted: i64) -> Option<i64> {
    frames.filter(|p| *p != wanted).max_by_key(|p| p.abs_diff(wanted))
}
pub fn acceleration(explicit: Option<&str>, avc_profile: Option<u8>, width: u32, height: u32, measured_platform: bool) -> &'static str {
    match explicit {
        Some("software") => "prefer-software",
        Some("hardware") => "prefer-hardware",
        Some("default") => "no-preference",
        _ if measured_platform
            && matches!(avc_profile, Some(66 | 77 | 100))
            && width > 0
            && height > 0
            && width.max(height) <= 1280
            && u64::from(width) * u64::from(height) <= 1280 * 720 =>
        {
            "prefer-software"
        }
        _ => "no-preference",
    }
}
