//! Core of the VectorCraft Cartographer.

pub mod abi;
pub mod branch;
pub mod builder;
pub mod connector;
pub mod crossing;
pub mod crosssection;
pub mod embank;
pub mod geom;
pub mod hub;
pub mod joint;
pub mod junction;
pub mod level;
pub mod library;
pub mod materials;
pub mod offset;
pub mod output;
pub mod rail_builder;
pub mod rail_standards;
pub mod roads;
pub mod scenarios;
pub mod schema;
pub mod standards;
pub mod storage;
pub mod style;
pub mod ties;
pub mod tracks;
pub mod transition;
pub mod turnout;
pub mod turnout_table;
pub mod units;
pub mod vcdoc;
pub mod water;
pub mod wet;
pub mod yard;

/// The plug-in's version (the workspace version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
