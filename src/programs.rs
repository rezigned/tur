use crate::types::{Program, TuringMachineError};

use std::sync::RwLock;

// Default embedded programs
const PROGRAM_TEXTS: [&str; 9] = [
    include_str!("../examples/binary-addition.tur"),
    include_str!("../examples/palindrome.tur"),
    include_str!("../examples/binary-counter.tur"),
    include_str!("../examples/event-number-checker.tur"),
    include_str!("../examples/subtraction.tur"),
    include_str!("../examples/busy-beaver-3.tur"),
    include_str!("../examples/multi-tape-copy.tur"),
    include_str!("../examples/multi-tape-addition.tur"),
    include_str!("../examples/multi-tape-compare.tur"),
];

lazy_static::lazy_static! {
    pub static ref PROGRAMS: RwLock<Vec<Program>> = RwLock::new(Vec::new());
}

pub struct ProgramManager;

impl ProgramManager {
    /// Initialize the ProgramManager with programs from the specified directory
    pub fn load() -> Result<(), TuringMachineError> {
        // Load embedded programs first
        let mut programs = Vec::new();

        for program_text in PROGRAM_TEXTS {
            if let Ok(program) = crate::parser::parse(program_text) {
                programs.push(program);
            } else {
                eprintln!("Failed to parse program");
            }
        }

        // Store the loaded programs and their texts
        if let Ok(mut write_guard) = PROGRAMS.write() {
            *write_guard = programs;
        } else {
            return Err(TuringMachineError::FileError(
                "Failed to acquire write lock".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the number of available programs
    pub fn get_program_count() -> usize {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAMS.read().map(|programs| programs.len()).unwrap_or(0)
    }

    /// Get a program by its index
    pub fn get_program_by_index(index: usize) -> Result<Program, TuringMachineError> {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAMS
            .read()
            .map_err(|_| TuringMachineError::FileError("Failed to acquire read lock".to_string()))?
            .get(index)
            .cloned()
            .ok_or_else(|| {
                TuringMachineError::ValidationError(format!("Program index {} out of range", index))
            })
    }

    /// Get a program by its name
    pub fn get_program_by_name(name: &str) -> Result<Program, TuringMachineError> {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAMS
            .read()
            .map_err(|_| TuringMachineError::FileError("Failed to acquire read lock".to_string()))?
            .iter()
            .find(|program| program.name == name)
            .cloned()
            .ok_or_else(|| {
                TuringMachineError::ValidationError(format!("Program '{}' not found", name))
            })
    }

    /// List all program names
    pub fn list_program_names() -> Vec<String> {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAMS
            .read()
            .map(|programs| {
                programs
                    .iter()
                    .map(|program| program.name.clone())
                    .collect()
            })
            .unwrap_or_else(|_| Vec::new())
    }

    /// Get information about a program by its index
    pub fn get_program_info(index: usize) -> Result<ProgramInfo, TuringMachineError> {
        let program = Self::get_program_by_index(index)?;

        Ok(ProgramInfo {
            index,
            name: program.name.clone(),
            initial_state: program.initial_state.clone(),
            initial_tape: program.initial_tape(),
            state_count: program.rules.len(),
            transition_count: program
                .rules
                .values()
                .map(|transitions| transitions.len())
                .sum(),
        })
    }

    /// Search for programs by name
    pub fn search_programs(query: &str) -> Vec<usize> {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAMS
            .read()
            .map(|programs| {
                programs
                    .iter()
                    .enumerate()
                    .filter(|(_, program)| {
                        program.name.to_lowercase().contains(&query.to_lowercase())
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_else(|_| Vec::new())
    }

    /// Get the original text of a program by its index
    pub fn get_program_text_by_index(index: usize) -> Result<&'static str, TuringMachineError> {
        // Initialize with default programs if not already initialized
        let _ = Self::load();

        PROGRAM_TEXTS.get(index).cloned().ok_or_else(|| {
            TuringMachineError::ValidationError(format!(
                "Program text index {} out of range",
                index
            ))
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    pub index: usize,
    pub name: String,
    pub initial_state: String,
    pub initial_tape: String,
    pub state_count: usize,
    pub transition_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::TuringMachine;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_program_manager_initialization() {
        // Initialize with default programs
        let result = ProgramManager::load();
        assert!(result.is_ok());

        // Check that we have the expected number of programs
        assert!(ProgramManager::get_program_count() >= 4);
    }

    #[test]
    fn test_program_manager_with_custom_directory() {
        let dir = tempdir().unwrap();

        // Create a custom program file
        let file_path = dir.path().join("custom.tur");
        let content = r#"
name: Custom Program
tape: x, y, z
rules:
  start:
    x -> y, R, stop
  stop:"#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        // Test that ProgramLoader can load the file directly
        let program = crate::loader::ProgramLoader::load_program(&file_path);
        assert!(program.is_ok());

        let program = program.unwrap();
        assert_eq!(program.name, "Custom Program");
        assert_eq!(program.initial_tape(), "xyz");

        // Test that ProgramLoader can load from directory
        let results = crate::loader::ProgramLoader::load_programs(dir.path());
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[test]
    fn test_all_programs_are_valid() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let count = ProgramManager::get_program_count();
        for i in 0..count {
            let program = ProgramManager::get_program_by_index(i).unwrap();
            assert!(
                TuringMachine::validate_program(&program).is_ok(),
                "Program '{}' is invalid",
                program.name
            );
        }
    }

    #[test]
    fn test_program_names() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let names = ProgramManager::list_program_names();
        assert!(names.contains(&"Binary addition".to_string()));
        assert!(names.contains(&"Palindrome Checker".to_string()));
        assert!(names.contains(&"Binary Counter".to_string()));
        assert!(names.contains(&"Subtraction".to_string()));
    }

    #[test]
    fn test_programs_can_be_executed() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let count = ProgramManager::get_program_count();
        for i in 0..count {
            let program = ProgramManager::get_program_by_index(i).unwrap();
            let program_name = program.name.clone();
            let mut machine = TuringMachine::new(program);
            let result = machine.step();

            // Should either continue or halt, but not error on first step
            match result {
                crate::types::ExecutionResult::Continue => {}
                crate::types::ExecutionResult::Halt => {}
                crate::types::ExecutionResult::Error(e) => {
                    panic!("Program '{}' failed on first step: {}", program_name, e);
                }
            }
        }
    }

    #[test]
    fn test_program_manager_get_program_by_index() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let program = ProgramManager::get_program_by_index(0);
        assert!(program.is_ok());

        let result = ProgramManager::get_program_by_index(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_program_manager_get_program_by_name() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let program = ProgramManager::get_program_by_name("Binary addition");
        assert!(program.is_ok());
        assert_eq!(program.unwrap().initial_tape(), "$00111-");

        let result = ProgramManager::get_program_by_name("Nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_program_manager_list_program_names() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let names = ProgramManager::list_program_names();
        assert!(names.len() >= 4);
        assert!(names.contains(&"Binary addition".to_string()));
        assert!(names.contains(&"Palindrome Checker".to_string()));
        assert!(names.contains(&"Binary Counter".to_string()));
        assert!(names.contains(&"Subtraction".to_string()));
    }

    #[test]
    fn test_program_manager_get_program_info() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let info = ProgramManager::get_program_info(0);
        assert!(info.is_ok());

        let info = info.unwrap();
        assert_eq!(info.index, 0);
        assert!(!info.name.is_empty());
        assert!(!info.initial_tape.is_empty());
        assert!(info.state_count > 0);
        assert!(info.transition_count > 0);

        let result = ProgramManager::get_program_info(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_program_manager_search_programs() {
        // Initialize with default programs
        let _ = ProgramManager::load();

        let results = ProgramManager::search_programs("binary");
        assert!(results.len() >= 2); // "Binary addition" and "Binary counter"

        let results = ProgramManager::search_programs("palindrome");
        assert!(!results.is_empty());

        let results = ProgramManager::search_programs("nonexistent");
        assert_eq!(results.len(), 0);
    }
}
