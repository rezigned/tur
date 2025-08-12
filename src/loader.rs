//! This module provides the `ProgramLoader` struct, responsible for loading Turing Machine
//! programs from various sources, including files and strings.

use crate::parser::parse;
use crate::types::{Program, TuringMachineError};
use std::fs;
use std::path::{Path, PathBuf};

/// `ProgramLoader` is a utility struct for loading Turing Machine programs.
/// It provides methods to load programs from individual files, from string content,
/// and to discover and load all `.tur` files within a specified directory.
pub struct ProgramLoader;

impl ProgramLoader {
    /// Loads a single Turing Machine program from the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the `Path` of the `.tur` file to load.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` if the file is successfully read and parsed into a `Program`.
    /// * `Err(TuringMachineError::FileError)` if the file cannot be read.
    /// * `Err(TuringMachineError::ParseError)` if the file content is not a valid program.
    pub fn load_program(path: &Path) -> Result<Program, TuringMachineError> {
        let content = fs::read_to_string(path).map_err(|e| {
            TuringMachineError::FileError(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        parse(&content)
    }

    /// Loads a single Turing Machine program from the provided string content.
    ///
    /// This is useful for parsing programs that are not stored in files, e.g., from user input.
    ///
    /// # Arguments
    ///
    /// * `content` - A string slice containing the Turing Machine program definition.
    ///
    /// # Returns
    ///
    /// * `Ok(Program)` if the content is successfully parsed into a `Program`.
    /// * `Err(TuringMachineError::ParseError)` if the content is not a valid program.
    pub fn load_program_from_string(content: &str) -> Result<Program, TuringMachineError> {
        parse(content)
    }

    /// Loads all valid Turing Machine program files (`.tur` extension) from a given directory.
    ///
    /// It iterates through the directory, attempts to load each `.tur` file, and collects
    /// the results. Directories and non-`.tur` files are skipped.
    ///
    /// # Arguments
    ///
    /// * `directory` - A reference to the `Path` of the directory to scan for programs.
    ///
    /// # Returns
    ///
    /// * `Vec<Result<(PathBuf, Program), TuringMachineError>>` - A vector where each element
    ///   is a `Result` indicating whether a program was successfully loaded (containing its
    ///   path and the `Program` itself) or if an error occurred during loading (containing
    ///   a `TuringMachineError`).
    pub fn load_programs(directory: &Path) -> Vec<Result<(PathBuf, Program), TuringMachineError>> {
        if !directory.exists() {
            return vec![Err(TuringMachineError::FileError(format!(
                "Directory {} does not exist",
                directory.display()
            )))];
        }

        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(e) => {
                return vec![Err(TuringMachineError::FileError(format!(
                    "Failed to read directory {}: {}",
                    directory.display(),
                    e
                )))]
            }
        };

        entries
            .filter_map(|entry| {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        return Some(Err(TuringMachineError::FileError(format!(
                            "Failed to read directory entry: {}",
                            e
                        ))))
                    }
                };

                let path = entry.path();

                // Skip directories and non-.tur files
                if path.is_dir() || path.extension().is_none_or(|ext| ext != "tur") {
                    return None;
                }

                match Self::load_program(&path) {
                    Ok(program) => Some(Ok((path, program))),
                    Err(e) => Some(Err(TuringMachineError::FileError(format!(
                        "Failed to load program from {}: {}",
                        path.display(),
                        e
                    )))),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_valid_program() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.tur");

        let program_content =
            "name: Test Program\ntape: a\nrules:\n  start:\n    a -> b, R, stop\n  stop:";

        let mut file = File::create(&file_path).unwrap();
        file.write_all(program_content.as_bytes()).unwrap();

        let result = ProgramLoader::load_program(&file_path);
        assert!(result.is_ok());

        let program = result.unwrap();
        assert_eq!(program.name, "Test Program");
        assert_eq!(program.initial_tape(), "a");
        assert!(program.rules.contains_key("start"));
        assert!(program.rules.contains_key("stop"));
    }

    #[test]
    fn test_load_invalid_program() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.tur");

        let invalid_content = "This is not a valid program";

        let mut file = File::create(&file_path).unwrap();
        file.write_all(invalid_content.as_bytes()).unwrap();

        let result = ProgramLoader::load_program(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_programs_from_directory() {
        let dir = tempdir().unwrap();

        // Create a valid program file
        let valid_path = dir.path().join("valid.tur");
        let valid_content =
            "name: Valid Program\ntape: a\nrules:\n  start:\n    a -> b, R, stop\n  stop:";
        let mut valid_file = File::create(&valid_path).unwrap();
        valid_file.write_all(valid_content.as_bytes()).unwrap();

        // Create an invalid program file
        let invalid_path = dir.path().join("invalid.tur");
        let invalid_content = "This is not a valid program";
        let mut invalid_file = File::create(&invalid_path).unwrap();
        invalid_file.write_all(invalid_content.as_bytes()).unwrap();

        // Create a non-.tur file that should be ignored
        let ignored_path = dir.path().join("ignored.txt");
        let ignored_content = "This file should be ignored";
        let mut ignored_file = File::create(&ignored_path).unwrap();
        ignored_file.write_all(ignored_content.as_bytes()).unwrap();

        let results = ProgramLoader::load_programs(dir.path());

        // We should have 2 results: 1 success and 1 error
        assert_eq!(results.len(), 2);

        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }

        assert_eq!(success_count, 1);
        assert_eq!(error_count, 1);
    }
}
