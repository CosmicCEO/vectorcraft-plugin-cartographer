//! Generated from library/standards-rail.json (table rp12_35) by `node scripts/standards.mjs rs`. Do not edit by hand.
//! NMRA RP-12.35, N Scale Straight Switch Turnout (revised February 2015). Inches and degrees, as printed.

#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub frog: u32,
    /// (2) switch rail length.
    pub switch_rail: f64,
    /// (5) switch heel angle.
    pub heel_angle_deg: f64,
    /// (8) lead from the point of switch to the theoretical point of frog.
    pub lead: f64,
    /// (11) closure curved rail radius.
    pub radius: f64,
    /// (18) frog angle.
    pub frog_angle_deg: f64,
    /// (13, 12), (15, 14), (17, 16): distance along and offset of three points on the curved closure rail's gage line.
    pub gage_points: [[f64; 2]; 3],
    /// (24) point of frog to intersection of centre lines.
    pub pf_to_icl: f64,
    /// (26) crossover track distance, point of frog to point of frog, for tracks 0.975 in (13 prototype feet) apart.
    pub crossover_crossover: f64,
}

pub const ROWS: [Row; 17] = [
    Row { frog: 4, switch_rail: 1.11, heel_angle_deg: 3.211, lead: 2.806, radius: 7.333, frog_angle_deg: 14.25, gage_points: [[1.456, 0.09], [1.802, 0.134], [2.148, 0.195]], pf_to_icl: 1.412, crossover_crossover: 1.137 },
    Row { frog: 5, switch_rail: 1.104, heel_angle_deg: 3.228, lead: 3.175, radius: 12.245, frog_angle_deg: 11.421, gage_points: [[1.536, 0.094], [1.968, 0.142], [2.401, 0.205]], pf_to_icl: 1.765, crossover_crossover: 1.394 },
    Row { frog: 6, switch_rail: 1.119, heel_angle_deg: 3.185, lead: 3.544, radius: 18.704, frog_angle_deg: 9.527, gage_points: [[1.632, 0.098], [2.145, 0.148], [2.658, 0.212]], pf_to_icl: 2.118, crossover_crossover: 1.655 },
    Row { frog: 7, switch_rail: 1.677, heel_angle_deg: 2.125, lead: 4.634, radius: 23.823, frog_angle_deg: 8.171, gage_points: [[2.301, 0.094], [2.926, 0.141], [3.55, 0.206]], pf_to_icl: 2.471, crossover_crossover: 1.918 },
    Row { frog: 8, switch_rail: 1.74, heel_angle_deg: 2.048, lead: 5.075, radius: 31.944, frog_angle_deg: 7.153, gage_points: [[2.448, 0.095], [3.156, 0.144], [3.864, 0.209]], pf_to_icl: 2.824, crossover_crossover: 2.182 },
    Row { frog: 9, switch_rail: 1.749, heel_angle_deg: 2.037, lead: 5.394, radius: 40.333, frog_angle_deg: 6.36, gage_points: [[2.506, 0.096], [3.264, 0.145], [4.021, 0.207]], pf_to_icl: 3.177, crossover_crossover: 2.448 },
    Row { frog: 10, switch_rail: 1.843, heel_angle_deg: 1.933, lead: 5.875, radius: 51.524, frog_angle_deg: 5.725, gage_points: [[2.693, 0.098], [3.542, 0.148], [4.392, 0.211]], pf_to_icl: 3.53, crossover_crossover: 2.714 },
    Row { frog: 11, switch_rail: 2.301, heel_angle_deg: 1.549, lead: 6.855, radius: 60.672, frog_angle_deg: 5.205, gage_points: [[3.266, 0.096], [4.232, 0.145], [5.197, 0.21]], pf_to_icl: 3.883, crossover_crossover: 2.981 },
    Row { frog: 12, switch_rail: 2.325, heel_angle_deg: 1.532, lead: 7.212, radius: 73.038, frog_angle_deg: 4.772, gage_points: [[3.356, 0.097], [4.386, 0.146], [5.416, 0.21]], pf_to_icl: 4.236, crossover_crossover: 3.248 },
    Row { frog: 13, switch_rail: 2.365, heel_angle_deg: 1.506, lead: 7.599, radius: 87.546, frog_angle_deg: 4.405, gage_points: [[3.471, 0.098], [4.576, 0.148], [5.681, 0.212]], pf_to_icl: 4.589, crossover_crossover: 3.516 },
    Row { frog: 14, switch_rail: 2.41, heel_angle_deg: 1.478, lead: 7.986, radius: 103.626, frog_angle_deg: 4.091, gage_points: [[3.589, 0.099], [4.768, 0.15], [5.947, 0.214]], pf_to_icl: 4.942, crossover_crossover: 3.783 },
    Row { frog: 15, switch_rail: 3.193, heel_angle_deg: 1.116, lead: 9.431, radius: 112.621, frog_angle_deg: 3.818, gage_points: [[4.519, 0.096], [5.845, 0.145], [7.171, 0.21]], pf_to_icl: 5.295, crossover_crossover: 4.051 },
    Row { frog: 16, switch_rail: 3.201, heel_angle_deg: 1.113, lead: 9.8, radius: 131.38, frog_angle_deg: 3.58, gage_points: [[4.614, 0.097], [6.026, 0.147], [7.439, 0.213]], pf_to_icl: 5.648, crossover_crossover: 4.319 },
    Row { frog: 17, switch_rail: 3.23, heel_angle_deg: 1.103, lead: 10.158, radius: 149.407, frog_angle_deg: 3.369, gage_points: [[4.705, 0.098], [6.181, 0.148], [7.657, 0.213]], pf_to_icl: 6.001, crossover_crossover: 4.587 },
    Row { frog: 18, switch_rail: 3.261, heel_angle_deg: 1.092, lead: 10.516, radius: 168.858, frog_angle_deg: 3.182, gage_points: [[4.8, 0.099], [6.338, 0.149], [7.876, 0.213]], pf_to_icl: 6.354, crossover_crossover: 4.856 },
    Row { frog: 19, switch_rail: 3.311, heel_angle_deg: 1.076, lead: 10.925, radius: 192.183, frog_angle_deg: 3.015, gage_points: [[4.935, 0.1], [6.56, 0.151], [8.184, 0.216]], pf_to_icl: 6.707, crossover_crossover: 5.124 },
    Row { frog: 20, switch_rail: 3.364, heel_angle_deg: 1.059, lead: 11.334, radius: 217.256, frog_angle_deg: 2.864, gage_points: [[5.074, 0.101], [6.783, 0.152], [8.493, 0.218]], pf_to_icl: 7.06, crossover_crossover: 5.392 },
];
