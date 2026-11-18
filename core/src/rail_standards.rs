//! Named N-scale dimensions (millimetres, 1:160). Each cites an entry in `library/standards-rail.json`; `core/tests/rail_standards.rs` checks they agree.

/// entry `scale`, verified (S-3.2)
pub const SCALE: f64 = 160.0;
/// entry `gauge`: the target of S-3.2, 0.355 in (verified); the limits are 0.353 to 0.359 in
pub const GAUGE: f64 = 9.017;
/// entry `rail`, unverified
pub const RAIL_HEAD: f64 = 0.6;
/// entry `tie`, unverified
pub const TIE_LENGTH: f64 = 16.2;
pub const TIE_WIDTH: f64 = 1.4;
pub const TIE_PITCH: f64 = 3.1;
/// Prototype feet per millimetre at 1:160 divisor: model mm = feet * 304.8 / SCALE. entries `tangent-centers`, `crossover-basis`
pub const FOOT: f64 = 304.8 / SCALE;
/// 13 ft centres, the basis of the RP-12.x crossover tables (verified), scaled
pub const TRACK_CENTERS: f64 = 13.0 * FOOT;
/// entry `radius-frog`: the smallest radius RP-11 lists for N locomotive-hauled equipment (verified)
pub const MIN_RADIUS: f64 = 191.0;
/// Document points per millimetre of model. A document point is 1 metre of ground, and 1 mm of model at 1:160 is 160 mm = 0.16 m of ground.
pub const PT_PER_MM: f64 = 0.16;
