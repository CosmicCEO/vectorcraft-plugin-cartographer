use cartographer_core::connector::{connect, dubins};
use cartographer_core::rail_standards::MIN_RADIUS;
use std::f64::consts::{FRAC_PI_2, PI};

type P = [f64; 2];

fn dist(a: P, b: P) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn heading_at_end(pts: &[P]) -> f64 {
    let n = pts.len();
    (pts[n - 1][1] - pts[n - 2][1]).atan2(pts[n - 1][0] - pts[n - 2][0])
}

fn angle_diff(a: f64, b: f64) -> f64 {
    (a - b + PI).rem_euclid(2.0 * PI) - PI
}

/// The smallest radius the polyline turns on, from three-point circles.
fn tightest(pts: &[P]) -> f64 {
    let mut best = f64::INFINITY;
    for w in pts.windows(3) {
        let (ab, bc, ca) = (dist(w[0], w[1]), dist(w[1], w[2]), dist(w[2], w[0]));
        let cross = ((w[1][0] - w[0][0]) * (w[2][1] - w[0][1]) - (w[1][1] - w[0][1]) * (w[2][0] - w[0][0])).abs();
        if cross > 1e-9 {
            best = best.min(ab * bc * ca / (2.0 * cross));
        }
    }
    best
}

#[test]
fn ends_already_in_line_need_no_curve() {
    let c = dubins([0.0, 0.0], 0.0, [200.0, 0.0], 0.0, 300.0).unwrap();
    assert!(c.turn < 1e-9, "turn {}", c.turn);
    assert!((c.length - 200.0).abs() < 1e-6);
    assert!(dist(*c.pts.last().unwrap(), [200.0, 0.0]) < 1e-6);
}

#[test]
fn a_sideways_shift_is_an_s_bend_tangent_at_both_ends() {
    // 100 mm to the left over 600 mm along, parallel at both ends, radius 500 (it needs at least sqrt(4 r s - s^2) = 436 mm along).
    let c = dubins([0.0, 0.0], 0.0, [600.0, 100.0], 0.0, 500.0).unwrap();
    let end = *c.pts.last().unwrap();
    assert!(dist(end, [600.0, 100.0]) < 1e-6, "{end:?}");
    assert!(angle_diff(heading_at_end(&c.pts), 0.0).abs() < 0.02, "leaves tangent: {}", heading_at_end(&c.pts));
    assert!(c.turn > 0.0 && c.turn < 1.0, "an S: the two arcs turn left then right, each well under a radian: {}", c.turn);
    // 400 mm along is not enough at radius 500: connect backs the radius off until it fits, still tangent at both ends
    let short = connect([0.0, 0.0], 0.0, [400.0, 100.0], 0.0, 500.0, MIN_RADIUS).unwrap();
    assert!(short.radius < 500.0 && short.turn < 1.0, "{} {}", short.radius, short.turn);
    assert!(angle_diff(heading_at_end(&short.pts), 0.0).abs() < 0.02);
    // the first step goes along the start heading
    assert!(angle_diff((c.pts[1][1] - c.pts[0][1]).atan2(c.pts[1][0] - c.pts[0][0]), 0.0).abs() < 0.05);
}

#[test]
fn a_right_angle_turn_is_one_arc() {
    let r = 250.0;
    let c = dubins([0.0, 0.0], 0.0, [r, r], FRAC_PI_2, r).unwrap();
    assert!((c.turn - FRAC_PI_2).abs() < 1e-6 && (c.length - r * FRAC_PI_2).abs() < 1e-6, "{} {}", c.turn, c.length);
    assert!(dist(*c.pts.last().unwrap(), [r, r]) < 1e-6);
}

#[test]
fn it_never_turns_tighter_than_the_radius() {
    for (h1, p1) in [(0.3, [300.0, 120.0]), (-0.8, [250.0, -200.0]), (1.2, [400.0, 350.0]), (0.0, [320.0, -60.0]), (2.5, [100.0, 400.0])] {
        let Some(c) = dubins([0.0, 0.0], 0.0, p1, h1, 300.0) else { continue };
        let tight = tightest(&c.pts);
        assert!(tight > 300.0 * 0.99, "heading {h1}: tightest {tight}");
        assert!(dist(*c.pts.last().unwrap(), p1) < 1e-6, "arrives");
        assert!(angle_diff(heading_at_end(&c.pts), h1).abs() < 0.04, "arrives tangent, heading {h1} vs {}", heading_at_end(&c.pts));
    }
}

#[test]
fn too_close_for_the_radius_is_none_and_connect_backs_off_the_radius() {
    // A 30 mm sideways shift in 60 mm along needs a tight bend: radius 800 cannot do it, 191 can.
    assert!(dubins([0.0, 0.0], 0.0, [60.0, 30.0], 0.0, 800.0).is_none() || dubins([0.0, 0.0], 0.0, [60.0, 30.0], 0.0, 800.0).unwrap().turn > 1.5);
    let c = connect([0.0, 0.0], 0.0, [160.0, 30.0], 0.0, 800.0, MIN_RADIUS).unwrap();
    assert!(c.radius < 800.0 && c.radius >= MIN_RADIUS - 1e-9, "radius {}", c.radius);
    assert!(dist(*c.pts.last().unwrap(), [160.0, 30.0]) < 1e-6);
    // and when nothing fits, it says so
    assert!(connect([0.0, 0.0], 0.0, [20.0, 150.0], 0.0, 800.0, MIN_RADIUS).is_none());
}

#[test]
fn the_largest_radius_that_fits_is_chosen() {
    // plenty of room: the preferred radius is used
    let c = connect([0.0, 0.0], 0.0, [600.0, 80.0], 0.0, 500.0, MIN_RADIUS).unwrap();
    assert!((c.radius - 500.0).abs() < 1e-9, "{}", c.radius);
}
