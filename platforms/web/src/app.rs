use crate::components::{GraphView, MachineState, ProgramEditor, ShareButton, TapeView};
use crate::url_sharing::UrlSharing;
use action::Action;
use gloo_events::EventListener;

use keymap::{Config, KeyMapConfig};
use tur::types::Halt;
use tur::{Direction, Program, ProgramManager, Step, Transition, TuringMachine};
use wasm_bindgen::JsCast;
use yew::prelude::*;

pub enum Msg {
    PerformAction(Action),
    Step,
    Reset,
    ToggleAutoPlay,
    SelectProgram(usize),
    AutoStep,
    LoadCustomProgram(Program),
    EditorError(String),
    UpdateEditorText(String),
    SetSpeed(u64),
    HideProgramEditorHelp,
}

pub struct App {
    machine: TuringMachine,
    current_program: usize,
    auto_play: bool,
    message: String,
    editor_text: String,
    is_program_ready: bool,
    current_program_def: Program,
    last_transition: Option<Transition>,
    previous_state: String,
    speed: u64,
    machine_state: MachineState,

    _keyboard_listener: EventListener,
    keymap: Config<Action>,
    show_program_editor_help: bool,
    tape_left_offsets: Vec<usize>,
}

impl App {
    fn new(
        program: Program,
        program_text: String,
        program_index: usize,
        message: String,
        keyboard_listener: EventListener,
    ) -> Self {
        let initial_state = program.initial_state.clone();
        let current_program_def = program.clone();
        let machine = TuringMachine::new(program);
        let num_tapes = machine.tapes().len();

        Self {
            machine,
            current_program: program_index,
            auto_play: false,
            message,
            editor_text: program_text,
            is_program_ready: true,
            current_program_def,
            last_transition: None,
            previous_state: initial_state,
            speed: 500,
            machine_state: MachineState::Running,

            _keyboard_listener: keyboard_listener,
            keymap: Action::keymap_config(),
            show_program_editor_help: false,
            tape_left_offsets: vec![0; num_tapes],
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let keymap_config = Action::keymap_config();
        let link = ctx.link().clone();
        let keyboard_listener = EventListener::new(
            &web_sys::window().unwrap().document().unwrap(),
            "keydown",
            move |event| {
                let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                let active_element = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .active_element();

                if let Some(element) = active_element {
                    if element.tag_name().to_lowercase() == "textarea"
                        || element.tag_name().to_lowercase() == "input"
                    {
                        return;
                    }
                }

                if let Some(action) = keymap_config.get(event) {
                    link.send_message(Msg::PerformAction(*action));
                }
            },
        );

        // Check if there's a shared program in the URL
        if let Some(shared_program) = UrlSharing::extract_from_url() {
            if let Ok(program) = tur::parser::parse(&shared_program.code) {
                return App::new(
                    program,
                    shared_program.code,
                    usize::MAX, // Custom program index
                    format!("Loaded shared program: {}", shared_program.name),
                    keyboard_listener,
                );
            } else if let Err(e) = tur::parser::parse(&shared_program.code) {
                web_sys::console::warn_1(&format!("Failed to parse shared program: {}", e).into());
            }
        }

        // Default initialization
        let program = ProgramManager::get_program_by_index(0).unwrap();
        let editor_text = ProgramManager::get_program_text_by_index(0)
            .unwrap_or("")
            .to_string();

        App::new(program, editor_text, 0, "".to_string(), keyboard_listener)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PerformAction(action) => {
                match action {
                    Action::Step => {
                        ctx.link().send_message(Msg::Step);
                    }
                    Action::Reset => {
                        ctx.link().send_message(Msg::Reset);
                    }
                    Action::ToggleAutoPlay => {
                        ctx.link().send_message(Msg::ToggleAutoPlay);
                    }
                    Action::ToggleHelp => {
                        self.show_program_editor_help = !self.show_program_editor_help;
                    }
                    Action::PreviousProgram => {
                        let count = ProgramManager::count();
                        if count > 0 {
                            let new_index = if self.current_program == 0
                                || self.current_program == usize::MAX
                            {
                                count - 1
                            } else {
                                self.current_program - 1
                            };
                            ctx.link().send_message(Msg::SelectProgram(new_index));
                        }
                    }
                    Action::NextProgram => {
                        let count = ProgramManager::count();
                        if count > 0 {
                            let new_index = if self.current_program == usize::MAX {
                                0
                            } else {
                                (self.current_program + 1) % count
                            };
                            ctx.link().send_message(Msg::SelectProgram(new_index));
                        }
                    }

                    Action::Quit => {
                        web_sys::console::log_1(
                            &"Quit action received. (No direct equivalent in web app)".into(),
                        );
                    }
                }

                true
            }
            Msg::Step => {
                if self.machine.is_halted() {
                    self.message = "Machine is halted. Press 'r' to reset.".to_string();
                    self.auto_play = false;
                    self.machine_state = MachineState::Halted;
                    return true;
                }

                self.previous_state = self.machine.state().to_string();
                self.last_transition = self.machine.transition().cloned();
                let last_head_positions = self.machine.heads().to_vec();
                let last_tape_lengths: Vec<usize> =
                    self.machine.tapes().iter().map(|t| t.len()).collect();

                let result = self.machine.step();

                match result {
                    Step::Continue => {
                        self.message = format!("Step {} completed", self.machine.step_count());
                        self.machine_state = MachineState::Running;
                    }
                    Step::Halt(reason) => {
                        self.message = match reason {
                            Halt::Ok => "Machine halted".to_string(),
                            Halt::Err(err) => format!("Machine halted with error: {err}"),
                        };
                        self.machine_state = MachineState::Halted;
                        self.auto_play = false;
                    }
                }

                // Update tape's left offset for animation purposes
                let new_tape_lengths: Vec<usize> =
                    self.machine.tapes().iter().map(|t| t.len()).collect();

                if let Some(transition) = &self.last_transition {
                    for i in 0..last_head_positions.len() {
                        // Check if tape expanded to the left (length increased)
                        if transition.directions[i] == Direction::Left
                            && new_tape_lengths[i] > last_tape_lengths[i]
                        {
                            self.tape_left_offsets[i] += 1;
                        }
                    }
                }
                true
            }
            Msg::Reset => {
                self.machine.reset();
                self.message = "Machine reset".to_string();
                self.auto_play = false;
                self.last_transition = None;
                self.tape_left_offsets = vec![0; self.machine.tapes().len()];
                self.previous_state = self.machine.initial_state().to_string();
                self.machine_state = MachineState::Running;
                true
            }
            Msg::ToggleAutoPlay => {
                self.auto_play = !self.auto_play;
                self.message = format!(
                    "Auto-play {}",
                    if self.auto_play {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );

                if self.auto_play {
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(500).await;
                        link.send_message(Msg::AutoStep);
                    });
                }
                true
            }
            Msg::SelectProgram(index) => {
                if index != self.current_program {
                    self.current_program = index;
                    let program = ProgramManager::get_program_by_index(index).unwrap();
                    self.editor_text = ProgramManager::get_program_text_by_index(index)
                        .unwrap_or("")
                        .into();

                    self.auto_play = false;
                    self.is_program_ready = true;
                    self.current_program_def = program.clone();
                    self.last_transition = None;
                    self.previous_state = program.initial_state.clone();
                    self.machine = TuringMachine::new(program);
                    self.tape_left_offsets = vec![0; self.machine.tapes().len()];
                    self.message = "".to_string();
                    self.machine_state = MachineState::Running;
                    true
                } else {
                    false
                }
            }
            Msg::AutoStep => {
                if self.auto_play && self.machine_state == MachineState::Running {
                    ctx.link().send_message(Msg::Step);

                    let link = ctx.link().clone();
                    let speed = self.speed;
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(speed.try_into().unwrap()).await;
                        link.send_message(Msg::AutoStep);
                    });
                }
                true
            }
            Msg::LoadCustomProgram(program) => {
                self.auto_play = false;
                self.is_program_ready = true;
                self.current_program = usize::MAX; // Indicate custom program
                self.current_program_def = program.clone();
                self.last_transition = None;
                self.previous_state = program.initial_state.clone();
                self.machine = TuringMachine::new(program);
                self.tape_left_offsets = vec![0; self.machine.tapes().len()];
                self.message = "".to_string();
                self.machine_state = MachineState::Running;
                true
            }
            Msg::EditorError(error) => {
                self.message = format!("Program error: {}", error);
                self.is_program_ready = false;
                true
            }
            Msg::UpdateEditorText(text) => {
                if self.current_program != usize::MAX {
                    if let Ok(canonical_text) =
                        ProgramManager::get_program_text_by_index(self.current_program)
                    {
                        if text != canonical_text {
                            self.current_program = usize::MAX;
                        }
                    } else {
                        self.current_program = usize::MAX;
                    }
                }
                self.editor_text = text;
                true
            }
            Msg::SetSpeed(speed) => {
                self.speed = speed;
                true
            }

            Msg::HideProgramEditorHelp => {
                self.show_program_editor_help = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let help_text_items = self.keymap.items.iter().filter(|(_, item)| !item.description.contains("Quit")).map(|(_, item)| {
            html! { <li><kbd class="kbd">{ item.keys.join(", ") }</kbd>{ format!(": {}", item.description) }</li> }
        }).collect::<Html>();

        html! {
                <main class="main-layout">
                    <div class="left-panel">
                        <div class="editor-section card card-compact bg-base-100">
                            <div class="card-body">
                                <div class="editor-header">
                                    <h3 class="card-title">{"Program Editor"}</h3>
                                    <ShareButton
                                        program_name={self.current_program_def.name.clone()}
                                        program_code={self.editor_text.clone()}
                                        is_enabled={self.is_program_ready}
                                    />
                                </div>
                                <ProgramEditor
                                    program_text={self.editor_text.clone()}
                                    is_ready={self.is_program_ready}
                                    on_program_submit={link.callback(Msg::LoadCustomProgram)}
                                    on_error={link.callback(Msg::EditorError)}
                                    on_text_change={link.callback(Msg::UpdateEditorText)}
                                    current_program={self.current_program}
                                    on_select_program={link.callback(Msg::SelectProgram)}
                                    program_name={self.current_program_def.name.clone()}
                                    show_help_modal_from_parent={self.show_program_editor_help}
                                    on_close_help_modal={link.callback(|_| Msg::HideProgramEditorHelp)}
                                />
                            </div>
                        </div>
                                                  <div class="help-section card card-compact bg-base-100">
                            <div class="card-body">
                                <h3 class="card-title">{"Keyboard Shortcuts"}</h3>
                                <ul>{ help_text_items }</ul>
                            </div>
                        </div>
                    </div>

                    <div class="right-panel">
                        <div class="visualization-section">
                            <div class="tape-and-state-section card card-compact bg-base-100">
                                <div class="card-body">
                                    <TapeView
                                        tapes={self.machine.tapes().to_vec()}
                                        head_positions={self.machine.heads().to_vec()}
                                        auto_play={self.auto_play}
                                        machine_state={self.machine_state.clone()}
                                        is_program_ready={self.is_program_ready}
                                        blank_symbol={self.machine.blank()}
                                        state={self.machine.state().to_string()}
                                        step_count={self.machine.step_count()}
                                        current_symbols={self.machine.symbols()}
                                        on_step={link.callback(|_| Msg::Step)}
                                        on_reset={link.callback(|_| Msg::Reset)}
                                        on_toggle_auto={link.callback(|_| Msg::ToggleAutoPlay)}
                                        speed={self.speed}
                                        on_speed_change={link.callback(|speed: u64| Msg::SetSpeed(speed))}
                                        tape_left_offsets={self.tape_left_offsets.clone()}
                                        message={self.message.clone()}
                                    />
                                </div>
                            </div>

                            <div class="graph-section card card-compact bg-base-100">
                                <div class="card-body">
                                    <h3 class="card-title">{"State Graph"}</h3>
                                    <GraphView
                                        program={self.current_program_def.clone()}
                                        current_state={self.machine.state().to_string()}
                                        previous_state={self.previous_state.clone()}
                                        last_transition={self.last_transition.clone()}
                                        step_count={self.machine.step_count()}
                                    />
                                </div>
                            </div>
                        </div>

                    </div>
                </main>
        }
    }
}
