#[path = "../src/decoder_policy.rs"]
mod policy;

#[test]
fn byte_eviction_preserves_the_requested_frame_and_handles_extreme_timestamps() {
    assert_eq!(policy::eviction_candidate([12, 13, 14, 15].into_iter(), 12), Some(15));
    assert_eq!(policy::eviction_candidate([12].into_iter(), 12), None);
    assert_eq!(policy::eviction_candidate([i64::MIN, 0, i64::MAX].into_iter(), 0), Some(i64::MIN));
    let mut frames = vec![100, 101, 102, 103, 104, 105, 106, 107, 108];
    while frames.len() > 3 {
        let victim = policy::eviction_candidate(frames.iter().copied(), 100).unwrap();
        frames.retain(|p| *p != victim);
    }
    assert_eq!(frames, vec![100, 101, 102], "lookahead must not evict the requested 4K picture before it can be returned");
}

#[test]
fn measured_policy_is_limited_to_small_eight_bit_avc_on_the_measured_platform() {
    for profile in [66, 77, 100] {
        assert_eq!(policy::acceleration(None, Some(profile), 1280, 720, true), "prefer-software");
    }
    assert_eq!(policy::acceleration(None, Some(100), 720, 1280, true), "prefer-software");
    for (profile, width, height, platform) in [
        (Some(100), 1280, 720, false),
        (None, 1280, 720, true),
        (Some(110), 1280, 720, true),
        (Some(100), 1920, 1080, true),
        (Some(100), 3840, 2160, true),
        (Some(100), 0, 720, true),
        (Some(100), u32::MAX, u32::MAX, true),
    ] {
        assert_eq!(policy::acceleration(None, profile, width, height, platform), "no-preference");
    }
}

#[test]
fn explicit_decoder_choices_override_the_policy() {
    for (choice, expected) in [("hardware", "prefer-hardware"), ("software", "prefer-software"), ("default", "no-preference")] {
        assert_eq!(policy::acceleration(Some(choice), Some(100), 1280, 720, true), expected);
    }
}
