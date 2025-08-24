use action::Action;
use keymap::{Config, KeyMapConfig};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
    Frame,
};
use tur::{
    types::{DEFAULT_BLANK_SYMBOL, INPUT_BLANK_SYMBOL},
    Program, ProgramLoader, ProgramManager, Step, TuringMachine,
};

const BLOCK_PADDING: Padding = Padding::new(1, 1, 0, 0);

pub struct App {
    machine: TuringMachine,
    current_program_index: usize,
    auto_play: bool,
    scroll_offset: usize,
    message: String,
    show_help: bool,
    pub(crate) keymap: Config<Action>,
    // Indicates if the program was loaded from a file/stdin, disabling program switching
    program_loaded_from_source: bool,
    program_content: String,
}

impl App {
    pub fn new_default() -> Self {
        let program = ProgramManager::get_program_by_index(0).unwrap();
        let program_content = ProgramManager::get_program_text_by_index(0)
            .unwrap()
            .to_string();
        let machine = TuringMachine::new(program);

        Self {
            machine,
            keymap: Action::keymap_config(),
            current_program_index: 0,
            auto_play: false,
            scroll_offset: 0,
            message: "Press 'h' for help.".to_string(),
            show_help: false,
            program_loaded_from_source: false,
            program_content,
        }
    }

    pub fn new_from_program_string(program_content: String) -> Result<Self, String> {
        let program: Program = ProgramLoader::load_program_from_string(&program_content)
            .map_err(|e| format!("Failed to load program: {}", e))?;
        let machine = TuringMachine::new(program);

        Ok(Self {
            machine,
            keymap: Action::keymap_config(),
            current_program_index: 0, // Not relevant for single program, but keep for consistency
            auto_play: false,
            scroll_offset: 0,
            message: "Program loaded from source. Press 'h' for help.".to_string(),
            show_help: false,
            program_loaded_from_source: true,
            program_content,
        })
    }

    pub fn render(&mut self, f: &mut Frame) {
        let margin_size = Margin::new(1, 0); // Define margin size
        let inner_area = f.area().inner(margin_size);

        // Main vertical chunks: Program Info, Middle (Source + Machine), Status
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Program info (fixed height + margin)
                Constraint::Min(0),    // Middle section (flexible height)
                Constraint::Length(3), // Status/controls (fixed height + margin)
            ])
            .split(inner_area);

        // Render Program Info in the top main chunk
        self.render_program_info(f, main_chunks[0]);

        // Middle horizontal chunks: Source Code (left), Machine (right)
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Source code
                Constraint::Length(1),
                Constraint::Percentage(50), // Machine
            ])
            .split(main_chunks[1]);

        self.render_source_code(f, middle_chunks[0]);

        // Right vertical chunks: Machine State, Tapes/Help
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Machine State
                Constraint::Min(0),    // Tapes
            ])
            .split(middle_chunks[2]);

        self.render_machine_state(f, right_chunks[0]);

        if self.show_help {
            self.render_help(f, right_chunks[1]);
        } else {
            self.render_tapes(f, right_chunks[1]);
        }

        // Render Status in the bottom main chunk
        self.render_status(f, main_chunks[2]);
    }

    fn render_source_code(&self, f: &mut Frame, area: Rect) {
        let keywords = [
            "name:", "mode:", "head:", "heads:", "blank:", "tape:", "tapes:", "states:", "rules:",
        ];

        let mut lines = Vec::new();
        for line in self.program_content.lines() {
            let mut spans = Vec::new();
            let mut parts = line.split_whitespace();
            if let Some(first_word) = parts.next() {
                if keywords.contains(&first_word) {
                    spans.push(Span::styled(first_word, Style::default().fg(Color::Yellow)));
                    spans.push(Span::raw(" "));
                    spans.push(Span::raw(parts.collect::<Vec<_>>().join(" ")));
                } else {
                    spans.push(Span::raw(line));
                }
            } else {
                spans.push(Span::raw(line));
            }
            lines.push(Line::from(spans));
        }

        let paragraph = section("Source Code", lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    fn render_program_info(&self, f: &mut Frame, area: Rect) {
        let program = ProgramManager::get_program_by_index(self.current_program_index).unwrap();
        let info = ProgramManager::get_program_info(self.current_program_index).unwrap();
        let tape_count = program.tapes.len();

        let mut text = vec![Line::from(vec![
            Span::styled("Program: ", Style::default().fg(Color::Yellow)),
            Span::raw(if self.program_loaded_from_source {
                format!("{} (Custom)", program.name)
            } else {
                format!(
                    "{} ({}/{})",
                    program.name,
                    self.current_program_index + 1,
                    ProgramManager::count()
                )
            }),
        ])];

        // Always show tapes in array format for consistency
        let tapes_preview: String = program
            .tapes
            .iter()
            .take(3) // Show first 3 tapes
            .map(|tape| format!("\"{}\"", tape))
            .collect::<Vec<_>>()
            .join(", ");
        let tapes_display = if tape_count > 3 {
            format!("{}, ... ({} tapes)", tapes_preview, tape_count)
        } else {
            format!("{} ({} tapes)", tapes_preview, tape_count)
        };

        text.push(Line::from(vec![
            Span::styled("Initial Tapes: ", Style::default().fg(Color::Yellow)),
            Span::raw(tapes_display),
        ]));

        text.push(Line::from(vec![
            Span::styled("States: ", Style::default().fg(Color::Yellow)),
            Span::raw(info.state_count.to_string()),
            Span::styled(" | Rules: ", Style::default().fg(Color::Yellow)),
            Span::raw(info.transition_count.to_string()),
        ]));

        let paragraph = Paragraph::new(text)
            .block(block("Tur - Turing Machine Language (TUI)").title_alignment(Alignment::Center));

        f.render_widget(paragraph, area);
    }

    fn render_tapes(&self, f: &mut Frame, area: Rect) {
        let tapes = self.machine.tapes();
        let head_positions = self.machine.heads();
        let tape_count = tapes.len();

        let mut text_lines = Vec::new();

        for (tape_idx, (tape, &head_pos)) in tapes.iter().zip(head_positions.iter()).enumerate() {
            // Always show tape number for consistency (even for single tape)
            text_lines.push(Line::from(vec![Span::styled(
                format!("Tape {}: ", tape_idx + 1),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]));

            // Create tape visualization
            let mut tape_spans = Vec::new();
            for (i, &symbol) in tape.iter().enumerate() {
                // Fix span render blank char incorrectly (it renders in a new line).
                let symbol = if symbol == DEFAULT_BLANK_SYMBOL {
                    INPUT_BLANK_SYMBOL
                } else {
                    symbol
                };

                if i == head_pos {
                    // For head position, use brackets to make highlighting more visible
                    tape_spans.push(Span::styled(
                        format!(" {symbol} "),
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    // Normal cells with standard formatting
                    tape_spans.push(Span::styled(format!(" {symbol} "), Style::default()));
                }
            }

            text_lines.push(Line::from(tape_spans));

            // Head position indicator (consistent format)
            let current_symbol = if head_pos < tape.len() {
                tape[head_pos]
            } else {
                self.machine.blank()
            };
            let head_indicator = format!(
                "Head at position: {} (symbol: '{}')",
                head_pos, current_symbol
            );

            text_lines.push(Line::from(Span::styled(
                head_indicator,
                Style::default().fg(Color::Cyan),
            )));

            // Add spacing between tapes (except for the last one)
            if tape_idx < tape_count - 1 {
                text_lines.push(Line::from(""));
            }
        }

        // Always use "Tapes" title for consistency
        let paragraph = section("Tapes", text_lines).wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    fn render_machine_state(&self, f: &mut Frame, area: Rect) {
        let state = self.machine.state();
        let step_count = self.machine.step_count();
        let is_halted = self.machine.is_halted();

        let (status_text, status_color) = if is_halted {
            ("HALTED", Color::Red)
        } else if step_count == 0 {
            ("READY", Color::Blue)
        } else {
            ("RUNNING", Color::Green)
        };

        let mut text = vec![Line::from(vec![
            Span::styled("Current State: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                state,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | Steps: ", Style::default().fg(Color::Yellow)),
            Span::raw(step_count.to_string()),
            Span::styled(" | Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(status_text, Style::default().fg(status_color)),
        ])];

        // Always use array format for symbols (consistent for single and multi-tape)
        let current_symbols = self.machine.symbols();
        let symbols_str: String = current_symbols
            .iter()
            .map(|&c| format!("'{}'", c))
            .collect::<Vec<_>>()
            .join(", ");

        text.push(Line::from(vec![
            Span::styled("Current Symbols: [", Style::default().fg(Color::Cyan)),
            Span::raw(symbols_str),
            Span::styled("]", Style::default().fg(Color::Cyan)),
        ]));

        let paragraph = section("Machine State", text);

        f.render_widget(paragraph, area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let tape_count = self.machine.tapes().len();

        let help_text = vec![
            Line::from("Controls:"),
            Line::from("  Space - Step forward"),
            Line::from("  r - Reset machine"),
            Line::from("  p - Toggle auto-play"),
            Line::from(if self.program_loaded_from_source {
                "  ← → - Program switching disabled (loaded from file/stdin)"
            } else {
                "  ← → - Switch programs"
            }),

            Line::from("  h - Toggle this help"),
            Line::from("  q - Quit"),
            Line::from(""),
            Line::from(format!("Current Mode: {}-Tape", tape_count)),
            Line::from("  All tapes displayed with numbered labels"),
            Line::from("  Each tape shows its head position with bracket notation [symbol] and yellow background"),
            Line::from("  Current symbols shown in array format"),
            Line::from("  Use '_' as a special symbol to match/write the program's blank symbol"),
        ];

        let paragraph = section("Help", help_text);

        f.render_widget(paragraph, area);
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let repo = "@rezigned/tur";
        let outer = block("Status");
        let inner = outer.inner(area);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(repo.len() as u16)])
            .split(inner);

        let auto_play_status = if self.auto_play { "ON" } else { "OFF" };
        let status = Line::from(vec![
            Span::raw("Auto-play: "),
            Span::styled(auto_play_status, Style::default().fg(Color::Yellow)),
            Span::raw(format!(" | {}", self.message)),
        ]);

        let social = Text::from(
            Line::from(Span::styled(repo, Style::default().fg(Color::Yellow))).right_aligned(),
        );

        f.render_widget(outer, area);
        f.render_widget(status, chunks[0]);
        f.render_widget(social, chunks[1]);
    }

    pub fn step_machine(&mut self) {
        match self.machine.step() {
            Step::Continue => {
                self.message = format!("Step {} completed", self.machine.step_count());
            }
            Step::Halt(_) => {
                self.message = "Machine is halted. Press 'r' to reset.".to_string();
                self.auto_play = false;
            }
        }
    }

    pub fn reset_machine(&mut self) {
        self.machine.reset();
        self.message = "Machine reset".to_string();
        self.auto_play = false;
        self.scroll_offset = 0;
    }

    pub fn toggle_auto_play(&mut self) {
        self.auto_play = !self.auto_play;
        self.message = format!(
            "Auto-play {}",
            if self.auto_play {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    pub fn is_auto_playing(&self) -> bool {
        self.auto_play && !self.machine.is_halted()
    }

    pub fn next_program(&mut self) {
        if self.program_loaded_from_source {
            self.message = "Cannot switch programs when loaded from file/stdin.".to_string();
            return;
        }
        let count = ProgramManager::count();
        self.current_program_index = (self.current_program_index + 1) % count;
        self.load_current_program();
    }

    pub fn previous_program(&mut self) {
        if self.program_loaded_from_source {
            self.message = "Cannot switch programs when loaded from file/stdin.".to_string();
            return;
        }
        let count = ProgramManager::count();
        self.current_program_index = if self.current_program_index == 0 {
            count - 1
        } else {
            self.current_program_index - 1
        };
        self.load_current_program();
    }

    fn load_current_program(&mut self) {
        let program = ProgramManager::get_program_by_index(self.current_program_index).unwrap();
        self.program_content =
            ProgramManager::get_program_text_by_index(self.current_program_index)
                .unwrap()
                .to_string();
        let tape_count = program.tapes.len();
        let program_name = program.name.clone();
        self.machine = TuringMachine::new(program);
        self.auto_play = false;
        self.scroll_offset = 0;

        self.message = format!("Loaded {}-tape program: {}", tape_count, program_name);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

fn section<'a>(title: &'a str, content: Vec<Line<'a>>) -> Paragraph<'a> {
    Paragraph::new(content).block(block(title))
}

fn block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .padding(BLOCK_PADDING)
}
