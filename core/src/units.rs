//! Units and the target print scale. 1 document point is 1 metre of ground, so VectorCraft's print scaling does the 1:160 or A0 fitting.

/// Document points per metre of ground.
pub const POINTS_PER_METRE: f64 = 1.0;

/// N gauge (9 mm at 1:160) in metres of ground.
pub const N_GAUGE_M: f64 = 1.44;

const BAD_SCALE: &str = "target print scale must be true or 1:N (N a whole number from 1 to 1000000)";

/// Metres of ground that `print_mm` of paper covers at 1:`ratio`.
pub fn ground_m_for_print_mm(print_mm: f64, ratio: u32) -> f64 {
    print_mm * f64::from(ratio) / 1000.0
}

/// The scale the owner intends to print at: true proportions, or 1:N (then small things get a minimum print size).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetScale {
    True,
    Ratio(u32),
}

impl TargetScale {
    /// `true`, or `1:N` with N a whole number from 1 to 1,000,000 (spaces around the colon are allowed).
    pub fn parse(text: &str) -> Result<TargetScale, String> {
        let t = text.trim().to_ascii_lowercase();
        if t == "true" {
            return Ok(TargetScale::True);
        }
        let bad = || BAD_SCALE.to_string();
        let (one, n) = t.split_once(':').ok_or_else(bad)?;
        if one.trim() != "1" {
            return Err(bad());
        }
        match n.trim().parse::<u32>() {
            Ok(n) if (1..=1_000_000).contains(&n) => Ok(TargetScale::Ratio(n)),
            _ => Err(bad()),
        }
    }

    /// `"true"` or `"1:5000"`.
    pub fn to_text(&self) -> String {
        match self {
            TargetScale::True => "true".to_string(),
            TargetScale::Ratio(n) => format!("1:{n}"),
        }
    }

    /// The smallest width in metres of ground that prints as `print_mm` on paper (0 when drawing true).
    pub fn min_width_m(&self, print_mm: f64) -> f64 {
        match self {
            TargetScale::True => 0.0,
            TargetScale::Ratio(n) => ground_m_for_print_mm(print_mm, *n),
        }
    }
}

/// The narrowest a band prints, millimetres on paper (the seed materials use 0.25 to 0.3).
pub const MIN_BAND_PRINT_MM: f64 = 0.3;
/// The thinnest a road stroke prints, millimetres on paper.
pub const MIN_STROKE_PRINT_MM: f64 = 0.12;
