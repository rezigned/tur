//! This module defines the `TuringMachine` struct, which simulates the behavior of a
//! multi-tape Turing Machine. It handles the machine's state, tape operations, head movements,
//! and execution of transition rules.

use crate::types::{
    Direction, Halt, Mode, Program, Step, Transition, TuringMachineError, INPUT_BLANK_SYMBOL,
    MAX_EXECUTION_STEPS,
};

/// Represents a multi-tape Turing Machine.
///
/// This struct encapsulates the current state of the Turing Machine, including its
/// current state, the contents of its tapes, the positions of its read/write heads,
/// the blank symbol, and the set of transition rules.
pub struct TuringMachine {
    state: String,
    tapes: Vec<Vec<char>>,
    heads: Vec<usize>,
    blank: char,
    program: Program,
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
    /// * `program` - The `Program` defining the Turing Machine.
    pub fn new(program: Program) -> Self {
        Self {
            state: program.initial_state.clone(),
            tapes: program.tapes().clone(),
            heads: program.heads.clone(),
            blank: program.blank,
            program,
            step_count: 0,
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
    /// * `ExecutionResult::Halt(_)` if the machine enters a halt state (no defined transitions).
    pub fn step(&mut self) -> Step {
        if self.is_halted() {
            return Step::Halt(Halt::Ok);
        }

        // Ensure all tapes are large enough
        for (i, head_pos) in self.heads.iter().enumerate() {
            if *head_pos >= self.tapes[i].len() {
                self.tapes[i].resize(*head_pos + 1, self.blank);
            }
        }

        // Find matching transition
        let transition = match self.transition().cloned() {
            Some(t) => t,
            None => {
                // No transition found for the current symbols.
                return match self.program.mode {
                    Mode::Normal => Step::Halt(Halt::Ok),
                    Mode::Strict => Step::Halt(Halt::Err(TuringMachineError::UndefinedTransition(
                        self.state.clone(),
                        self.symbols(),
                    ))),
                };
            }
        };

        // Apply transition to all tapes
        for i in 0..self.tapes.len() {
            // Write new symbol
            self.tapes[i][self.heads[i]] = if transition.write[i] == INPUT_BLANK_SYMBOL {
                self.blank
            } else {
                transition.write[i]
            };

            // Move head according to direction
            match transition.directions[i] {
                Direction::Left => {
                    if self.heads[i] == 0 {
                        // Extend tape to the left
                        self.tapes[i].insert(0, self.blank);
                    } else {
                        self.heads[i] -= 1;
                    }
                }
                Direction::Right => {
                    self.heads[i] += 1;
                    if self.heads[i] >= self.tapes[i].len() {
                        self.tapes[i].push(self.blank);
                    }
                }
                Direction::Stay => {
                    // Head position remains unchanged
                }
            }
        }

        self.state = transition.next_state.clone();
        self.step_count += 1;

        Step::Continue
    }

    /// Runs the Turing Machine until it halts or reaches a maximum step count.
    pub fn run(&mut self) -> Step {
        for _ in 0..MAX_EXECUTION_STEPS {
            match self.step() {
                Step::Continue => continue,
                halt => return halt,
            }
        }

        Step::Halt(Halt::Ok)
    }

    /// Returns the current state of the Turing Machine.
    pub fn state(&self) -> &str {
        &self.state
    }

    /// Returns the initial state of the Turing Machine.
    pub fn initial_state(&self) -> &str {
        &self.program.initial_state
    }

    /// Resets the Turing Machine to its initial configuration.
    /// This includes resetting the state, tapes, head positions, and step count.
    pub fn reset(&mut self) {
        self.state = self.program.initial_state.clone();
        self.tapes = self.program.tapes().clone();
        self.heads = self.program.heads.clone();
        self.step_count = 0;
    }

    /// Returns the total number of steps executed by the Turing Machine.
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Checks if the Turing Machine is currently in a halted state.
    /// A machine is halted if there are no defined transitions for its current state.
    pub fn is_halted(&self) -> bool {
        self.program
            .rules
            .get(&self.state)
            .is_none_or(|transitions| transitions.is_empty())
    }

    /// Returns a slice of the machine's tapes.
    pub fn tapes(&self) -> &[Vec<char>] {
        &self.tapes
    }

    /// Returns a slice of the machine's head positions for all tapes.
    pub fn heads(&self) -> &[usize] {
        &self.heads
    }

    /// Returns a vector of symbols currently under each tape's head.
    /// If a head is beyond its tape's current length, the blank symbol is returned for that tape.
    ///
    /// | a | b | c | tape 1
    /// | d | e |   | tape 2
    ///   0   1   2   index
    ///
    /// heads [0, 2] will return ['a', '_']
    pub fn symbols(&self) -> Vec<char> {
        self.heads
            .iter()
            .enumerate()
            .map(|(i, &pos)| {
                if pos < self.tapes[i].len() {
                    self.tapes[i][pos]
                } else {
                    self.blank
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
    pub fn transition(&self) -> Option<&Transition> {
        match self.program.rules.get(&self.state) {
            Some(transitions) => {
                let symbols = self.symbols();

                transitions.iter().find(|t| {
                    if t.read.len() != symbols.len() {
                        return false;
                    }

                    for (i, &read) in t.read.iter().enumerate() {
                        // If the transition rule specifies `INPUT_BLANK_SYMBOL`, it matches the program's blank symbol
                        let expected = if read == INPUT_BLANK_SYMBOL {
                            self.blank
                        } else {
                            read
                        };

                        if symbols[i] != expected {
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
    pub fn blank(&self) -> char {
        self.blank
    }

    /// Sets the content of a specific tape.
    ///
    /// # Arguments
    ///
    /// * `tape_index` - The index of the tape to modify (0-based)
    /// * `content` - The new content for the tape as a string
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the tape was successfully set
    /// * `Err(TuringMachineError)` if the tape index is invalid
    pub fn set_tape_content(
        &mut self,
        tape_index: usize,
        content: &str,
    ) -> Result<(), TuringMachineError> {
        if tape_index >= self.tapes.len() {
            return Err(TuringMachineError::ValidationError(format!(
                "Tape index {} is out of bounds (machine has {} tapes)",
                tape_index,
                self.tapes.len()
            )));
        }

        self.tapes[tape_index] = content
            .chars()
            .map(|c| {
                if c == INPUT_BLANK_SYMBOL {
                    self.blank
                } else {
                    c
                }
            })
            .collect();
        Ok(())
    }

    /// Sets the content of multiple tapes at once.
    ///
    /// # Arguments
    ///
    /// * `contents` - A vector of strings representing the new content for each tape
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all tapes were successfully set
    /// * `Err(TuringMachineError)` if there are more contents than tapes
    pub fn set_tapes_content(&mut self, contents: &[String]) -> Result<(), TuringMachineError> {
        if contents.len() > self.tapes.len() {
            return Err(TuringMachineError::ValidationError(format!(
                "Too many tape contents provided: {} contents for {} tapes",
                contents.len(),
                self.tapes.len()
            )));
        }

        for (i, content) in contents.iter().enumerate() {
            self.set_tape_content(i, content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod multi_tape_tests {
    use super::*;
    use crate::types::{Direction, Halt, Mode, Program, Transition};
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
            mode: Mode::default(),
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
        let machine = TuringMachine::new(program);

        assert_eq!(machine.state(), "start");
        assert_eq!(machine.tapes(), &[vec!['a'], vec!['x']]);
        assert_eq!(machine.heads(), &[0, 0]);
        assert_eq!(machine.step_count(), 0);
    }

    #[test]
    fn test_multi_tape_single_step() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        let result = machine.step();

        assert_eq!(result, Step::Continue);
        assert_eq!(machine.state(), "halt");
        assert_eq!(machine.tapes(), &[vec!['b', '-'], vec!['y', '-']]); // Tapes extended when moving right
        assert_eq!(machine.heads(), &[1, 1]);
        assert_eq!(machine.step_count(), 1);
    }

    #[test]
    fn test_multi_tape_halt_state() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        // First step should continue
        let result1 = machine.step();
        assert_eq!(result1, Step::Continue);

        // Second step should halt (no transitions in halt state)
        let result2 = machine.step();
        assert_eq!(result2, Step::Halt(Halt::Ok));
    }

    #[test]
    fn test_multi_tape_rejection() {
        let mut program = create_simple_multi_tape_program();
        program.mode = Mode::Strict;

        let mut machine = TuringMachine::new(program);

        // Manually set tapes to symbols that have no transition
        machine
            .set_tapes_content(&["z".to_string(), "z".to_string()])
            .unwrap();

        let result = machine.step();

        match result {
            Step::Halt(Halt::Err(TuringMachineError::UndefinedTransition(state, symbols))) => {
                assert_eq!(state, "start");
                assert_eq!(symbols, vec!['z', 'z']);
            }
            _ => panic!("Expected a Rejection result, but got {:?}", result),
        }
    }

    #[test]
    fn test_multi_tape_reset() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        // Execute a step
        machine.step();
        assert_eq!(machine.state(), "halt");
        assert_eq!(machine.step_count(), 1);

        // Reset
        machine.reset();
        assert_eq!(machine.state(), "start");
        assert_eq!(machine.tapes(), &[vec!['a'], vec!['x']]);
        assert_eq!(machine.heads(), &[0, 0]);
        assert_eq!(machine.step_count(), 0);
    }

    #[test]
    fn test_multi_tape_run_to_completion() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        let step = machine.run();
        assert_eq!(step, Step::Halt(Halt::Ok));
    }

    #[test]
    fn test_multi_tape_is_halted() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        assert!(!machine.is_halted()); // Should not be halted initially

        machine.step(); // Move to halt state
        assert!(machine.is_halted()); // Should be halted now
    }

    #[test]
    fn test_multi_tape_get_current_symbols() {
        let program = create_simple_multi_tape_program();
        let machine = TuringMachine::new(program);

        assert_eq!(machine.symbols(), vec!['a', 'x']);
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
            mode: Mode::default(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        };

        let mut machine = TuringMachine::new(program);
        machine.step();

        // First head should stay at position 0, second head should move right
        assert_eq!(machine.heads(), &[0, 1]);
        assert_eq!(machine.tapes(), &[vec!['b'], vec!['y', '-']]);
    }

    #[test]
    fn test_set_tape_content_with_blank_symbol() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        // Test setting tape content with INPUT_BLANK_SYMBOL ('_')
        machine.set_tape_content(0, "a_b").unwrap();

        // The '_' should be converted to the machine's blank symbol ('-')
        assert_eq!(machine.tapes()[0], vec!['a', '-', 'b']);
    }

    #[test]
    fn test_set_tapes_content_with_blank_symbol() {
        let program = create_simple_multi_tape_program();
        let mut machine = TuringMachine::new(program);

        // Test setting multiple tapes with INPUT_BLANK_SYMBOL ('_')
        let contents = vec!["a_b".to_string(), "x_y".to_string()];
        machine.set_tapes_content(&contents).unwrap();

        // The '_' should be converted to the machine's blank symbol ('-')
        assert_eq!(machine.tapes()[0], vec!['a', '-', 'b']);
        assert_eq!(machine.tapes()[1], vec!['x', '-', 'y']);
    }

    #[test]
    fn test_write_input_blank_symbol_with_custom_blank() {
        let custom_blank = 'X';
        let program_content = format!(
            r#"
name: Custom Blank Write Test
blank: {custom_blank}
tape: a, _
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
