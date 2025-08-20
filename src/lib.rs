//! This crate provides the core logic for a Turing Machine simulator.
//! It includes modules for parsing Turing Machine programs, simulating their execution,
//! analyzing program correctness, and managing a collection of predefined programs.

pub mod analyzer;
pub mod encoder;
pub mod loader;
pub mod machine;
pub mod parser;
pub mod programs;
pub mod types;

/// Re-exports the `Rule` enum from the parser module, used by the `pest` grammar.
pub use crate::parser::Rule;
/// Re-exports the `analyze` function and `AnalysisError` enum from the analyzer module.
pub use analyzer::{analyze, AnalysisError};
/// Re-exports the encoding functions from the encoder module.
pub use encoder::{decode, encode};
/// Re-exports the `ProgramLoader` struct from the loader module.
pub use loader::ProgramLoader;
/// Re-exports the `TuringMachine` struct from the machine module.
pub use machine::TuringMachine;
/// Re-exports the `parse` function from the parser module.
pub use parser::parse;
/// Re-exports `ProgramInfo`, `ProgramManager`, and `PROGRAMS` from the programs module.
pub use programs::{ProgramInfo, ProgramManager, PROGRAMS};
/// Re-exports various types related to Turing Machine definition and execution from the types module.
pub use types::{Direction, Program, Step, Transition, TuringMachineError, MAX_PROGRAM_SIZE};
