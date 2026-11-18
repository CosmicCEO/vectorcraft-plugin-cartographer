//! Turnout geometry in a local frame: x runs along the stock (straight) track, y is to its left, the point of switch is at the origin.
//!
//! The shape follows NMRA RP-12.35 (N scale straight switch turnout, `turnout_table.rs`): the switch rail is straight at the heel angle for
//! its length, then a closure arc of the listed radius turns to the frog angle, then the route runs straight. The table lists the gage line
//! of the closure rail; the centre line here has the same shape (a rail half a gauge to its side differs by less than 0.15 mm at the
//! points the table gives, checked in `core/tests/turnouts.rs`).

use crate::geom::P;
use crate::turnout_table::ROWS;

const INCH: f64 = 25.4;

#[derive(Clone, Copy, Debug)]
pub struct Frog {
    pub n: u32,
    /// Frog angle, radians.
    pub alpha: f64,
    /// Switch heel angle, radians.
    pub beta: f64,
    /// Switch rail length, mm.
    pub switch_rail: f64,
    /// Closure curved rail radius, mm.
    pub radius: f64,
    /// Distance from the point of switch to the theoretical point of frog, mm.
    pub lead: f64,
}

impl Frog {
    pub const MIN: u32 = 4;
    pub const MAX: u32 = 20;

    /// The turnout of frog number `n` (clamped to the 4 to 20 the NMRA table lists).
    pub fn new(n: u32) -> Frog {
        let n = n.clamp(Frog::MIN, Frog::MAX);
        let r = ROWS[(n - Frog::MIN) as usize];
        Frog { n, alpha: r.frog_angle_deg.to_radians(), beta: r.heel_angle_deg.to_radians(), switch_rail: r.switch_rail * INCH, radius: r.radius * INCH, lead: r.lead * INCH }
    }

    /// End of the switch rail: (x, y) of its heel.
    fn heel(&self) -> P {
        [self.switch_rail * self.beta.cos(), self.switch_rail * self.beta.sin()]
    }

    /// Where the closure arc ends and the straight run at the frog angle begins: (x, y) from the point of switch.
    pub fn end(&self) -> P {
        let h = self.heel();
        [h[0] + self.radius * (self.alpha.sin() - self.beta.sin()), h[1] + self.radius * (self.beta.cos() - self.alpha.cos())]
    }

    /// Centre of the closure arc.
    fn centre(&self) -> P {
        let h = self.heel();
        [h[0] - self.radius * self.beta.sin(), h[1] + self.radius * self.beta.cos()]
    }

    /// How far, sideways, the route has left the stock line at the end of the closure arc: x sin(a) - y cos(a), the distance the route
    /// is from where it would be if it had run straight at the frog angle from the point of switch.
    pub fn q(&self) -> f64 {
        let e = self.end();
        e[0] * self.alpha.sin() - e[1] * self.alpha.cos()
    }

    /// Sideways distance of the diverging route from the stock line, `u` mm past the point of switch.
    pub fn lateral(&self, u: f64) -> f64 {
        let (h, e, c) = (self.heel(), self.end(), self.centre());
        if u <= 0.0 {
            0.0
        } else if u <= h[0] {
            u * self.beta.tan()
        } else if u <= e[0] {
            c[1] - (self.radius * self.radius - (u - c[0]).powi(2)).max(0.0).sqrt()
        } else {
            e[1] + (u - e[0]) * self.alpha.tan()
        }
    }

    /// The `u` at which the diverging route is `y` mm from the stock line.
    pub fn u_at(&self, y: f64) -> f64 {
        let (h, e, c) = (self.heel(), self.end(), self.centre());
        if y <= h[1] {
            y / self.beta.tan()
        } else if y <= e[1] {
            c[0] + (self.radius * self.radius - (c[1] - y).powi(2)).max(0.0).sqrt()
        } else {
            e[0] + (y - e[1]) / self.alpha.tan()
        }
    }

    /// Distance from the point of switch to the theoretical point of frog, where the two inner rails cross.
    pub fn frog_point(&self) -> f64 {
        self.lead
    }

    /// Walks the diverging route from the pen's position and heading: the switch rail, then the closure arc. `sign` is +1 to diverge to the
    /// left, -1 to the right. The pen ends turned by the frog angle, ready to run straight.
    pub fn diverge(&self, pen: &mut Pen, sign: f64, out: &mut Vec<P>) {
        pen.h += sign * self.beta;
        pen.line(self.switch_rail, out);
        pen.arc(self.radius, sign * (self.alpha - self.beta), out);
    }
}

impl Frog {
    /// Walks the end of a crossover: the diverging route traversed the other way, from the straight run at the frog angle back to the
    /// point of switch on the second track (arriving at the heel angle, as the real switch rail does). `sign` is the side the crossover
    /// has been rising toward, +1 left.
    pub fn converge(&self, pen: &mut Pen, sign: f64, out: &mut Vec<P>) {
        pen.arc(self.radius, -sign * (self.alpha - self.beta), out);
        pen.line(self.switch_rail, out);
    }

    /// Length of the straight run at the frog angle between the two leads of a crossover between tracks `spacing` apart, or None if the
    /// two leads would not fit.
    pub fn crossover_run(&self, spacing: f64) -> Option<f64> {
        let run = (spacing - 2.0 * self.end()[1]) / self.alpha.sin();
        (run > 0.0).then_some(run)
    }

    /// The centre line of a crossover starting at the point of switch (x0, y0), rising to the left (`sign` +1) or right (-1) by `spacing`.
    pub fn crossover(&self, x0: f64, y0: f64, sign: f64, spacing: f64) -> Option<Vec<P>> {
        let run = self.crossover_run(spacing)?;
        let mut pts = vec![[x0, y0]];
        let mut pen = Pen::new([x0, y0], 0.0);
        self.diverge(&mut pen, sign, &mut pts);
        pen.line(run, &mut pts);
        self.converge(&mut pen, sign, &mut pts);
        Some(pts)
    }
}

/// A pen that walks straight lines and arcs and records the points it passes. Heading is in radians, counter-clockwise (left) positive.
pub struct Pen {
    pub p: P,
    pub h: f64,
}

impl Pen {
    pub fn new(p: P, h: f64) -> Pen {
        Pen { p, h }
    }

    pub fn line(&mut self, len: f64, out: &mut Vec<P>) {
        self.p = [self.p[0] + len * self.h.cos(), self.p[1] + len * self.h.sin()];
        out.push(self.p);
    }

    /// An arc of `radius` turning by `turn` radians (positive turns left), sampled about every 2 degrees.
    pub fn arc(&mut self, radius: f64, turn: f64, out: &mut Vec<P>) {
        let steps = ((turn.abs() / 0.035).ceil() as usize).max(2);
        let sgn = turn.signum();
        let c = [self.p[0] - sgn * radius * self.h.sin(), self.p[1] + sgn * radius * self.h.cos()];
        let r0 = [self.p[0] - c[0], self.p[1] - c[1]];
        for k in 1..=steps {
            let a = turn * k as f64 / steps as f64;
            self.p = [c[0] + r0[0] * a.cos() - r0[1] * a.sin(), c[1] + r0[0] * a.sin() + r0[1] * a.cos()];
            out.push(self.p);
        }
        self.h += turn;
    }
}
