//! This module defines the core data structures and types used throughout the Turing Machine
//! simulator, including program representation, transitions, execution results, and error types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::Rule;

/// The default blank symbol used on the Turing Machine tape.
pub const DEFAULT_BLANK_SYMBOL: char = ' ';
/// A special input symbol used in program definitions to represent the blank symbol.
pub const INPUT_BLANK_SYMBOL: char = '_';
/// The maximum allowed size for a Turing Machine program in bytes.
pub const MAX_PROGRAM_SIZE: usize = 65536; // 64KB
/// The maximum number of steps to execute before halting.
pub const MAX_EXECUTION_STEPS: usize = 10000;

/// Represents a Turing Machine program, supporting both single and multi-tape configurations.
///
/// A program defines the initial setup of the machine and its transition rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Program {
    /// The name of the Turing Machine program.
    pub name: String,
    /// Execution mode of the simulator.
    pub mode: Mode,
    /// The initial state of the Turing Machine.
    pub initial_state: String,
    /// A vector of strings, where each string represents the initial content of a tape.
    pub tapes: Vec<String>,
    /// A vector of head positions, one for each tape, indicating the initial position of the head.
    pub heads: Vec<usize>,
    /// The blank symbol used on the tapes.
    pub blank: char,
    /// A hash map representing the transition rules. The key is the current state,
    /// and the value is a vector of possible `Transition`s from that state.
    pub rules: HashMap<String, Vec<Transition>>,
}

/// The execution mode for a Turing Machine program.
///
/// Controls how the simulator handles undefined transitions:
/// - `Normal` (default): undefined transitions halt the machine normally (faithful to TM theory).
/// - `Strict`: undefined transitions trigger an error, useful for debugging or catching missing rules.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    /// Undefined transitions halt normally.
    #[default]
    Normal,
    /// Undefined transitions are treated as errors.
    Strict,
}

impl Program {
    /// Returns the initial content of the first tape as a `String`.
    /// This is a convenience method for single-tape compatibility.
    pub fn initial_tape(&self) -> String {
        self.tapes.first().cloned().unwrap_or_default()
    }

    /// Returns the initial head position of the first tape.
    /// This is a convenience method for single-tape compatibility.
    pub fn head_position(&self) -> usize {
        self.heads.first().cloned().unwrap_or(0)
    }

    /// Checks if the program is configured for a single-tape Turing Machine.
    pub fn is_single_tape(&self) -> bool {
        self.tapes.len() == 1
    }

    pub fn tapes(&self) -> Vec<Vec<char>> {
        self.tapes
            .iter()
            .map(|tape| tape.chars().collect())
            .collect()
    }
}

/// Represents a single transition rule for a Turing Machine.
///
/// A transition defines how the machine behaves when it is in a certain state
/// and reads specific symbols from its tapes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    /// A vector of characters to be read from each tape.
    pub read: Vec<char>,
    /// A vector of characters to be written to each tape.
    pub write: Vec<char>,
    /// A vector of directions for each tape's head to move after the transition.
    pub directions: Vec<Direction>,
    /// The next state the machine transitions to.
    pub next_state: String,
}

/// Represents the possible directions a Turing Machine head can move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Move the head one position to the left.
    Left,
    /// Move the head one position to the right.
    Right,
    /// Keep the head in the same position.
    Stay,
}

/// Represents the outcome of a Turing Machine execution step.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// The machine successfully performed a step and continues execution.
    Continue,
    /// The machine has halted (reached a state with no outgoing transitions).
    Halt(Halt),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Halt {
    /// Halted in a specific state (no outgoing transitions).
    Ok,

    Err(TuringMachineError),
}

/// Details of a rejection outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejection {
    pub state: String,
    pub symbols: Vec<char>,
}

/// Represents various errors that can occur during Turing Machine operations.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TuringMachineError {
    /// Indicates an attempt to transition to an invalid or undefined state.
    #[error("Invalid state: {0}")]
    InvalidState(String),
    /// Indicates that there's no rule defined for a particular set of symbols.
    #[error("No rule defined for state {0} and symbols {1:?}")]
    UndefinedTransition(String, Vec<char>),
    /// Indicates that a tape head attempted to move beyond the defined tape boundaries.
    #[error("Tape boundary exceeded")]
    TapeBoundary,
    /// Indicates an error during the parsing of a Turing Machine program definition.
    #[error("Program parsing error: {0}")]
    ParseError(#[from] Box<pest::error::Error<Rule>>),
    /// Indicates an error during the validation of a Turing Machine program's structure or logic.
    #[error("Program validation error: {0}")]
    ValidationError(String),
    /// Indicates an error related to file system operations, such as reading or writing program files.
    #[error("File error: {0}")]
    FileError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_serialization() {
        let left = Direction::Left;
        let right = Direction::Right;

        let left_json = serde_json::to_string(&left).unwrap();
        let right_json = serde_json::to_string(&right).unwrap();

        assert_eq!(left_json, "\"Left\"");
        assert_eq!(right_json, "\"Right\"");

        let left_deserialized: Direction = serde_json::from_str(&left_json).unwrap();
        let right_deserialized: Direction = serde_json::from_str(&right_json).unwrap();

        assert_eq!(left, left_deserialized);
        assert_eq!(right, right_deserialized);
    }

    #[test]
    fn test_transition_creation() {
        let transition = Transition {
            read: vec!['A'],
            write: vec!['X'],
            directions: vec![Direction::Right],
            next_state: "q1".to_string(),
        };

        assert_eq!(transition.write, vec!['X']);
        assert_eq!(transition.directions, vec![Direction::Right]);
        assert_eq!(transition.next_state, "q1");
    }

    #[test]
    fn test_error_display() {
        let error = TuringMachineError::InvalidState("q0".to_string());

        let error_msg = format!("{}", error);
        assert!(error_msg.contains("Invalid state"));
        assert!(error_msg.contains("q0"));
    }
}
