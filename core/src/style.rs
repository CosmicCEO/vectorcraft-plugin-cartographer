//! How the document is painted: colours, a hand-drawn displacement and a per-object stroke factor (design 6.4). Medium-contrast is the plain
//! kit, which changes nothing. The displacement is a function of position only, so two objects that share a point move to the same place and
//! a join stays closed (long edges are cut into pieces first, see `densify`); it never reaches the lines you drew (`Out` does not pass them through it).

use crate::storage::Style;

/// Grey tones of the black-and-white style (black, two greys, white).
const TONES: [f64; 4] = [0.0, 0.4, 0.75, 1.0];

/// The wander changes slowly: one noise cell is this many times the amplitude (a gentle hand, not a tremor).
const CELL_PER_AMPLITUDE: f64 = 48.0;
/// A long edge is cut into pieces of this many amplitudes (a quarter of a cell) before it is displaced, so it follows the wander to within
/// about 5% of the amplitude and a point that lies along it stays on it.
const STEP_PER_AMPLITUDE: f64 = 12.0;
/// Most pieces one edge is cut into (a guard for absurd lengths).
const MAX_PIECES: f64 = 5000.0;

#[derive(Clone, Copy, Debug)]
pub struct StyleKit {
    style: Style,
    seed: u32,
    /// Hand-drawn only: the largest displacement along each axis, metres (0: none).
    amplitude: f64,
}

impl StyleKit {
    pub fn new(style: Style, seed: u32, amplitude: f64) -> StyleKit {
        let amplitude = if amplitude.is_finite() && amplitude > 0.0 { amplitude } else { 0.0 };
        StyleKit { style, seed, amplitude }
    }

    /// Medium-contrast: every method returns its input.
    pub fn plain() -> StyleKit {
        StyleKit { style: Style::MediumContrast, seed: 1, amplitude: 0.0 }
    }

    pub fn is_plain(&self) -> bool {
        self.style == Style::MediumContrast
    }

    /// Does this kit move points?
    pub fn wobbles(&self) -> bool {
        self.style == Style::HandDrawn && self.amplitude > 0.0
    }

    /// The colour as this style paints it.
    pub fn tone(&self, rgb: [f64; 3]) -> [f64; 3] {
        match self.style {
            Style::HighContrast => rgb.map(|c| ((c - 0.5) * 1.6 + 0.5).clamp(0.0, 1.0)),
            Style::BlackWhite => {
                let l = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
                let t = TONES.iter().copied().fold(TONES[0], |best, t| if (t - l).abs() < (best - l).abs() { t } else { best });
                [t, t, t]
            }
            Style::HandDrawn | Style::MediumContrast => rgb,
        }
    }

    /// The point displaced by the hand-drawn noise (the point itself in every other style).
    pub fn wobble(&self, p: [f64; 2]) -> [f64; 2] {
        if !self.wobbles() {
            return p;
        }
        let cell = CELL_PER_AMPLITUDE * self.amplitude;
        let (x, y) = (p[0] / cell, p[1] / cell);
        [p[0] + self.amplitude * self.noise(x, y, 0), p[1] + self.amplitude * self.noise(x, y, 1)]
    }

    /// The points of a polyline (or ring, when `closed`) with every long edge cut into short pieces, ready to be displaced: the wander moves
    /// vertices, so an edge with only two of them would stay straight and leave other objects' points that touch it behind.
    pub fn densify(&self, pts: &[[f64; 2]], closed: bool) -> Vec<[f64; 2]> {
        let step = STEP_PER_AMPLITUDE * self.amplitude;
        let mut out = Vec::with_capacity(pts.len() * 2);
        let n = pts.len();
        let edges = if closed && n > 1 && pts[0] != pts[n - 1] { n } else { n.saturating_sub(1) };
        for i in 0..n {
            out.push(pts[i]);
            if i >= edges {
                continue;
            }
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            let pieces = (len / step).ceil().min(MAX_PIECES);
            if pieces >= 2.0 {
                for k in 1..pieces as usize {
                    let t = k as f64 / pieces;
                    out.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
                }
            }
        }
        out
    }

    /// A stable factor for the width of the stroke of the object called `name` (0.85 to 1.15 in the hand-drawn style, else 1).
    pub fn stroke_factor(&self, name: &str) -> f64 {
        if self.style != Style::HandDrawn {
            return 1.0;
        }
        let mut h: u32 = 0x811C_9DC5 ^ self.seed;
        for b in name.bytes() {
            h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
        }
        0.85 + 0.3 * (f64::from(mix(h)) / f64::from(u32::MAX))
    }

    /// Smooth value noise in -1..1: a hash at the corners of each cell, smoothstep between them.
    fn noise(&self, x: f64, y: f64, channel: u32) -> f64 {
        let (fx, fy) = (x.floor(), y.floor());
        let (ix, iy) = (fx as i32, fy as i32);
        let (tx, ty) = (smooth(x - fx), smooth(y - fy));
        let at = |dx: i32, dy: i32| lattice(ix.wrapping_add(dx), iy.wrapping_add(dy), self.seed, channel);
        let top = at(0, 0) + (at(1, 0) - at(0, 0)) * tx;
        let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * tx;
        top + (bottom - top) * ty
    }
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn mix(mut h: u32) -> u32 {
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}

/// A repeatable number in -1..1 for a lattice corner.
fn lattice(ix: i32, iy: i32, seed: u32, channel: u32) -> f64 {
    let h = (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA6B) ^ seed.wrapping_mul(0xC2B2_AE35) ^ channel.wrapping_mul(0x27D4_EB2F);
    f64::from(mix(h)) / f64::from(u32::MAX) * 2.0 - 1.0
}
