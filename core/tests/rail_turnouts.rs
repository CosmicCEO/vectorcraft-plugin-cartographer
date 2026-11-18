//! The turnout geometry against NMRA RP-12.35 (N scale straight switch turnout), frog numbers 4 to 20.
use cartographer_core::rail_standards::*;
use cartographer_core::turnout::Frog;
use cartographer_core::turnout_table::ROWS;

const INCH: f64 = 25.4;

#[test]
fn the_table_covers_frogs_4_to_20_in_order() {
    assert_eq!(ROWS.len(), 17);
    for (i, r) in ROWS.iter().enumerate() {
        assert_eq!(r.frog as usize, 4 + i);
    }
}

#[test]
fn frog_angle_rule_matches_every_listed_angle() {
    for r in ROWS {
        let rule = 2.0 * (0.5 / r.frog as f64).atan().to_degrees();
        assert!((rule - r.frog_angle_deg).abs() < 0.0011, "No. {}: {rule} vs {}", r.frog, r.frog_angle_deg);
    }
}

#[test]
fn the_route_reaches_one_gauge_at_the_listed_lead() {
    for r in ROWS {
        let f = Frog::new(r.frog);
        let at_lead = f.lateral(f.lead);
        assert!((at_lead - GAUGE).abs() < 0.2, "No. {}: the route is {at_lead} mm out at the lead, want one gauge, {GAUGE}", r.frog);
    }
}

#[test]
fn the_route_passes_through_the_listed_gage_line_points() {
    for r in ROWS {
        let f = Frog::new(r.frog);
        for [x, y] in r.gage_points {
            let got = f.lateral(x * INCH);
            // The table gives the gage line of the closure rail, the route here is the centre line: they differ by half a gauge times (1 / cos - 1).
            assert!((got - y * INCH).abs() < 0.2, "No. {} at {x} in: {got} mm vs {} mm", r.frog, y * INCH);
        }
    }
}

#[test]
fn number_six_in_millimetres() {
    let f = Frog::new(6);
    assert!((f.lead - 90.02).abs() < 0.01, "lead {}", f.lead); // 3.544 in
    assert!((f.radius - 475.1).abs() < 0.1, "radius {}", f.radius); // 18.704 in
    assert!((f.switch_rail - 28.43).abs() < 0.01); // 1.119 in
    assert!((f.alpha.to_degrees() - 9.527).abs() < 1e-9);
}

#[test]
fn lateral_and_u_at_are_inverses_and_continuous() {
    let f = Frog::new(8);
    let e = f.end();
    for u in [1.0, 20.0, 44.0, e[0] - 0.01, e[0] + 0.01, e[0] + 30.0, 200.0] {
        let y = f.lateral(u);
        assert!((f.u_at(y) - u).abs() < 1e-6, "u {u}");
    }
    assert!((f.lateral(e[0] - 1e-9) - f.lateral(e[0] + 1e-9)).abs() < 1e-6, "no step where the arc ends");
    let heel = f.switch_rail * f.beta.cos();
    assert!((f.lateral(heel - 1e-9) - f.lateral(heel + 1e-9)).abs() < 1e-6, "no step at the heel");
}

#[test]
fn the_crossover_basis_is_the_tables_13_feet() {
    assert!((TRACK_CENTERS - 0.975 * INCH).abs() < 1e-9);
    assert!((FOOT - 0.075 * INCH).abs() < 1e-9);
}

#[test]
fn the_tables_crossover_column_follows_from_its_other_columns() {
    // RP-12.35 line (26): the crossover track distance, point of frog to point of frog, is the distance between the two intersections of
    // centre lines (spacing / sin a) less the point-of-frog-to-intersection distance (24) at each end.
    for r in ROWS {
        let a = r.frog_angle_deg.to_radians();
        let worked = 0.975 / a.sin() - 2.0 * r.pf_to_icl;
        // The angle is printed to 0.001 degree, which moves a 16 inch distance by about 0.003 inch.
        assert!((worked - r.crossover_crossover).abs() < 0.003, "No. {}: {worked} vs {}", r.frog, r.crossover_crossover);
    }
}

#[test]
fn the_closing_of_the_closure_rails_lies_just_short_of_the_point_of_frog() {
    // The straight run at the frog angle meets the stock line at an intersection of centre lines; the point of frog is pf_to_icl further on,
    // and the closure rails end a frog toe length (a few tenths of an inch) before it.
    for r in ROWS {
        let f = Frog::new(r.frog);
        let closure_to_icl = f.end()[1] / f.alpha.sin();
        let gap = r.pf_to_icl * INCH - closure_to_icl;
        assert!(gap > 0.0 && gap < 1.2 * INCH, "No. {}: the closure ends {gap} mm short of the point of frog", r.frog);
    }
}
