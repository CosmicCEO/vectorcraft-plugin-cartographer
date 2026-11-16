//! Core of the VectorCraft Cartographer.

pub mod abi;
pub mod geom;
pub mod offset;
pub mod output;
pub mod storage;
pub mod units;
pub mod vcdoc;

/// The plug-in's version (the workspace version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
