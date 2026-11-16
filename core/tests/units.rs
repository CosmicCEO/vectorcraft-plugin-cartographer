use cartographer_core::units::{N_GAUGE_M, POINTS_PER_METRE, TargetScale, ground_m_for_print_mm};

#[test]
fn print_millimetres_become_ground_metres() {
    assert_eq!(POINTS_PER_METRE, 1.0);
    assert_eq!(ground_m_for_print_mm(0.25, 5000), 1.25);
    // 9 mm of N gauge at 1:160 is 1.44 m of ground.
    assert!((ground_m_for_print_mm(9.0, 160) - N_GAUGE_M).abs() < 1e-9);
}

#[test]
fn true_and_ratios_parse() {
    assert_eq!(TargetScale::parse("true"), Ok(TargetScale::True));
    assert_eq!(TargetScale::parse(" TRUE "), Ok(TargetScale::True));
    assert_eq!(TargetScale::parse("1:5000"), Ok(TargetScale::Ratio(5000)));
    assert_eq!(TargetScale::parse(" 1 : 160 "), Ok(TargetScale::Ratio(160)));
}

#[test]
fn minimum_widths_follow_the_scale() {
    assert_eq!(TargetScale::Ratio(5000).min_width_m(0.25), 1.25);
    assert_eq!(TargetScale::True.min_width_m(0.25), 0.0);
}

#[test]
fn text_round_trips() {
    for text in ["true", "1:5000", "1:160", "1:1"] {
        assert_eq!(TargetScale::parse(text).unwrap().to_text(), text);
    }
}

#[test]
fn bad_scales_are_plain_messages() {
    for bad in ["", "abc", "0:5", "1:0", "1:-5", "1:2.5", "1:1000001", "1:", ":5"] {
        let e = TargetScale::parse(bad).unwrap_err();
        assert!(e.contains("1:N"), "{bad:?} gave {e:?}");
    }
}
