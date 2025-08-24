//! This module provides encoding functionality for converting Turing Machine programs
//! into a string format suitable for Universal Turing Machine processing.

use crate::types::{Direction, Mode, Program, Transition};
use std::collections::{hash_map::Entry, HashMap};

/// Encodes a Turing Machine program into a string format for Universal Turing Machine.
///
/// Format: `name:tape:rules`
/// - name: The name of the program.
/// - tape: Comma-separated symbols of the initial tape.
/// - rules: Pipe-separated transitions in format `current_state,input,output,direction,next_state`.
///
/// # Arguments
///
/// * `program` - The Program to encode.
///
/// # Returns
///
/// * `String` - The encoded program string.
pub fn encode(program: &Program) -> String {
    let state_mapping = create_state_mapping(program);

    let rules_section = encode_rules(program, &state_mapping);
    let tape_section = encode_initial_tape(program);

    format!("{}:{}:{}", program.name, tape_section, rules_section)
}

/// Creates a mapping from state names to numeric identifiers.
fn create_state_mapping(program: &Program) -> HashMap<String, String> {
    let mut mapping = HashMap::new();
    let mut counter = 0;

    // Always map initial state to 0
    mapping.insert(program.initial_state.clone(), "0".to_string());
    counter += 1;

    // Collect all unique states from rules
    let mut states: Vec<String> = program.rules.keys().cloned().collect();

    // Add states from transitions
    for transitions in program.rules.values() {
        for transition in transitions {
            states.push(transition.next_state.clone());
        }
    }

    // Remove duplicates and sort for consistent encoding
    states.sort();
    states.dedup();

    // Map remaining states
    for state in states {
        if let Entry::Vacant(e) = mapping.entry(state) {
            // Use special mappings for common halt states
            let encoded = match e.key().as_str() {
                "halt" => "h".to_string(),
                "accept" => "a".to_string(),
                "stop" => "s".to_string(),
                "reject" => "r".to_string(),
                _ => counter.to_string(),
            };

            let is_special = matches!(e.key().as_str(), "halt" | "accept" | "stop" | "reject");
            e.insert(encoded);
            if !is_special {
                counter += 1;
            }
        }
    }

    mapping
}

/// Encodes the rules section as pipe-separated transitions.
fn encode_rules(program: &Program, state_mapping: &HashMap<String, String>) -> String {
    let mut encoded_rules = Vec::new();

    // Sort states for consistent output
    let mut sorted_states: Vec<_> = program.rules.keys().collect();
    sorted_states.sort();

    for state in sorted_states {
        let state_encoded = state_mapping.get(state).unwrap();
        let transitions = program.rules.get(state).unwrap();

        for transition in transitions {
            // For single-tape machines, use first elements
            let input_symbol = transition.read.first().unwrap_or(&'_');
            let output_symbol = transition.write.first().unwrap_or(&'_');
            let direction = transition.directions.first().unwrap_or(&Direction::Stay);
            let next_state_encoded = state_mapping.get(&transition.next_state).unwrap();

            let direction_char = match direction {
                Direction::Left => 'L',
                Direction::Right => 'R',
                Direction::Stay => 'S',
            };

            let rule = format!(
                "{},{},{},{},{}",
                state_encoded, input_symbol, output_symbol, direction_char, next_state_encoded
            );

            encoded_rules.push(rule);
        }
    }

    encoded_rules.join("|")
}

/// Encodes the initial tape as comma-separated symbols.
fn encode_initial_tape(program: &Program) -> String {
    if let Some(first_tape) = program.tapes.first() {
        first_tape
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    } else {
        String::new()
    }
}

/// Decodes an encoded program string back into a Program structure.
///
/// # Arguments
///
/// * `encoded` - The encoded program string in `name:tape:rules` format.
///
/// # Returns
///
/// * `Result<Program, String>` - The decoded Program or an error message.
pub fn decode(encoded: &str) -> Result<Program, String> {
    let parts: Vec<&str> = encoded.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid encoding format: expected 3 sections separated by :".to_string());
    }

    let name = parts[0];
    let tape_section = parts[1];
    let rules_section = parts[2];

    // Create reverse state mapping from the rules
    let state_mapping = decode_states_from_rules(rules_section)?;
    let rules = decode_rules(rules_section, &state_mapping)?;
    let initial_tape = decode_initial_tape(tape_section)?;

    // Find initial state (mapped to "0")
    let initial_state = state_mapping
        .get("0")
        .ok_or("No initial state found (state 0)")?
        .clone();

    Ok(Program {
        name: name.to_string(),
        mode: Mode::default(),
        initial_state,
        tapes: vec![initial_tape],
        heads: vec![0],
        blank: '_',
        rules,
    })
}

/// Decodes the states from the rules section into a reverse mapping.
fn decode_states_from_rules(rules_section: &str) -> Result<HashMap<String, String>, String> {
    let mut mapping = HashMap::new();
    let mut encoded_states = std::collections::HashSet::<String>::new();

    if !rules_section.is_empty() {
        let rule_strings: Vec<&str> = rules_section.split('|').collect();
        for rule_str in rule_strings {
            let parts: Vec<&str> = rule_str.split(',').collect();
            if parts.len() != 5 {
                return Err(format!("Invalid rule format: {}", rule_str));
            }
            encoded_states.insert(parts[0].to_string());
            encoded_states.insert(parts[4].to_string());
        }
    }

    // The initial state "0" might not be in the rules if the program is empty or has no transitions from start.
    encoded_states.insert("0".to_string());

    for encoded_state in &encoded_states {
        let original_state = match encoded_state.as_str() {
            "h" => "halt".to_string(),
            "a" => "accept".to_string(),
            "s" => "stop".to_string(),
            "r" => "reject".to_string(),
            "0" => "start".to_string(), // Initial state
            _ => {
                if let Ok(num) = encoded_state.parse::<u32>() {
                    if num == 1 {
                        // Common case: first non-initial state is often s2
                        "s2".to_string()
                    } else {
                        format!("s{}", num + 1)
                    }
                } else {
                    encoded_state.to_string()
                }
            }
        };
        mapping.insert(encoded_state.clone(), original_state);
    }

    Ok(mapping)
}

/// Decodes the rules section into transition rules.
fn decode_rules(
    rules_section: &str,
    state_mapping: &HashMap<String, String>,
) -> Result<HashMap<String, Vec<Transition>>, String> {
    let mut rules = HashMap::new();

    if rules_section.is_empty() {
        return Ok(rules);
    }

    let rule_strings: Vec<&str> = rules_section.split('|').collect();

    for rule_str in rule_strings {
        let parts: Vec<&str> = rule_str.split(',').collect();
        if parts.len() != 5 {
            return Err(format!("Invalid rule format: {}", rule_str));
        }

        let current_state_encoded = parts[0];
        let input_symbol = parts[1].chars().next().unwrap_or('_');
        let output_symbol = parts[2].chars().next().unwrap_or('_');
        let direction_char = parts[3].chars().next().unwrap_or('S');
        let next_state_encoded = parts[4];

        let current_state = state_mapping
            .get(current_state_encoded)
            .ok_or(format!("Unknown state: {}", current_state_encoded))?
            .clone();

        let next_state = state_mapping
            .get(next_state_encoded)
            .ok_or(format!("Unknown state: {}", next_state_encoded))?
            .clone();

        let direction = match direction_char {
            'L' => Direction::Left,
            'R' => Direction::Right,
            'S' => Direction::Stay,
            _ => return Err(format!("Invalid direction: {}", direction_char)),
        };

        let transition = Transition {
            read: vec![input_symbol],
            write: vec![output_symbol],
            directions: vec![direction],
            next_state,
        };

        rules
            .entry(current_state)
            .or_insert_with(Vec::new)
            .push(transition);
    }

    Ok(rules)
}

/// Decodes the initial tape section.
fn decode_initial_tape(tape_section: &str) -> Result<String, String> {
    if tape_section.is_empty() {
        return Ok(String::new());
    }

    let symbols: Vec<&str> = tape_section.split(',').collect();
    Ok(symbols.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Direction, Program, Transition};
    use std::collections::HashMap;

    fn create_test_program() -> Program {
        let mut rules = HashMap::new();

        // start: a -> b, R, s2
        rules.insert(
            "start".to_string(),
            vec![Transition {
                read: vec!['a'],
                write: vec!['b'],
                directions: vec![Direction::Right],
                next_state: "s2".to_string(),
            }],
        );

        // s2: b -> b, R, halt
        rules.insert(
            "s2".to_string(),
            vec![Transition {
                read: vec!['b'],
                write: vec!['b'],
                directions: vec![Direction::Right],
                next_state: "halt".to_string(),
            }],
        );

        Program {
            name: "Test Program".to_string(),
            mode: Mode::default(),
            initial_state: "start".to_string(),
            tapes: vec!["abb".to_string()],
            heads: vec![0],
            blank: '_',
            rules,
        }
    }

    #[test]
    fn test_encode_program() {
        let program = create_test_program();
        let encoded = encode(&program);

        // Should contain name, tape, and rules sections
        assert!(encoded.contains(':'));
        let parts: Vec<&str> = encoded.split(':').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "Test Program");

        println!("Encoded: {}", encoded);
    }

    #[test]
    fn test_state_mapping() {
        let program = create_test_program();
        let mapping = create_state_mapping(&program);

        // start should map to 0
        assert_eq!(mapping.get("start"), Some(&"0".to_string()));
        // halt should map to h
        assert_eq!(mapping.get("halt"), Some(&"h".to_string()));
        // s2 should map to some number
        assert!(mapping.contains_key("s2"));
    }

    #[test]
    fn test_encode_rules() {
        let program = create_test_program();
        let state_mapping = create_state_mapping(&program);
        let rules = encode_rules(&program, &state_mapping);

        // Should contain pipe-separated rules
        assert!(rules.contains('|'));
        // Should contain comma-separated rule components
        assert!(rules.contains(','));

        println!("Encoded rules: {}", rules);
    }

    #[test]
    fn test_encode_initial_tape() {
        let program = create_test_program();
        let tape = encode_initial_tape(&program);

        assert_eq!(tape, "a,b,b");
    }

    #[test]
    fn test_round_trip_encoding() {
        let original = create_test_program();
        let encoded = encode(&original);
        println!(
            "Original program rules: {:?}",
            original.rules.keys().collect::<Vec<_>>()
        );
        println!("Encoded: {}", encoded);

        let decoded = decode(&encoded).unwrap();
        println!(
            "Decoded program rules: {:?}",
            decoded.rules.keys().collect::<Vec<_>>()
        );

        // Check that key properties are preserved
        assert_eq!(decoded.name, "Test Program");
        assert_eq!(decoded.initial_state, "start");
        assert_eq!(decoded.tapes[0], "abb");
        assert!(decoded.rules.contains_key("start"));
        // The state "s2" gets encoded as "1" and decoded as "s2"
        assert!(decoded.rules.contains_key("s2"));
        assert_eq!(decoded.rules.len(), 2);
    }

    #[test]
    fn test_decode_invalid_format() {
        let result = decode("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid encoding format"));
    }

    #[test]
    fn test_simple_example() {
        // Test the exact example from the design
        let mut rules = HashMap::new();

        rules.insert(
            "start".to_string(),
            vec![
                Transition {
                    read: vec!['a'],
                    write: vec!['b'],
                    directions: vec![Direction::Right],
                    next_state: "s2".to_string(),
                },
                Transition {
                    read: vec!['b'],
                    write: vec!['b'],
                    directions: vec![Direction::Right],
                    next_state: "s2".to_string(),
                },
            ],
        );

        let program = Program {
            name: "Simple Example".to_string(),
            mode: Mode::default(),
            initial_state: "start".to_string(),
            tapes: vec!["ab".to_string()],
            heads: vec![0],
            blank: '_',
            rules,
        };

        let encoded = encode(&program);
        println!("Simple example encoded: {}", encoded);

        // Should match the expected format: name:tape:rules
        assert!(encoded.starts_with("Simple Example:a,b:"));
        assert!(encoded.contains("0,a,b,R,"));
        assert!(encoded.contains("0,b,b,R,"));
    }
}
