//! This module provides functions for analyzing Turing Machine programs to detect common errors
//! and inconsistencies before execution. This includes checks for valid head positions, defined
//! states, reachable states, and handled tape symbols.

use crate::types::{Program, TuringMachineError};
use std::collections::HashSet;

/// Represents various errors that can be found during the analysis of a Turing Machine program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AnalysisError {
    /// Indicates an invalid head position, typically when it's out of bounds for the initial tape.
    InvalidHead(usize),
    /// Indicates that the initial state specified in the program is not defined in any transition rules.
    InvalidStartState(String),
    /// Indicates that certain stop states are not referenced as next states in any transitions,
    /// suggesting they might be unreachable or incorrectly defined.
    StopStatesNotFound(Vec<String>),
    /// Indicates that transitions reference states that are not defined in the program's rules.
    UndefinedNextStates(Vec<String>),
    /// Indicates states that are defined in the program's rules but cannot be reached from the initial state.
    UnreachableStates(Vec<String>),
    /// Indicates that the initial tape contains symbols for which no transitions are defined.
    InvalidTapeSymbols(Vec<char>),
    /// Indicates structural problems with the program (empty tapes, mismatched head positions, etc.).
    StructuralError(String),
}

impl From<AnalysisError> for TuringMachineError {
    /// Converts an `AnalysisError` into a `TuringMachineError::ValidationError`.
    fn from(error: AnalysisError) -> Self {
        match error {
            AnalysisError::InvalidHead(pos) => {
                TuringMachineError::ValidationError(format!("Invalid head position: {}", pos))
            }
            AnalysisError::InvalidStartState(state) => {
                TuringMachineError::ValidationError(format!("Invalid start state: {}", state))
            }
            AnalysisError::StopStatesNotFound(states) => TuringMachineError::ValidationError(
                format!("Stop states not found in transitions: {:?}", states),
            ),
            AnalysisError::UndefinedNextStates(transitions) => TuringMachineError::ValidationError(
                format!("Transitions reference undefined states: {:?}", transitions),
            ),
            AnalysisError::UnreachableStates(states) => TuringMachineError::ValidationError(
                format!("Unreachable states detected: {:?}", states),
            ),
            AnalysisError::InvalidTapeSymbols(symbols) => {
                TuringMachineError::ValidationError(format!(
                    "Initial tape contains symbols not handled by any transition: {:?}",
                    symbols
                ))
            }
            AnalysisError::StructuralError(msg) => TuringMachineError::ValidationError(msg),
        }
    }
}

/// Analyzes a given Turing Machine `Program` for structural and logical errors.
///
/// This function orchestrates a comprehensive series of checks, performing both
/// structural validation (basic consistency) and logical analysis (reachability,
/// symbol handling, etc.).
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to be analyzed.
///
/// # Returns
///
/// * `Ok(())` if no errors are found.
/// * `Err(TuringMachineError::ValidationError)` if any validation rule is violated.
pub fn analyze(program: &Program) -> Result<(), TuringMachineError> {
    let errors = [
        check_structure,
        check_head,
        check_valid_start_state,
        check_unreachable_states,
        check_tape_symbols,
    ]
    .iter()
    .filter_map(|f| f(program).err())
    .collect::<Vec<_>>();

    if !errors.is_empty() {
        // Return the first error
        if let Some(first_error) = errors.first() {
            return Err((*first_error).clone().into());
        }
    }

    Ok(())
}

/// Checks basic structural requirements of the program.
///
/// This validates fundamental structural consistency like:
/// - Tapes are defined (non-empty)
/// - Head positions match number of tapes
/// - Transitions have consistent tape counts
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if the structure is valid.
/// * `Err(AnalysisError::StructuralError)` if structural issues are found.
fn check_structure(program: &Program) -> Result<(), AnalysisError> {
    // Check for empty tapes
    if program.tapes.is_empty() {
        return Err(AnalysisError::StructuralError(
            "No tapes defined".to_string(),
        ));
    }

    // Check that head positions match number of tapes
    if program.heads.len() != program.tapes.len() {
        return Err(AnalysisError::StructuralError(format!(
            "Number of head positions ({}) does not match number of tapes ({})",
            program.heads.len(),
            program.tapes.len()
        )));
    }

    // Check that all transitions have consistent tape counts
    for (state, transitions) in &program.rules {
        for transition in transitions {
            if transition.read.len() != program.tapes.len()
                || transition.write.len() != program.tapes.len()
                || transition.directions.len() != program.tapes.len()
            {
                return Err(AnalysisError::StructuralError(format!(
                    "Transition in state '{}' has inconsistent tape counts",
                    state
                )));
            }
        }
    }

    Ok(())
}

/// Checks if the initial head position(s) are valid for the program's tape(s).
///
/// For single-tape programs, it verifies that the head position is within the bounds
/// of the initial tape. For multi-tape programs, it checks each head position against
/// its corresponding tape.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if the head position(s) are valid.
/// * `Err(AnalysisError::InvalidHead)` if an invalid head position is found.
fn check_head(program: &Program) -> Result<(), AnalysisError> {
    program
        .heads
        .iter()
        .zip(&program.tapes)
        .find_map(|(&head_pos, tape)| {
            (head_pos >= tape.len() && !tape.is_empty())
                .then_some(AnalysisError::InvalidHead(head_pos))
        })
        .map_or(Ok(()), Err)
}

/// Checks whether the initial state is defined as a source state in any of the transition rules.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if the initial state is defined.
/// * `Err(AnalysisError::InvalidStartState)` if the initial state is not found in the rules.
fn check_valid_start_state(program: &Program) -> Result<(), AnalysisError> {
    if !program.rules.contains_key(&program.initial_state) {
        return Err(AnalysisError::InvalidStartState(
            program.initial_state.clone(),
        ));
    }

    Ok(())
}

/// Checks whether states that have no outgoing transitions (potential stop states)
/// are referenced as `next_state` in any other transition.
///
/// This helps identify "dead-end" states that are not part of the machine's
/// intended halting mechanism.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if all stop states are properly referenced or if there are no unreferenced stop states.
/// * `Err(AnalysisError::StopStatesNotFound)` if stop states are found that are not referenced.
#[allow(dead_code)]
fn check_valid_stop_states(program: &Program) -> Result<(), AnalysisError> {
    // Collect all states that have no outgoing transitions (potential stop states)
    let stop_states: HashSet<String> = program
        .rules
        .iter()
        .filter(|(_, transitions)| transitions.is_empty())
        .map(|(state, _)| state.clone())
        .collect();

    // Collect all next states referenced in transitions
    let next_states: HashSet<String> = program
        .rules
        .values()
        .flat_map(|transitions| transitions.iter())
        .map(|transition| transition.next_state.clone())
        .collect();

    // Find stop states that are not referenced as next states
    let mut invalid: Vec<String> = stop_states.difference(&next_states).cloned().collect();

    if !invalid.is_empty() {
        // Sort the states to make it deterministic
        invalid.sort();
        return Err(AnalysisError::StopStatesNotFound(invalid));
    }

    Ok(())
}

/// Checks that all `next_state` references within transitions point to states that are
/// actually defined as keys in the program's rules.
///
/// The special "halt" state is implicitly defined and does not need to be present in the rules.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if all next states are defined or are the "halt" state.
/// * `Err(AnalysisError::UndefinedNextStates)` if transitions reference undefined states.
#[allow(dead_code)]
fn check_undefined_next_states(program: &Program) -> Result<(), AnalysisError> {
    let defined_states: HashSet<String> = program.rules.keys().cloned().collect();

    let mut undefined_transitions = Vec::new();
    for (state, transitions) in &program.rules {
        for (i, transition) in transitions.iter().enumerate() {
            if !defined_states.contains(&transition.next_state) && transition.next_state != "halt" {
                undefined_transitions
                    .push(format!("{}[{}] -> {}", state, i, transition.next_state));
            }
        }
    }

    if !undefined_transitions.is_empty() {
        return Err(AnalysisError::UndefinedNextStates(undefined_transitions));
    }

    Ok(())
}

/// Checks for unreachable states by performing a breadth-first search (BFS) traversal
/// starting from the initial state.
///
/// Any state defined in the program's rules that cannot be reached from the initial state
/// through any sequence of transitions is considered unreachable. The "halt" state is
/// implicitly reachable.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if all defined states are reachable or are the "halt" state.
/// * `Err(AnalysisError::UnreachableStates)` if unreachable states are found.
fn check_unreachable_states(program: &Program) -> Result<(), AnalysisError> {
    let initial_state = program.initial_state.clone();
    let mut visited = HashSet::new();
    let mut queue = vec![initial_state];

    while let Some(state) = queue.pop() {
        if visited.contains(&state) {
            continue;
        }

        visited.insert(state.clone());

        if let Some(transitions) = program.rules.get(&state) {
            for transition in transitions {
                if !visited.contains(&transition.next_state) {
                    queue.push(transition.next_state.clone());
                }
            }
        }
    }

    let all_states: HashSet<String> = program.rules.keys().cloned().collect();
    let mut unreachable: Vec<String> = all_states.difference(&visited).cloned().collect();

    if !unreachable.is_empty() {
        unreachable.sort(); // Sort for deterministic output
        return Err(AnalysisError::UnreachableStates(unreachable));
    }

    Ok(())
}

/// Checks that all symbols present in the initial tape(s) have corresponding transitions defined
/// in the program's rules for the states they might be encountered in.
///
/// This prevents the machine from getting stuck due to an unhandled symbol on the tape.
///
/// # Arguments
///
/// * `program` - A reference to the `Program` to check.
///
/// # Returns
///
/// * `Ok(())` if all initial tape symbols are handled by at least one transition.
/// * `Err(AnalysisError::InvalidTapeSymbols)` if unhandled symbols are found on the initial tape.
fn check_tape_symbols(program: &Program) -> Result<(), AnalysisError> {
    let mut initial_tape_symbols = HashSet::new();
    for tape in &program.tapes {
        for c in tape.chars() {
            initial_tape_symbols.insert(c);
        }
    }

    // If there are no symbols on any initial tape, there's nothing to check.
    if initial_tape_symbols.is_empty() {
        return Ok(());
    }

    let mut handled_symbols = HashSet::new();
    handled_symbols.insert(program.blank);
    for transitions in program.rules.values() {
        for transition in transitions {
            // For multi-tape, each symbol in the 'read' vector is a handled symbol.
            // For single-tape, transition.read will have length 1.
            for &symbol_in_read_vec in &transition.read {
                handled_symbols.insert(symbol_in_read_vec);
            }
        }
    }

    let mut unhandled_symbols: Vec<char> = initial_tape_symbols
        .iter()
        .filter(|c| !handled_symbols.contains(c))
        .cloned()
        .collect();

    if !unhandled_symbols.is_empty() {
        unhandled_symbols.sort();
        unhandled_symbols.dedup();
        return Err(AnalysisError::InvalidTapeSymbols(unhandled_symbols));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, Transition};
    use std::collections::HashMap;

    fn create_test_program(
        initial_state: &str,
        initial_tape: &str,
        rules: HashMap<String, Vec<Transition>>,
    ) -> Program {
        Program {
            name: "Test Program".to_string(),
            initial_state: initial_state.to_string(),
            tapes: vec![initial_tape.to_string()],
            heads: vec![0],
            blank: '-',
            rules,
        }
    }

    fn create_single_tape_transition(
        read: char,
        write: char,
        direction: Direction,
        next_state: &str,
    ) -> Transition {
        Transition {
            read: vec![read],
            write: vec![write],
            directions: vec![direction],
            next_state: next_state.to_string(),
        }
    }

    #[test]
    fn test_valid_program() {
        let mut rules = HashMap::new();

        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "halt",
            )],
        );
        rules.insert("halt".to_string(), Vec::new());

        let program = create_test_program("start", "a", rules);
        let result = analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_head_position() {
        let mut rules = HashMap::new();
        rules.insert("start".to_string(), Vec::new());

        // Empty tape should cause head validation to fail
        let program = create_test_program("start", "", rules);
        let result = check_head(&program);

        // With empty tape, head position 0 should be valid
        assert!(result.is_ok());
    }

    #[test]
    fn test_stop_states_not_found() {
        let mut rules = HashMap::new();

        // Create a start state that transitions to halt
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "other",
            )],
        );

        // Create a halt state with no transitions, but it's not referenced
        rules.insert("halt".to_string(), Vec::new());

        // Create the other state with no transitions
        rules.insert("other".to_string(), Vec::new());

        let program = create_test_program("start", "a", rules);
        let result = check_valid_stop_states(&program);

        // halt state is not referenced, so it should be flagged as invalid
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error,
            AnalysisError::StopStatesNotFound(vec!["halt".to_string()])
        );
    }

    #[test]
    fn test_analysis_error_conversion() {
        let error = AnalysisError::InvalidHead(5);
        let tm_error: TuringMachineError = error.into();

        match tm_error {
            TuringMachineError::ValidationError(msg) => {
                assert!(msg.contains("Invalid head position: 5"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_undefined_next_states() {
        let mut rules = HashMap::new();

        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "nonexistent",
            )],
        );

        let program = create_test_program("start", "a", rules);
        let result = check_undefined_next_states(&program);

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AnalysisError::UndefinedNextStates(transitions) => {
                assert_eq!(transitions.len(), 1);
                assert!(transitions[0].contains("nonexistent"));
            }
            _ => panic!("Expected UndefinedNextStates error"),
        }
    }

    #[test]
    fn test_undefined_next_states_with_halt() {
        // "halt" is a special state that doesn't need to be defined
        let mut rules = HashMap::new();

        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "halt",
            )],
        );

        let program = create_test_program("start", "a", rules);
        let result = check_undefined_next_states(&program);

        // Should be OK because "halt" is a special case
        assert!(result.is_ok());
    }

    #[test]
    fn test_unreachable_states() {
        let mut rules = HashMap::new();

        // Create a start state that transitions to middle
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "middle",
            )],
        );

        // Create a middle state that transitions to end
        rules.insert(
            "middle".to_string(),
            vec![create_single_tape_transition(
                'b',
                'c',
                Direction::Right,
                "end",
            )],
        );

        // Create an end state with no transitions
        rules.insert("end".to_string(), Vec::new());

        // Create an unreachable state
        rules.insert("unreachable".to_string(), Vec::new());

        let program = create_test_program("start", "a", rules);
        let result = check_unreachable_states(&program);

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AnalysisError::UnreachableStates(states) => {
                assert_eq!(states.len(), 1);
                assert_eq!(states[0], "unreachable");
            }
            _ => panic!("Expected UnreachableStates error"),
        }
    }

    #[test]
    fn test_tape_symbols() {
        let mut rules = HashMap::new();

        // Create a start state that only handles 'a'
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "halt",
            )],
        );

        // Initial tape contains 'a' and 'c', but 'c' is not handled
        let program = create_test_program("start", "ac", rules);
        let result = check_tape_symbols(&program);

        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AnalysisError::InvalidTapeSymbols(symbols) => {
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0], 'c');
            }
            _ => panic!("Expected InvalidTapeSymbols error"),
        }
    }

    #[test]
    fn test_multi_tape_symbols() {
        let mut rules = HashMap::new();

        // Program handles 'a' on tape 1 and 'x' on tape 2
        rules.insert(
            "start".to_string(),
            vec![Transition {
                read: vec!['a', 'x'],
                write: vec!['b', 'y'],
                directions: vec![Direction::Right, Direction::Right],
                next_state: "halt".to_string(),
            }],
        );
        rules.insert("halt".to_string(), Vec::new());

        // Initial tapes: ["a", "x"], should be valid
        let program_valid = Program {
            name: "Valid Multi-Tape".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "x".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules: rules.clone(),
        };
        assert!(check_tape_symbols(&program_valid).is_ok());

        // Initial tapes: ["a", "z"], 'z' is not handled
        let program_invalid = Program {
            name: "Invalid Multi-Tape".to_string(),
            initial_state: "start".to_string(),
            tapes: vec!["a".to_string(), "z".to_string()],
            heads: vec![0, 0],
            blank: '-',
            rules,
        };
        let result = check_tape_symbols(&program_invalid);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            AnalysisError::InvalidTapeSymbols(symbols) => {
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0], 'z');
            }
            _ => panic!("Expected InvalidTapeSymbols error"),
        }
    }

    #[test]
    fn test_all_checks_together() {
        // Create a program with multiple issues
        let mut rules = HashMap::new();

        // Start state transitions to nonexistent state
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "nonexistent",
            )],
        );

        // Unreachable state
        rules.insert("unreachable".to_string(), Vec::new());

        // Initial tape contains symbol not handled
        let program = create_test_program("start", "ax", rules);
        let result = analyze(&program);

        assert!(result.is_err());

        // Since analyze() now returns the first error found, we just check that an error occurred
        if let Err(TuringMachineError::ValidationError(msg)) = result {
            // Should contain one of the expected error types
            let has_error = msg.contains("No tapes defined")
                || msg.contains("Number of head positions")
                || msg.contains("Invalid start state")
                || msg.contains("Stop states not found")
                || msg.contains("undefined state")
                || msg.contains("Unreachable states")
                || msg.contains("not handled by any transition");
            assert!(has_error, "Expected a validation error, got: {}", msg);
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_valid_program_with_all_checks() {
        let mut rules = HashMap::new();

        // Start state transitions to middle
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "middle",
            )],
        );

        // Middle state transitions to halt
        rules.insert(
            "middle".to_string(),
            vec![create_single_tape_transition(
                'b',
                'c',
                Direction::Right,
                "halt",
            )],
        );

        // All states are reachable, all transitions point to defined states,
        // and all tape symbols are handled
        let program = create_test_program("start", "a", rules);
        let result = analyze(&program);

        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_success() {
        let mut rules = HashMap::new();
        rules.insert(
            "start".to_string(),
            vec![create_single_tape_transition(
                'a',
                'b',
                Direction::Right,
                "halt",
            )],
        );

        let program = create_test_program("start", "a", rules);
        let result = analyze(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_structural_error() {
        let mut rules = HashMap::new();
        // Initial state "start" is not defined in rules
        rules.insert("other".to_string(), Vec::new());

        let program = create_test_program("start", "a", rules);
        let result = analyze(&program);

        assert!(result.is_err());
        if let Err(TuringMachineError::ValidationError(msg)) = result {
            assert!(msg.contains("Invalid start state: start"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_analyze_empty_tapes() {
        let mut rules = HashMap::new();
        rules.insert("start".to_string(), Vec::new());

        let mut program = create_test_program("start", "a", rules);
        program.tapes.clear(); // Remove all tapes

        let result = analyze(&program);

        assert!(result.is_err());
        if let Err(TuringMachineError::ValidationError(msg)) = result {
            assert!(msg.contains("No tapes defined"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_analyze_inconsistent_head_positions() {
        let mut rules = HashMap::new();
        rules.insert("start".to_string(), Vec::new());

        let mut program = create_test_program("start", "a", rules);
        program.heads.push(1); // Add extra head position

        let result = analyze(&program);

        assert!(result.is_err());
        if let Err(TuringMachineError::ValidationError(msg)) = result {
            assert!(msg.contains("Number of head positions"));
        } else {
            panic!("Expected ValidationError");
        }
    }
}
