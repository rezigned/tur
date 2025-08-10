use clap::Parser;
use std::path::Path;
use tur::loader::ProgramLoader;
use tur::machine::TuringMachine;
use tur::ExecutionResult;

#[derive(Parser)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// The Turing machine program file to execute
    #[clap(short, long)]
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

    let program = ProgramLoader::load_program(Path::new(&cli.program)).unwrap();
    let mut machine = TuringMachine::new(&program);

    for (i, input_str) in cli.input.iter().enumerate() {
        if i < machine.tapes.len() {
            machine.tapes[i] = input_str.chars().collect();
        }
    }

    if cli.debug {
        let print_state = |machine: &TuringMachine| {
            let tapes_str = machine
                .get_tapes_as_strings()
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            println!(
                "Step: {}, State: {}, Tapes: [{}], Heads: {:?}",
                machine.get_step_count(),
                machine.get_state(),
                tapes_str,
                machine.get_head_positions()
            );
        };

        print_state(&machine);

        loop {
            match machine.step() {
                ExecutionResult::Continue => {
                    print_state(&machine);
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
    } else {
        machine.run_to_completion();
    }

    println!("{}", machine.get_tapes_as_strings().join("\n"));
}
