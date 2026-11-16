//! Core of the VectorCraft Cartographer.

pub mod abi;
pub mod builder;
pub mod crossing;
pub mod crosssection;
pub mod embank;
pub mod geom;
pub mod hub;
pub mod junction;
pub mod library;
pub mod materials;
pub mod offset;
pub mod output;
pub mod roads;
pub mod scenarios;
pub mod schema;
pub mod standards;
pub mod storage;
pub mod transition;
pub mod units;
pub mod vcdoc;
pub mod water;
pub mod wet;

/// The plug-in's version (the workspace version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
