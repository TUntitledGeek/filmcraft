#[path = "../src/decoder_policy.rs"]
mod policy;

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
