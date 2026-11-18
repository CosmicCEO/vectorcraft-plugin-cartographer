use cartographer_core::storage::Style;
use cartographer_core::style::StyleKit;

fn hand(seed: u32, amplitude: f64) -> StyleKit {
    StyleKit::new(Style::HandDrawn, seed, amplitude)
}

#[test]
fn plain_is_the_identity() {
    let k = StyleKit::plain();
    assert!(k.is_plain());
    assert_eq!(k.tone([0.3, 0.6, 0.9]), [0.3, 0.6, 0.9]);
    assert_eq!(k.wobble([12.5, -3.0]), [12.5, -3.0]);
    assert_eq!(k.stroke_factor("gen.road.a.1.line.0"), 1.0);
    assert!(StyleKit::new(Style::MediumContrast, 7, 0.5).is_plain(), "medium-contrast is plain whatever the seed");
    assert!(!hand(1, 0.5).is_plain());
}

#[test]
fn high_contrast_pushes_light_lighter_and_dark_darker() {
    let k = StyleKit::new(Style::HighContrast, 1, 0.0);
    let light = k.tone([0.8, 0.8, 0.8]);
    let dark = k.tone([0.2, 0.2, 0.2]);
    assert!(light.iter().all(|&c| c > 0.8 && c <= 1.0), "{light:?}");
    assert!(dark.iter().all(|&c| c < 0.2 && c >= 0.0), "{dark:?}");
    assert_eq!(k.tone([0.5, 0.5, 0.5]), [0.5, 0.5, 0.5]);
}

#[test]
fn black_white_has_four_grey_tones() {
    let k = StyleKit::new(Style::BlackWhite, 1, 0.0);
    let mut seen: Vec<f64> = Vec::new();
    for i in 0..200 {
        let t = f64::from(i) / 199.0;
        let c = k.tone([t, (t * 7.0) % 1.0, 1.0 - t]);
        assert!(c[0] == c[1] && c[1] == c[2], "{c:?}");
        if !seen.contains(&c[0]) {
            seen.push(c[0]);
        }
    }
    assert_eq!(seen.len(), 4, "{seen:?}");
    assert!(seen.contains(&0.0) && seen.contains(&1.0));
}

#[test]
fn wobble_is_bounded_and_repeatable() {
    let k = hand(1, 0.5);
    for i in 0..50 {
        for j in 0..40 {
            let p = [f64::from(i) * 3.7 - 40.0, f64::from(j) * 2.9 - 30.0];
            let q = k.wobble(p);
            assert!((q[0] - p[0]).abs() <= 0.5 && (q[1] - p[1]).abs() <= 0.5, "{p:?} -> {q:?}");
            assert_eq!(q, k.wobble(p));
        }
    }
}

#[test]
fn wobble_is_continuous() {
    let k = hand(3, 0.5);
    for i in 0..200 {
        let p = [f64::from(i) * 1.37, f64::from(i) * 0.71];
        let (a, b) = (k.wobble(p), k.wobble([p[0] + 0.01, p[1]]));
        assert!((b[0] - a[0] - 0.01).abs() < 0.02 && (b[1] - a[1]).abs() < 0.02, "{p:?}: {a:?} {b:?}");
    }
}

#[test]
fn wobble_depends_on_the_seed() {
    assert_ne!(hand(1, 0.5).wobble([10.0, 10.0]), hand(2, 0.5).wobble([10.0, 10.0]));
}

#[test]
fn wobble_is_only_for_hand_drawn() {
    for s in [Style::BlackWhite, Style::HighContrast, Style::MediumContrast] {
        assert_eq!(StyleKit::new(s, 1, 0.5).wobble([10.0, 10.0]), [10.0, 10.0]);
    }
    assert_eq!(hand(1, 0.0).wobble([10.0, 10.0]), [10.0, 10.0], "no amplitude, no wobble");
}

#[test]
fn stroke_factor_is_stable_per_name_and_bounded() {
    let k = hand(1, 0.5);
    let names: Vec<String> = (0..20).map(|i| format!("gen.road.a.{i}.line.0")).collect();
    let factors: Vec<f64> = names.iter().map(|n| k.stroke_factor(n)).collect();
    assert!(factors.iter().all(|&f| (0.85..=1.15).contains(&f)), "{factors:?}");
    assert_eq!(factors, names.iter().map(|n| k.stroke_factor(n)).collect::<Vec<_>>());
    assert!(factors.windows(2).any(|w| w[0] != w[1]), "the factors differ between names");
    assert_eq!(StyleKit::new(Style::HighContrast, 1, 0.5).stroke_factor(&names[0]), 1.0);
}
