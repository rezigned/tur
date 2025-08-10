mod app;

use action::Action;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Read;
use std::{error::Error, fs, io, time::Duration};

/// A Turing Machine simulator with a Terminal User Interface.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(after_help = "EXAMPLES:
  tur-tui examples/simple.tur
  cat examples/binary-addition.tur | tur-tui")]
struct Cli {
    /// Path to a Turing machine program file (.tur).
    /// If not provided, the application will load built-in example programs.
    /// Can also pipe program content via stdin.
    program_file: Option<String>,
}

/// Represents the state of the application loop.
#[derive(PartialEq)]
enum AppState {
    Running,
    ShouldQuit,
}

/// A wrapper around the terminal to ensure it's restored on drop.
struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    /// Creates a new TUI.
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Restore the terminal to its original state.
        // The results are ignored as we can't do much about errors during drop.
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Load the program before initializing the TUI.
    // This way, if loading fails, we can print the error to stderr without
    // interfering with the terminal's alternate screen.
    let app = match load_program(&cli) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize the TUI. The `Tui` struct will handle cleanup on drop.
    let mut tui = Tui::new()?;

    // Run the application.
    run_app(&mut tui.terminal, app)?;

    Ok(())
}

/// Loads a Turing machine program based on CLI arguments.
///
/// It tries to load from a file path, then from stdin, and finally
/// falls back to the default built-in programs.
fn load_program(cli: &Cli) -> Result<App, String> {
    if let Some(file_path) = &cli.program_file {
        fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))
            .and_then(App::new_from_program_string)
    } else if atty::isnt(atty::Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| format!("Failed to read from stdin: {}", e))
            .and_then(|_| App::new_from_program_string(buffer))
    } else {
        Ok(App::new_default())
    }
}

/// Runs the main application loop.
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.render(f))?;

        let timeout = if app.is_auto_playing() {
            Duration::from_millis(500) // Faster updates during auto-play
        } else {
            Duration::from_millis(100) // Slower updates when idle
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_key_event(&mut app, key) == AppState::ShouldQuit {
                    return Ok(());
                }
            }
        }

        if app.is_auto_playing() {
            app.step_machine();
        }
    }
}

/// Handles key events and updates the application state.
fn handle_key_event(app: &mut App, key: KeyEvent) -> AppState {
    if let Some(action) = app.keymap.get(&key) {
        match action {
            Action::Quit => return AppState::ShouldQuit,
            Action::Reset => app.reset_machine(),
            Action::Step => app.step_machine(),
            Action::ToggleAutoPlay => app.toggle_auto_play(),
            Action::ToggleHelp => app.toggle_help(),
            Action::PreviousProgram => app.previous_program(),
            Action::NextProgram => app.next_program(),
        }
    }
    AppState::Running
}
