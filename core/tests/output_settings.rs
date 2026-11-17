use cartographer_core::output::Out;

fn dot(out: &mut Out, x: f64) {
    out.path("dot", &[vec![[x, 0.0], [x + 1.0, 1.0]]], |p| p, [0.0; 3], 1.0, false);
}

#[test]
fn the_default_is_three_decimals() {
    let mut out = Out::new();
    dot(&mut out, 12.3456789);
    let text = out.finish();
    assert!(text.contains("12.346") && !text.contains("12.3457"), "{text}");
}

#[test]
fn precision_five_writes_five_decimals() {
    let mut out = Out::new();
    out.precision(5);
    dot(&mut out, 12.3456789);
    assert!(out.finish().contains("12.34568"));
}

#[test]
fn precision_can_change_between_objects() {
    let mut out = Out::new();
    dot(&mut out, 1.23456789);
    out.precision(5);
    dot(&mut out, 2.23456789);
    out.precision(3);
    dot(&mut out, 3.23456789);
    let text = out.finish();
    assert!(text.contains("1.235") && text.contains("2.23457") && text.contains("3.235"), "{text}");
    assert!(!text.contains("1.2346") && !text.contains("3.2346"), "{text}");
}

#[test]
fn numbers_beyond_four_million_are_written_in_full() {
    let mut out = Out::new();
    dot(&mut out, 5_000_000.123);
    assert!(out.finish().contains("5000000.123"));
}

#[test]
fn a_bad_decimals_value_is_clamped() {
    let mut low = Out::new();
    low.precision(0);
    dot(&mut low, 1.23456789);
    assert!(low.finish().contains("1.235"), "0 behaves as 3");
    let mut high = Out::new();
    high.precision(9);
    dot(&mut high, 1.23456789);
    assert!(high.finish().contains("1.234568"), "9 behaves as 6");
}

fn wobble() -> Vec<[f64; 2]> {
    (0..30).map(|i| [f64::from(i), 0.01 * f64::from(i % 2)]).collect()
}

fn anchors(out: Out) -> usize {
    out.finish().matches("{\"p\":").count()
}

#[test]
fn thinning_is_a_setting() {
    let mut coarse = Out::new();
    coarse.curve("c", &[wobble()], |p| p, [0.0; 3], 1.0);
    let mut fine = Out::new();
    fine.thinning(0.001);
    fine.curve("c", &[wobble()], |p| p, [0.0; 3], 1.0);
    assert_eq!(anchors(coarse), 2, "within 0.12 of the chord is dropped");
    assert!(anchors(fine) > 2, "a tighter tolerance keeps points");
}

#[test]
fn negative_and_zero_still_write() {
    let mut out = Out::new();
    out.precision(5);
    out.path("n", &[vec![[-0.5, 0.0], [-0.00001, 1.0]]], |p| p, [0.0; 3], 1.0, false);
    let text = out.finish();
    assert!(text.contains("[-0.5,0]") && text.contains("[-0.00001,1]"), "{text}");
}
