use clap::Parser;
use std::io::{self, BufRead};
use std::path::Path;
use tur::loader::ProgramLoader;
use tur::machine::TuringMachine;
use tur::ExecutionResult;

#[derive(Parser)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// The Turing machine program file to execute
    program: String,

    /// The input to the Turing machine
    #[clap(short, long)]
    input: Vec<String>,

    /// Print each step of the execution
    #[clap(short = 'd', long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    let program = match ProgramLoader::load_program(Path::new(&cli.program)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading program: {}", e);
            std::process::exit(1);
        }
    };
    let mut machine = TuringMachine::new(program);

    // Get tape inputs from either CLI args or stdin
    let tapes = match read_tape_inputs(&cli.input) {
        Ok(inputs) => inputs,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Set tape contents if any inputs were provided
    if !tapes.is_empty() {
        if let Err(e) = machine.set_tapes_content(&tapes) {
            eprintln!("Error setting tape content: {}", e);
            std::process::exit(1);
        }
    }

    if cli.debug {
        run_with_debug(&mut machine);
    } else {
        machine.run_to_completion();
    }

    println!("{}", machine.get_tapes_as_strings().join("\n"));
}

/// Runs the Turing machine with debug output, printing each step.
fn run_with_debug(machine: &mut TuringMachine) {
    let print_state = |machine: &TuringMachine| {
        let tapes_str = machine
            .get_tapes_as_strings()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        println!(
            "Step: {}, State: {}, Tapes: [{}], Heads: {:?}",
            machine.step_count(),
            machine.state(),
            tapes_str,
            machine.head_positions()
        );
    };

    print_state(machine);

    loop {
        match machine.step() {
            ExecutionResult::Continue => {
                print_state(machine);
            }
            ExecutionResult::Halt => {
                println!("\nMachine halted.");
                break;
            }
            ExecutionResult::Error(e) => {
                println!("\nMachine error: {}", e);
                break;
            }
        }
    }
    println!("\nFinal tapes:");
}

/// Gets tape input from either command line arguments or stdin.
/// Returns a vector of strings representing the content for each tape.
fn read_tape_inputs(inputs: &[String]) -> Result<Vec<String>, String> {
    if !inputs.is_empty() {
        // Use command line inputs
        Ok(inputs.to_vec())
    } else if !atty::is(atty::Stream::Stdin) {
        // Read from stdin, each line represents a tape
        let stdin = io::stdin();
        let mut tape_inputs = Vec::new();

        for line in stdin.lock().lines() {
            match line {
                Ok(content) => tape_inputs.push(content.trim().to_string()),
                Err(e) => return Err(format!("Error reading from stdin: {}", e)),
            }
        }

        Ok(tape_inputs)
    } else {
        // No input provided
        Ok(Vec::new())
    }
}
