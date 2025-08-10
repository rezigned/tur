//! This module defines the `TuringMachine` struct, which simulates the behavior of a
//! multi-tape Turing Machine. It handles the machine's state, tape operations, head movements,
//! and execution of transition rules.

use crate::types::{
    Direction, ExecutionResult, ExecutionStep, Program, Transition, TuringMachineError,
    INPUT_BLANK_SYMBOL,
};
use std::collections::HashMap;

/// Represents a multi-tape Turing Machine.
///
/// This struct encapsulates the current state of the Turing Machine, including its
/// current state, the contents of its tapes, the positions of its read/write heads,
/// the blank symbol, and the set of transition rules.
pub struct TuringMachine {
    state: String,
    pub tapes: Vec<Vec<char>>,
    head_positions: Vec<usize>,
    blank_symbol: char,
    rules: HashMap<String, Vec<Transition>>,
    initial_state: String,
    initial_tapes: Vec<Vec<char>>,
    initial_heads: Vec<usize>,
    step_count: usize,
}

impl TuringMachine {
    /// Creates a new `TuringMachine` instance from a given `Program`.
    ///
    /// Initializes the machine with the program's initial state, tapes, head positions,
    /// blank symbol, and transition rules.
    ///
    /// # Arguments
    ///
    /// * `program` - A reference to the `Program` defining the Turing Machine.
    pub fn new(program: &Program) -> Self {
        let tapes: Vec<Vec<char>> = program
            .tapes
            .iter()
            .map(|tape| tape.chars().collect())
            .collect();

        Self {
            state: program.initial_state.clone(),
            tapes: tapes.clone(),
            head_positions: program.heads.clone(),
            blank_symbol: program.blank,
            rules: program.rules.clone(),
            initial_state: program.initial_state.clone(),
            initial_tapes: tapes,
            initial_heads: program.heads.clone(),
            step_count: 0,
        }
    }

    /// Returns the content of the first tape as a `Vec<char>`.
    /// This is a convenience method for single-tape compatibility.
    pub fn get_tape(&self) -> Vec<char> {
        self.tapes.first().cloned().unwrap_or_default()
    }

    /// Returns the head position of the first tape.
    /// This is a convenience method for single-tape compatibility.
    pub fn get_head_position(&self) -> usize {
        self.head_positions.first().cloned().unwrap_or(0)
    }

    /// Returns the symbol currently under the head of the first tape.
    /// If the head is beyond the tape's current length, the blank symbol is returned.
    /// This is a convenience method for single-tape compatibility.
    pub fn get_current_symbol(&self) -> char {
        let tape_index = 0; // First tape for single-tape compatibility
        let head_pos = self.head_positions.get(tape_index).cloned().unwrap_or(0);
        if let Some(tape) = self.tapes.get(tape_index) {
            if head_pos < tape.len() {
                tape[head_pos]
            } else {
                self.blank_symbol
            }
        } else {
            self.blank_symbol
        }
    }

    /// Returns the content of the first tape as a `String`.
    /// This is a convenience method for single-tape compatibility.
    pub fn get_tape_as_string(&self) -> String {
        self.get_tape().iter().collect()
    }

    /// Returns a vector of symbols that have defined transitions from the current state
    /// on the first tape.
    /// This is a convenience method for single-tape compatibility.
    pub fn get_available_transitions(&self) -> Vec<char> {
        // For single-tape compatibility, return symbols that have transitions in current state
        if let Some(transitions) = self.rules.get(&self.state) {
            transitions
                .iter()
                .filter_map(|t| {
                    if t.read.len() == 1 {
                        Some(t.read[0])
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Executes a single step of the Turing Machine's computation.
    ///
    /// This involves reading symbols, writing new symbols, moving heads, and transitioning
    /// to the next state based on the defined rules.
    ///
    /// # Returns
    ///
    /// * `ExecutionResult::Continue` if the machine successfully performs a step.
    /// * `ExecutionResult::Halt` if the machine enters a halt state (no defined transitions).
    /// * `ExecutionResult::Error` if an error occurs, such as no matching transition found.
    pub fn step(&mut self) -> ExecutionResult {
        // Check if we're in a halt state (no transitions defined)
        if !self.rules.contains_key(&self.state) {
            return ExecutionResult::Halt;
        }

        let state_transitions = match self.rules.get(&self.state) {
            Some(transitions) => transitions,
            None => {
                return ExecutionResult::Error(TuringMachineError::InvalidState(self.state.clone()))
            }
        };

        // If no transitions are defined for this state, it's a halt state
        if state_transitions.is_empty() {
            return ExecutionResult::Halt;
        }

        // Ensure all tapes are large enough
        for (i, head_pos) in self.head_positions.iter().enumerate() {
            if *head_pos >= self.tapes[i].len() {
                self.tapes[i].resize(*head_pos + 1, self.blank_symbol);
            }
        }

        // Find matching transition
        let transition = match self.get_current_transition().cloned() {
            Some(t) => t,
            None => {
                return ExecutionResult::Error(TuringMachineError::NoMultiTapeTransition {
                    state: self.state.clone(),
                    symbols: self.get_current_symbols(),
                });
            }
        };

        // Apply transition to all tapes
        for i in 0..self.tapes.len() {
            // Write new symbol
            self.tapes[i][self.head_positions[i]] = if transition.write[i] == INPUT_BLANK_SYMBOL {
                self.blank_symbol
            } else {
                transition.write[i]
            };

            // Move head according to direction
            match transition.directions[i] {
                Direction::Left => {
                    if self.head_positions[i] == 0 {
                        // Extend tape to the left
                        self.tapes[i].insert(0, self.blank_symbol);
                    } else {
                        self.head_positions[i] -= 1;
                    }
                }
                Direction::Right => {
                    self.head_positions[i] += 1;
                    if self.head_positions[i] >= self.tapes[i].len() {
                        self.tapes[i].push(self.blank_symbol);
                    }
                }
                Direction::Stay => {
                    // Head position remains unchanged
                }
            }
        }

        self.state = transition.next_state.clone();
        self.step_count += 1;

        ExecutionResult::Continue
    }

    /// Runs the Turing Machine until it halts or reaches a maximum step count.
    ///
    /// This method records each `ExecutionStep` taken by the machine.
    ///
    /// # Returns
    ///
    /// * `Vec<ExecutionStep>` - A vector of `ExecutionStep`s representing the computation history.
    pub fn run_to_completion(&mut self) -> Vec<ExecutionStep> {
        let mut steps = Vec::new();
        let max_steps = 10000; // Prevent infinite loops

        for _ in 0..max_steps {
            let step = ExecutionStep {
                state: self.state.clone(),
                tapes: self.tapes.clone(),
                head_positions: self.head_positions.clone(),
                symbols_read: self
                    .head_positions
                    .iter()
                    .enumerate()
                    .map(|(i, &pos)| {
                        if pos < self.tapes[i].len() {
                            self.tapes[i][pos]
                        } else {
                            self.blank_symbol
                        }
                    })
                    .collect(),
                transition: None, // Could be enhanced to include the transition taken
            };
            steps.push(step);

            match self.step() {
                ExecutionResult::Continue => continue,
                ExecutionResult::Halt => break,
                ExecutionResult::Error(_) => break,
            }
        }

        steps
    }

    /// Returns the current state of the Turing Machine.
    pub fn get_state(&self) -> &str {
        &self.state
    }

    /// Returns the initial state of the Turing Machine.
    pub fn get_initial_state(&self) -> &str {
        &self.initial_state
    }

    /// Resets the Turing Machine to its initial configuration.
    /// This includes resetting the state, tapes, head positions, and step count.
    pub fn reset(&mut self) {
        self.state = self.initial_state.clone();
        self.tapes = self.initial_tapes.clone();
        self.head_positions = self.initial_heads.clone();
        self.step_count = 0;
    }

    /// Returns the total number of steps executed by the Turing Machine.
    pub fn get_step_count(&self) -> usize {
        self.step_count
    }

    /// Checks if the Turing Machine is currently in a halted state.
    /// A machine is halted if there are no defined transitions for its current state.
    pub fn is_halted(&self) -> bool {
        !self.rules.contains_key(&self.state)
            || self
                .rules
                .get(&self.state)
                .is_none_or(|transitions| transitions.is_empty())
    }

    /// Validates a `Program` before it is used to create a `TuringMachine`.
    ///
    /// This performs various checks, including:
    /// - Ensuring the initial state is defined.
    /// - Checking for empty tapes.
    /// - Verifying that head positions match the number of tapes.
    /// - Confirming that all referenced states in transitions exist.
    /// - Ensuring consistency in tape counts for multi-tape transitions.
    /// - For single-tape programs, it also leverages the `analyzer` module for more in-depth checks.
    ///
    /// # Arguments
    ///
    /// * `program` - A reference to the `Program` to validate.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the program is valid.
    /// * `Err(TuringMachineError::ValidationError)` if any validation rule is violated.
    pub fn validate_program(program: &Program) -> Result<(), TuringMachineError> {
        // Check if initial state exists in rules
        if !program.rules.contains_key(&program.initial_state) {
            return Err(TuringMachineError::ValidationError(format!(
                "Initial state '{}' not defined in transitions",
                program.initial_state
            )));
        }

        // Check for empty tapes
        if program.tapes.is_empty() {
            return Err(TuringMachineError::ValidationError(
                "No tapes defined".to_string(),
            ));
        }

        // Check that head positions match number of tapes
        if program.heads.len() != program.tapes.len() {
            return Err(TuringMachineError::ValidationError(format!(
                "Number of head positions ({}) does not match number of tapes ({})",
                program.heads.len(),
                program.tapes.len()
            )));
        }

        // Validate that all referenced states exist
        for (state, transitions) in &program.rules {
            for transition in transitions {
                if !program.rules.contains_key(&transition.next_state)
                    && transition.next_state != "halt"
                {
                    return Err(TuringMachineError::ValidationError(format!(
                        "State '{}' references undefined state '{}'",
                        state, transition.next_state
                    )));
                }

                // Check that all transitions have consistent tape counts
                if transition.read.len() != program.tapes.len()
                    || transition.write.len() != program.tapes.len()
                    || transition.directions.len() != program.tapes.len()
                {
                    return Err(TuringMachineError::ValidationError(format!(
                        "Transition in state '{}' has inconsistent tape counts",
                        state
                    )));
                }
            }
        }

        // For single-tape programs, use the analyzer module
        if program.is_single_tape() {
            if let Err(errors) = crate::analyzer::analyze(program) {
                // Return the first error for backward compatibility
                if let Some(first_error) = errors.first() {
                    return Err((*first_error).clone().into());
                }
            }
        }

        Ok(())
    }

    /// Returns a slice of the machine's tapes.
    pub fn get_tapes(&self) -> &[Vec<char>] {
        &self.tapes
    }

    /// Returns a slice of the machine's head positions for all tapes.
    pub fn get_head_positions(&self) -> &[usize] {
        &self.head_positions
    }

    /// Returns the content of all tapes as a vector of `String`s.
    pub fn get_tapes_as_strings(&self) -> Vec<String> {
        self.tapes
            .iter()
            .map(|tape| tape.iter().collect())
            .collect()
    }

    /// Returns a vector of symbols currently under each tape's head.
    /// If a head is beyond its tape's current length, the blank symbol is returned for that tape.
    pub fn get_current_symbols(&self) -> Vec<char> {
        self.head_positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                if pos < self.tapes[i].len() {
                    self.tapes[i][pos]
                } else {
                    self.blank_symbol
                }
            })
            .collect()
    }

    /// Finds and returns the matching `Transition` for the current state and symbols under the heads.
    ///
    /// It iterates through the rules for the current state and finds the first transition
    /// whose `read` symbols match the current symbols on the tapes.
    /// Special handling for `INPUT_BLANK_SYMBOL` allows it to match the machine's actual blank symbol.
    ///
    /// # Returns
    ///
    /// * `Some(&Transition)` if a matching transition is found.
    /// * `None` if no matching transition exists.
    pub fn get_current_transition(&self) -> Option<&Transition> {
        match self.rules.get(&self.state) {
            Some(transitions) => {
                let symbols = self.get_current_symbols();

                transitions.iter().find(|t| {
                    if t.read.len() != symbols.len() {
                        return false;
                    }

                    for (i, &symbol) in t.read.iter().enumerate() {
                        // If the transition rule specifies `INPUT_BLANK_SYMBOL`, it matches the program's blank symbol
                        if symbol == INPUT_BLANK_SYMBOL {
                            if symbols[i] != self.blank_symbol {
                                return false;
                            }
                        } else if symbol != symbols[i] {
                            return false;
                        }
                    }

                    true
                })
            }
            _ => None,
        }
    }

    /// Returns the blank symbol used by this Turing Machine.
    pub fn get_blank_symbol(&self) -> char {
        self.blank_symbol
    }
}

#[cfg(test)]
mod multi_tape_tests {
    use super::*;
    use crate::types::{Direction, Program, Transition};
    use std::collections::HashMap;

    fn create_simple_multi_tape_program() -> Program {
        let mut rules = HashMap::new();

        // Simple program: replace ['a', 'x'] with ['b', 'y'] and move right on both tapes, then halt
        rules.insert(
            "start".to_string(),
            vec![Transition {
                read: vec!['a', 'x'],
                write: vec!['b', 'y'],
                directions: vec![Direction::Right, Direction::Right],
                next_state: "halt".to_string(),
            }],
        );

        // Halt state with no transitions
        rules.insert("halt".to_string(), Vec::new());

        Program {
            name: "Simple Multi-Tape Test".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        }
    }

    #[test]
    fn test_multi_tape_machine_creation() {
        let program = create_simple_multi_tape_program();
        let machine = TuringMachine::new(&program);

        assert_eq!(machine.get_state(), "start");
        assert_eq!(machine.get_tapes(), &[vec!['a'], vec!['x']]);
        assert_eq!(machine.get_head_positions(), &[0, 0]);
        assert_eq!(machine.get_step_count(), 0);
    }

    #[test]
    fn test_multi_tape_single_step() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        let result = machine.step();

        assert_eq!(result, ExecutionResult::Continue);
        assert_eq!(machine.get_state(), "halt");
        assert_eq!(machine.get_tapes(), &[vec!['b', '-'], vec!['y', '-']]); // Tapes extended when moving right
        assert_eq!(machine.get_head_positions(), &[1, 1]);
        assert_eq!(machine.get_step_count(), 1);
    }

    #[test]
    fn test_multi_tape_halt_state() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        // First step should continue
        let result1 = machine.step();
        assert_eq!(result1, ExecutionResult::Continue);

        // Second step should halt (no transitions in halt state)
        let result2 = machine.step();
        assert_eq!(result2, ExecutionResult::Halt);
    }

    #[test]
    fn test_multi_tape_no_transition_error() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        // Manually set tapes to symbols that have no transition
        machine.tapes = vec![vec!['z'], vec!['z']];

        let result = machine.step();

        match result {
            ExecutionResult::Error(TuringMachineError::NoMultiTapeTransition {
                state,
                symbols,
            }) => {
                assert_eq!(state, "start");
                assert_eq!(symbols, vec!['z', 'z']);
            }
            _ => panic!("Expected NoMultiTapeTransition error"),
        }
    }

    #[test]
    fn test_multi_tape_reset() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        // Execute a step
        machine.step();
        assert_eq!(machine.get_state(), "halt");
        assert_eq!(machine.get_step_count(), 1);

        // Reset
        machine.reset();
        assert_eq!(machine.get_state(), "start");
        assert_eq!(machine.get_tapes(), &[vec!['a'], vec!['x']]);
        assert_eq!(machine.get_head_positions(), &[0, 0]);
        assert_eq!(machine.get_step_count(), 0);
    }

    #[test]
    fn test_multi_tape_run_to_completion() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        let steps = machine.run_to_completion();

        // Should have recorded the initial state and the state after the step
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].state, "start");
        assert_eq!(steps[1].state, "halt");
    }

    #[test]
    fn test_multi_tape_is_halted() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(&program);

        assert!(!machine.is_halted()); // Should not be halted initially

        machine.step(); // Move to halt state
        assert!(machine.is_halted()); // Should be halted now
    }

    #[test]
    fn test_multi_tape_get_current_symbols() {
        let program = create_simple_multi_tape_program();
        let machine = TuringMachine::new(&program);

        assert_eq!(machine.get_current_symbols(), vec!['a', 'x']);
    }

    #[test]
    fn test_multi_tape_get_tapes_as_strings() {
        let program = create_simple_multi_tape_program();
        let machine = TuringMachine::new(&program);

        assert_eq!(
            machine.get_tapes_as_strings(),
            vec!["a".to_string(), "x".to_string()]
        );
    }

    #[test]
    fn test_multi_tape_validate_program_success() {
        let program = create_simple_multi_tape_program();
        assert!(TuringMachine::validate_program(&program).is_ok());
    }

    #[test]
    fn test_multi_tape_validate_program_initial_state_not_defined() {
        let mut rules = HashMap::new();
        rules.insert("other".to_string(), Vec::new());

        let program = Program {
            name: "Invalid".to_string(),
            initial_state: "nonexistent".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        };

        let result = TuringMachine::validate_program(&program);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initial state 'nonexistent' not defined in transitions"));
    }

    #[test]
    fn test_multi_tape_validate_program_empty_tapes() {
        let mut rules = HashMap::new();
        rules.insert("start".to_string(), Vec::new());

        let program = Program {
            name: "Invalid".to_string(),
            initial_state: "start".to_string(),
            tapes: vec![],
            heads: vec![],
            blank: '-',
            rules,
        };

        let result = TuringMachine::validate_program(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No tapes defined"));
    }

    #[test]
    fn test_multi_tape_validate_program_inconsistent_head_positions() {
        let mut rules = HashMap::new();
        rules.insert("start".to_string(), Vec::new());

        let program = Program {
            name: "Invalid".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0], // Only one head position for two tapes
            blank: '-',
            rules,
        };

        let result = TuringMachine::validate_program(&program);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Number of head positions"));
    }

    #[test]
    fn test_multi_tape_validate_program_undefined_state() {
        let mut rules = HashMap::new();
        rules.insert(
            "start".to_string(),
            vec![Transition {
                read: vec!['a', 'x'],
                write: vec!['b'], // Only one write symbol for two tapes
                directions: vec![Direction::Right, Direction::Right],
                next_state: "halt".to_string(),
            }],
        );
        rules.insert("halt".to_string(), Vec::new());

        let program = Program {
            name: "Invalid".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        };

        let result = TuringMachine::validate_program(&program);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("inconsistent tape counts"));
    }

    #[test]
    fn test_multi_tape_stay_direction() {
        let mut rules = HashMap::new();

        // Test the Stay direction
        rules.insert(
            "start".to_string(),
            vec![Transition {
                read: vec!['a', 'x'],
                write: vec!['b', 'y'],
                directions: vec![Direction::Stay, Direction::Right],
                next_state: "halt".to_string(),
            }],
        );
        rules.insert("halt".to_string(), Vec::new());

        let program = Program {
            name: "Stay Direction Test".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        };

        let mut machine = TuringMachine::new(&program);
        machine.step();

        // First head should stay at position 0, second head should move right
        assert_eq!(machine.get_head_positions(), &[0, 1]);
        assert_eq!(machine.get_tapes(), &[vec!['b'], vec!['y', '-']]);
    }

    #[test]
    fn test_write_input_blank_symbol_with_custom_blank() {
        let custom_blank = 'X';
        let program_content = format!(
            r#"
name: Custom Blank Write Test
blank: {custom_blank}
tape: a, _, b
rules:
  start:
    a -> a, R, halt
  halt:
"#,
        );

        let program = crate::parser::parse(&program_content).unwrap();
        assert_eq!(program.blank, custom_blank);
        assert_eq!(program.tapes[0].chars().nth(1).unwrap(), custom_blank);
    }
}
