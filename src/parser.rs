//! This module provides the parser for Turing Machine programs, utilizing the `pest` crate.
//! It defines the grammar for `.tur` files and functions to parse the input into a `Program` struct.

use crate::{
    analyzer::analyze,
    types::{
        Direction, Program, Transition, TuringMachineError, DEFAULT_BLANK_SYMBOL,
        INPUT_BLANK_SYMBOL,
    },
};
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
    Parser as PestParser, Span,
};
use pest_derive::Parser as PestParser;
use std::collections::{HashMap, HashSet};

type Tape = Vec<char>;

/// Derives a `PestParser` for the Turing Machine grammar defined in `grammar.pest`.
#[derive(PestParser)]
#[grammar = "grammar.pest"]
pub struct TuringMachineParser;

/// Parses the given input string into a `Program` struct.
///
/// This is the main entry point for parsing Turing Machine program definitions.
/// It trims the input, parses it using the `TuringMachineParser`, and then processes
/// the resulting parse tree into a structured `Program`. The parsed program is
/// automatically validated before being returned.
///
/// # Arguments
///
/// * `input` - A string slice containing the Turing Machine program definition.
///
/// # Returns
///
/// * `Ok(Program)` if the input is successfully parsed and validated.
/// * `Err(TuringMachineError::ParseError)` if there are any syntax errors.
/// * `Err(TuringMachineError::ValidationError)` if the program fails validation.
pub fn parse(input: &str) -> Result<Program, TuringMachineError> {
    let root = TuringMachineParser::parse(Rule::program, input.trim())
        .map_err(|e| TuringMachineError::ParseError(e.into()))? //
        .next()
        .unwrap();

    let program = parse_program(root)?;

    // Analyze the parsed program
    analyze(&program)?;

    Ok(program)
}

/// Parses the top-level structure of a Turing Machine program from a `Pair<Rule::program>`.
///
/// This function extracts the program's name, tapes, heads, blank symbol, rules, and initial state.
/// It also performs initial validation checks for uniqueness and consistency of sections.
fn parse_program(pair: Pair<Rule>) -> Result<Program, TuringMachineError> {
    let mut name: Option<String> = None;
    let mut tapes: Option<(Vec<Tape>, Vec<Vec<usize>>)> = None;
    let mut heads: Option<Vec<usize>> = None;
    let mut blank: Option<char> = None;
    let mut rules: Option<HashMap<String, Vec<Transition>>> = None;
    let mut initial_state: Option<String> = None;
    let mut seen = HashSet::new();

    // Parse top-level rules
    for p in pair.into_inner() {
        let span = p.as_span();
        let rule = p.as_rule();

        check_unique_rule(rule, span, &mut seen)?;

        match rule {
            Rule::name => name = Some(parse_inner_string(p)),
            Rule::blank => blank = Some(parse_symbol(&parse_inner_string(p))),
            Rule::rules => rules = Some(parse_transitions(p, &mut initial_state)?),
            Rule::tape | Rule::tapes => {
                check_exclusive_rule(tapes, vec!["tape", "tapes"], span)?;
                tapes = Some(parse_tapes(p));
            }
            Rule::head | Rule::heads => {
                check_exclusive_rule(heads, vec!["head", "heads"], span)?;
                heads = Some(parse_heads(p));
            }
            _ => {} // Skip other rules
        }
    }

    // Handle mandatory checks
    let name = check_required_rule(name, vec!["name"])?;
    let rules = check_required_rule(rules, vec!["rules"])?;
    let initial_state = check_required_rule(initial_state, vec!["initial_state"])?;
    let tapes = check_required_rule(tapes, vec!["tape", "tapes"])?;
    let blank = blank.unwrap_or(DEFAULT_BLANK_SYMBOL);

    // Rewrite blank symbol
    let tapes = rewrite_tapes(tapes, blank);
    let heads = heads.unwrap_or_else(|| vec![0; tapes.len()]);

    check_head_tape_consistency(&heads, &tapes)?;

    Ok(Program {
        name,
        tapes: tapes
            .into_iter()
            .map(|tape| tape.into_iter().collect())
            .collect(),
        heads,
        blank,
        rules,
        initial_state,
    })
}

/// Parses tape definitions from a `Pair<Rule::tape>` or `Pair<Rule::tapes>`.
///
/// It extracts the symbols for each tape and records the positions of any `INPUT_BLANK_SYMBOL`s
/// for later rewriting.
fn parse_tapes(pair: Pair<Rule>) -> (Vec<Vec<char>>, Vec<Vec<usize>>) {
    let mut tapes = Vec::new();
    let mut blank_indices = Vec::new();

    // Rule: (tape | tapes) > symbols > [symbol]
    for tape_pair in pair.into_inner() {
        if tape_pair.as_rule() == Rule::symbols {
            let mut line = vec![];
            let mut tape_blank_indices = Vec::new();
            for tape_item in tape_pair.into_inner() {
                // Stores blank indices of each tape for rewriting purpose.
                let symbol = parse_symbol(tape_item.as_str());
                if symbol == INPUT_BLANK_SYMBOL {
                    tape_blank_indices.push(line.len());
                }
                line.push(symbol);
            }

            tapes.push(line);
            blank_indices.push(tape_blank_indices);
        }
    }

    (tapes, blank_indices)
}

/// Creates a `TuringMachineError::ParseError` from a message and a `Span`.
fn parse_error(msg: &str, span: Span) -> TuringMachineError {
    TuringMachineError::ParseError(Box::new(Error::new_from_span(
        ErrorVariant::CustomError {
            message: msg.to_string(),
        },
        span,
    )))
}

/// Parses head position definitions from a `Pair<Rule::head>` or `Pair<Rule::heads>`.
///
/// If no head positions are explicitly defined, it defaults to `[0]` for each tape.
fn parse_heads(pair: Pair<Rule>) -> Vec<usize> {
    let mut positions = Vec::new();

    // Rule: (head | heads) > [index]
    for pos_pair in pair.into_inner() {
        if pos_pair.as_rule() == Rule::index {
            let pos = pos_pair.as_str().parse::<usize>().unwrap_or(0);
            positions.push(pos);
        }
    }

    if positions.is_empty() {
        positions.push(0);
    }

    positions
}

/// Parses the transition rules section from a `Pair<Rule::rules>`.
///
/// It extracts each state's transitions and sets the first encountered state as the initial state.
/// It also checks for duplicate transition rules for the same state.
fn parse_transitions(
    pair: Pair<Rule>,
    initial_state: &mut Option<String>,
) -> Result<HashMap<String, Vec<Transition>>, TuringMachineError> {
    let mut transitions = HashMap::new();

    for transition_pair in pair.into_inner() {
        let span = transition_pair.as_span();
        let (state, actions) = parse_multi_tape_transition(transition_pair)?;

        // Set first state as initial state
        if initial_state.is_none() {
            *initial_state = Some(state.clone());
        }

        // Prevent duplicated transition rule
        if transitions.contains_key(&state) {
            return Err(parse_error(
                &format!("Duplicate transition rule: {state}"),
                span,
            ));
        }

        transitions.insert(state, actions);
    }

    Ok(transitions)
}

/// Parses a single multi-tape transition rule from a `Pair<Rule::transition>`.
///
/// It extracts the source state and a list of actions (transitions) associated with that state.
fn parse_multi_tape_transition(
    pair: Pair<Rule>,
) -> Result<(String, Vec<Transition>), TuringMachineError> {
    let mut pairs = pair.into_inner();
    let state = parse_string(&mut pairs);
    let mut actions = Vec::new();

    for p in pairs {
        if p.as_rule() == Rule::action {
            for inner in p.into_inner() {
                match inner.as_rule() {
                    Rule::single_tape_action => {
                        // Convert single tape action to multi-tape format
                        let action = parse_single_tape_action(inner)?;
                        actions.push(Transition {
                            read: vec![action.read],
                            write: vec![action.write],
                            directions: vec![action.direction],
                            next_state: action.next,
                        });
                    }
                    Rule::multi_tape_action => {
                        actions.push(parse_multi_tape_action(inner)?);
                    }
                    _ => {} //
                }
            }
        }
    }

    Ok((state, actions))
}

/// Parses a single-tape action from a `Pair<Rule::single_tape_action>`.
///
/// It extracts the read symbol, write symbol (defaults to read if omitted), direction, and next state.
fn parse_single_tape_action(pair: Pair<Rule>) -> Result<ParsedAction, TuringMachineError> {
    let mut pairs = pair.into_inner();
    let read = parse_symbol_from_pairs(&mut pairs);

    // If `write` is omitted, we'll make `write` equal to `read`
    let write = match pairs.peek().unwrap().as_rule() {
        Rule::direction => read,
        _ => parse_symbol_from_pairs(&mut pairs),
    };

    let direction = parse_direction(pairs.next().unwrap())?;
    let next = parse_string(&mut pairs);

    Ok(ParsedAction {
        read,
        write,
        direction,
        next,
    })
}

/// Parses a multi-tape action from a `Pair<Rule::multi_tape_action>`.
///
/// It extracts the read symbols, write symbols, directions, and next state for all tapes.
/// It also validates that the number of read symbols, write symbols, and directions are consistent.
fn parse_multi_tape_action(pair: Pair<Rule>) -> Result<Transition, TuringMachineError> {
    let span = pair.as_span();
    let mut pairs = pair.into_inner();

    // Parse read symbols
    let read_symbols = parse_multi_tape_symbols(pairs.next().unwrap())?;

    // Parse write symbols (or use read symbols if omitted)
    let write_symbols = parse_multi_tape_symbols(pairs.next().unwrap())?;

    // Parse directions
    let directions = parse_directions(pairs.next().unwrap())?;

    // Parse next state
    let next_state = parse_string(&mut pairs);

    // Validate that all arrays have the same length
    if read_symbols.len() != write_symbols.len() || read_symbols.len() != directions.len() {
        return Err(parse_error(
            &format!(
                "Inconsistent multi-tape action: read={}, write={}, directions={}",
                read_symbols.len(),
                write_symbols.len(),
                directions.len()
            ),
            span,
        ));
    }

    Ok(Transition {
        read: read_symbols,
        write: write_symbols,
        directions,
        next_state,
    })
}

/// Parses a list of multi-tape symbols from a `Pair<Rule::multi_tape_symbols>`.
fn parse_multi_tape_symbols(pair: Pair<Rule>) -> Result<Vec<char>, TuringMachineError> {
    let mut symbols = Vec::new();

    for symbol_pair in pair.into_inner() {
        if symbol_pair.as_rule() == Rule::symbol {
            symbols.push(parse_symbol(symbol_pair.as_str()));
        }
    }

    Ok(symbols)
}

/// Parses a list of directions from a `Pair<Rule::directions>`.
fn parse_directions(pair: Pair<Rule>) -> Result<Vec<Direction>, TuringMachineError> {
    let mut directions = Vec::new();

    for dir_pair in pair.into_inner() {
        if dir_pair.as_rule() == Rule::direction {
            directions.push(parse_direction(dir_pair)?);
        }
    }

    Ok(directions)
}

/// Parses a single direction from a `Pair<Rule::direction>`.
///
/// Supports '<' or 'L' for Left, '>' or 'R' for Right, and '-' or 'S' for Stay.
fn parse_direction(pair: Pair<Rule>) -> Result<Direction, TuringMachineError> {
    let span = pair.as_span();
    match pair.as_str() {
        "<" | "L" => Ok(Direction::Left),
        ">" | "R" => Ok(Direction::Right),
        "-" | "S" => Ok(Direction::Stay),
        _ => Err(parse_error(
            &format!("Unsupported direction: {}", pair.as_str()),
            span,
        )),
    }
}

/// Parses a single character symbol from a string, handling quoted and unquoted symbols.
fn parse_symbol(input: &str) -> char {
    input
        .trim_matches('\'')
        .chars()
        .next()
        .unwrap_or(DEFAULT_BLANK_SYMBOL)
}

/// Parses a single character symbol from a `Pairs` iterator.
fn parse_symbol_from_pairs(pairs: &mut Pairs<Rule>) -> char {
    parse_symbol(&parse_string(pairs))
}

/// Extracts the inner string content from a `Pair`.
fn parse_inner_string(pair: Pair<Rule>) -> String {
    pair.into_inner().next().unwrap().as_str().into()
}

/// Extracts the string content from the current `Pair` in a `Pairs` iterator.
fn parse_string(pairs: &mut Pairs<Rule>) -> String {
    pairs.next().unwrap().as_str().into()
}

/// Checks if a given rule has already been declared, ensuring uniqueness for top-level sections.
fn check_unique_rule(
    rule: Rule,
    span: Span,
    seen: &mut HashSet<Rule>,
) -> Result<(), TuringMachineError> {
    if !matches!(
        rule,
        Rule::name
            | Rule::blank
            | Rule::tape
            | Rule::tapes
            | Rule::head
            | Rule::heads
            | Rule::rules
    ) {
        return Ok(());
    };

    if seen.contains(&rule) {
        return Err(parse_error(
            &format!("Duplicate \"{rule:?}:\" declaration"),
            span,
        ));
    }

    seen.insert(rule);

    Ok(())
}

/// Checks if an exclusive rule (e.g., `tape` vs. `tapes`) has been violated.
fn check_exclusive_rule<T>(
    value: Option<T>,
    names: Vec<&str>,
    span: Span,
) -> Result<(), TuringMachineError> {
    if value.is_some() {
        return Err(parse_error(
            &format!("Only one of {} is allowed", format_rules(names)),
            span,
        ));
    }

    Ok(())
}

/// Checks if a required rule is present, returning an `Err` if it's missing.
fn check_required_rule<T>(value: Option<T>, names: Vec<&str>) -> Result<T, TuringMachineError> {
    value.ok_or_else(|| {
        TuringMachineError::ValidationError(format!("Missing {} section", format_rules(names)))
    })
}

/// Checks for consistency between the number of head positions and the number of tapes.
fn check_head_tape_consistency(heads: &[usize], tapes: &[Tape]) -> Result<(), TuringMachineError> {
    if heads.len() != tapes.len() {
        return Err(TuringMachineError::ValidationError(format!(
            "Number of head positions ({}) does not match number of tapes ({})",
            heads.len(),
            tapes.len()
        )));
    }
    Ok(())
}

/// Replace INPUT_BLANK_SYMBOL with the actual blank symbol in tapes using collected indices
fn rewrite_tapes(
    (mut tapes, blank_indices): (Vec<Tape>, Vec<Vec<usize>>),
    blank: char,
) -> Vec<Tape> {
    for (tape_idx, indices) in blank_indices.into_iter().enumerate() {
        for idx in indices {
            tapes[tape_idx][idx] = blank;
        }
    }

    tapes
}

/// Formats a list of rule names into a human-readable string for error messages.
fn format_rules(names: Vec<&str>) -> String {
    names
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(" or ")
}

/// A helper struct to temporarily hold parsed single-tape action data.
struct ParsedAction {
    read: char,
    write: char,
    direction: Direction,
    next: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_program() {
        let input = r#"
name: Simple Test
tape: a
rules:
  start:
    a -> b, R, halt
  halt:
"#;

        let result = parse(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.name, "Simple Test");
        assert_eq!(program.initial_tape(), "a");
        assert!(program.rules.contains_key("start"));
        assert!(program.rules.contains_key("halt"));
    }

    #[test]
    fn test_parse_simple_multi_tape_program() {
        let input = r#"
name: Simple Multi-Tape
heads: [0, 0]
tapes:
  [a]
  [d]
rules:
  start:
    [a, d] -> [b, e], [R, R], halt
  halt:
"#;

        let result = parse(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.name, "Simple Multi-Tape");
        assert_eq!(program.tapes, vec!["a", "d"]);
        assert_eq!(
            program.rules["start"][0],
            Transition {
                read: vec!['a', 'd'],
                write: vec!['b', 'e'],
                directions: vec![Direction::Right, Direction::Right],
                next_state: "halt".into(),
            }
        );
        assert!(program.rules.contains_key("start"));
        assert!(program.rules.contains_key("halt"));
    }

    #[test]
    fn test_parse_duplicate_section() {
        let input = r#"
name: First Name
name: Second Name
tape: a
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error
            .to_string()
            .contains("Duplicate \"name:\" declaration"));
    }

    #[test]
    fn test_parse_missing_name() {
        let input = r#"
tape: a
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
    }

    #[test]
    fn test_parse_missing_tape() {
        let input = r#"
name: Missing Tape
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ValidationError(_)));
        assert_eq!(
            error.to_string(),
            "Program validation error: Missing 'tape' or 'tapes' section"
        );
    }

    #[test]
    fn test_parse_missing_transitions() {
        let input = r#"
name: Missing Transitions
tape: a
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ValidationError(_)));
        assert_eq!(
            error.to_string(),
            "Program validation error: Missing 'rules' section"
        );
    }

    #[test]
    fn test_parse_exclusive_tape_and_tapes() {
        let input = r#"
name: Exclusive Tapes
tape: a
tapes:
  [b]
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error
            .to_string()
            .contains("Only one of 'tape' or 'tapes' is allowed"));
    }

    #[test]
    fn test_parse_exclusive_head_and_heads() {
        let input = r#"
name: Exclusive Heads
head: 0
heads: [1]
tape: a
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error
            .to_string()
            .contains("Only one of 'head' or 'heads' is allowed"));
    }

    #[test]
    fn test_parse_mismatched_heads_and_tapes() {
        let input = r#"
name: Mismatched
heads: [0, 1]
tapes:
  [a]
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ValidationError(_)));
        assert_eq!(
            error.to_string(),
            "Program validation error: Number of head positions (2) does not match number of tapes (1)"
        );
    }

    #[test]
    fn test_parse_duplicate_transition_rule() {
        let input = r#"
name: Duplicate Transition
tape: a
rules:
  start:
    a -> b, R, halt
  start:
    b -> a, L, start
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error
            .to_string()
            .contains("Duplicate transition rule: start"));
    }

    #[test]
    fn test_parse_inconsistent_multi_tape_action() {
        let input = r#"
name: Inconsistent Action
tapes:
  [a]
  [b]
rules:
  start:
    [a, b] -> [c], [R, R], halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error
            .to_string()
            .contains("Inconsistent multi-tape action: read=2, write=1, directions=2"));
    }

    #[test]
    fn test_parse_unsupported_direction() {
        let input = r#"
name: Bad Direction
tape: a
rules:
  start:
    a -> b, X, halt
"#;
        let result = parse(input);
        assert!(result.is_err());
        if let Err(e) = &result {
            eprintln!("Parsing error for unsupported direction: {:?}", e);
        }
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
    }

    #[test]
    fn test_parse_with_custom_blank() {
        let input = r#"
name: Custom Blank
blank: '_'
tape: a
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.blank, '_');
    }

    #[test]
    fn test_parse_with_default_blank() {
        let input = r#"
name: Default Blank
tape: a
rules:
  start:
    a -> b, R, halt
"#;
        let result = parse(input);
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.blank, DEFAULT_BLANK_SYMBOL);
    }

    #[test]
    fn test_parse_with_single_head() {
        let input = r#"
name: Single Head
head: 5
tape: "123456789"
rules:
  start:
    '6' -> 'X', R, halt
"#;
        let result = parse(input);
        if let Err(e) = &result {
            eprintln!("Parsing error for single head: {:?}", e);
        }
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, TuringMachineError::ParseError(_)));
        assert!(error.to_string().contains("tape"));
    }

    #[test]
    fn test_parse_omitted_write_symbol() {
        let input = r#"
name: Omitted Write
tape: a
rules:
  start:
    a, R, halt
"#;
        let result = parse(input);
        if let Err(e) = &result {
            eprintln!("Parsing error: {:?}", e);
        }
        assert!(result.is_ok());
        let program = result.unwrap();
        let transition = &program.rules["start"][0];
        assert_eq!(transition.read, vec!['a']);
        assert_eq!(transition.write, vec!['a']); // Should write what it read
        assert_eq!(transition.directions, vec![Direction::Right]);
        assert_eq!(transition.next_state, "halt");
    }

    #[test]
    fn test_parse_tape_with_blank_symbol() {
        let input = r#"
name: Blank Tape Test
tape: a, _
rules:
  start:
    a -> a, R, halt
  halt:
"#;

        let result = parse(input);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(
            program.tapes[0].chars().nth(1).unwrap(),
            DEFAULT_BLANK_SYMBOL
        );
    }
}
