//! FUDE lesson core.
//!
//! Everything the lessons app knows about lessons lives here, free of any UI: the on-disk
//! format ([`model`]), reading a library of lesson folders ([`library`]), format checks
//! ([`validate`]), animated guides made of PhotoCraft control calls ([`guide`]), hint
//! ghost shapes ([`hint`]), task assessment from editor state and canvas snapshots
//! ([`assess`]), `.fude` packages ([`package`]) and the lesson editor's file operations
//! ([`authoring`]).
//!
//! The format is described for authors in `docs/lesson-format.md`.

pub mod assess;
pub mod authoring;
pub mod control;
pub mod error;
pub mod guide;
pub mod hint;
pub mod library;
pub mod model;
pub mod package;
pub mod paths;
pub mod validate;

pub use error::{Error, Result};

/// Version written into packages and shown by the CLI.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
