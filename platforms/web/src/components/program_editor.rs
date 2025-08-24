use crate::components::ProgramSelector;
use tur::{parser::parse, Program, TuringMachineError, MAX_PROGRAM_SIZE};
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgramEditorProps {
    pub program_text: String,
    pub is_ready: bool,
    pub on_program_submit: Callback<Program>,
    pub on_error: Callback<String>,
    pub on_text_change: Callback<String>,
    pub current_program: usize,
    pub on_select_program: Callback<usize>,
    pub program_name: String,
    pub show_help_modal_from_parent: bool,
    pub on_close_help_modal: Callback<()>,
}

pub enum ProgramEditorMsg {
    UpdateText(String),
    ValidateProgram,
    OpenHelpModal,
    CloseHelpModal,
}

pub struct ProgramEditor {
    program_text: String,
    parse_error: Option<String>,
    is_valid: bool,
    validation_timeout: Option<gloo_timers::callback::Timeout>,
    show_help_modal: bool,
}

impl Component for ProgramEditor {
    type Message = ProgramEditorMsg;
    type Properties = ProgramEditorProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            program_text: ctx.props().program_text.clone(),
            parse_error: None,
            is_valid: ctx.props().is_ready,
            validation_timeout: None,
            show_help_modal: false,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let mut should_render = false;

        if old_props.current_program != ctx.props().current_program {
            self.program_text = ctx.props().program_text.clone();
            self.is_valid = ctx.props().is_ready;
            self.parse_error = None;

            if let Some(timeout) = self.validation_timeout.take() {
                timeout.cancel();
            }

            let escaped_value = serde_json::to_string(&self.program_text).unwrap();
            js_sys::eval(&format!("window.updateCodeMirrorValue({})", escaped_value)).unwrap();

            should_render = true;
        }

        if old_props.show_help_modal_from_parent != ctx.props().show_help_modal_from_parent {
            self.show_help_modal = ctx.props().show_help_modal_from_parent;
            should_render = true;
        }

        should_render
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ProgramEditorMsg::UpdateText(text) => {
                if text.len() > MAX_PROGRAM_SIZE {
                    self.parse_error = Some(format!(
                        "Program too large: {} characters (max: {})

Please reduce the program size.",
                        text.len(),
                        MAX_PROGRAM_SIZE
                    ));
                    self.is_valid = false;
                    return true;
                }

                self.program_text = text;

                // Cancel previous validation timeout
                if let Some(timeout) = self.validation_timeout.take() {
                    timeout.cancel();
                }

                // Set up debounced validation and parent notification
                let delay = if self.program_text.len() > 1000 {
                    500
                } else {
                    300
                };
                let link = ctx.link().clone();
                let text_for_callback = self.program_text.clone();
                let on_text_change_callback = ctx.props().on_text_change.clone();
                self.validation_timeout =
                    Some(gloo_timers::callback::Timeout::new(delay, move || {
                        on_text_change_callback.emit(text_for_callback);
                        link.send_message(ProgramEditorMsg::ValidateProgram);
                    }));

                true
            }
            ProgramEditorMsg::ValidateProgram => {
                self.validation_timeout = None;

                if self.program_text.trim().is_empty() {
                    self.parse_error = None;
                    self.is_valid = false;
                    ctx.props().on_error.emit("Program is empty.".to_string());
                } else {
                    match parse(&self.program_text) {
                        Ok(program) => {
                            self.parse_error = None;
                            self.is_valid = true;
                            ctx.props().on_program_submit.emit(program);
                        }
                        Err(e) => {
                            let error_msg = format_parse_error(&e);
                            self.parse_error = Some(error_msg.clone());
                            self.is_valid = false;
                            ctx.props().on_error.emit(error_msg);
                        }
                    }
                }
                true
            }
            ProgramEditorMsg::OpenHelpModal => {
                self.show_help_modal = true;
                true
            }
            ProgramEditorMsg::CloseHelpModal => {
                self.show_help_modal = false;
                ctx.props().on_close_help_modal.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let on_input = {
            let link = link.clone();
            Callback::from(move |e: InputEvent| {
                let target = e.target_dyn_into::<HtmlTextAreaElement>().unwrap();
                link.send_message(ProgramEditorMsg::UpdateText(target.value()));
            })
        };

        let on_keydown = {
            let _link = link.clone();
            Callback::from(move |_e: KeyboardEvent| {
                // No longer needed as program updates live
            })
        };

        let on_modal_keydown = {
            let link = link.clone();
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Escape" {
                    link.send_message(ProgramEditorMsg::CloseHelpModal);
                }
            })
        };

        html! {
            <div class="program-editor">
                <ProgramSelector
                    key={ctx.props().current_program.to_string()}
                    current_program={ctx.props().current_program}
                    on_select={ctx.props().on_select_program.clone()}
                />
                <div class="editor-content" style="position: relative;">
                    <textarea
                        id="turing-program-editor"
                        class={classes!("textarea", "textarea-bordered", "program-textarea", if self.parse_error.is_some() { Some("error") } else { None })}
                        placeholder="Paste or type your Turing machine program here...

Single-Tape Example:
name: Simple Program
tape: abc
rules:
  start:
    a -> b, R, next
    b -> c, R, halt
  next:
    b -> a, L, start
  halt:

Multi-Tape Example:
name: Multi-Tape Program
heads: [0, 0]
tapes:
  [a, b]
  [c, d]
rules:
  start:
    [a, c] -> [b, d], [R, L], halt
  halt:"
                        value={self.program_text.clone()}
                        oninput={on_input}
                        onkeydown={on_keydown}
                        rows="20"
                        spellcheck="false"
                        aria-describedby="editor-help"
                    />

                    {if let Some(error) = &self.parse_error {
                        let (line_info, error_body) = extract_line_info(error);
                        html! {
                            <div class="program-status error">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                                </svg>
                                <span>
                                    <strong>{"Parse Error"}</strong>
                                    {if !line_info.is_empty() {
                                        html! { <span class="opacity-70">{format!(" ({})", line_info)}</span> }
                                    } else {
                                        html! {}
                                    }}
                                    <pre class="text-xs mt-1 opacity-90">{error_body}</pre>
                                </span>
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    <div class="editor-help" id="editor-help" style="position: absolute; bottom: 16px; right: 16px;">
                        <button
                            class="btn btn-circle btn-sm"
                            onclick={link.callback(|_| ProgramEditorMsg::OpenHelpModal)}
                            title="Help"
                            style={format!("color: {};",
                                if self.parse_error.is_some() { "red" } else { "white" }
                            )}
                        >
                            {"?"}
                        </button>

                        <dialog
                            id="help_modal"
                            class={classes!("modal", if self.show_help_modal { Some("modal-open") } else { None })}
                            onclick={link.callback(|_| ProgramEditorMsg::CloseHelpModal)}
                            onkeydown={on_modal_keydown}
                        >
                            <div class="modal-box w-11/12 max-w-5xl relative" onclick={link.callback(|e: MouseEvent| { e.stop_propagation(); ProgramEditorMsg::ValidateProgram })}>
                                <button
                                    class="btn btn-sm btn-circle btn-ghost"
                                    onclick={link.callback(|_| ProgramEditorMsg::CloseHelpModal)}
                                    style="position: absolute; top: 8px; right: 8px; display: flex; align-items: center; justify-content: center; font-size: 18px; line-height: 1; z-index: 10;"
                                >
                                    {"×"}
                                </button>
                                <h3 class="font-bold text-lg">{"Program Format Help"}</h3>
                                <div class="help-content">
                                    <h4 class="help-heading">{"Single-Tape Format:"}</h4>
                                    <p class="help-paragraph">{"Your program should include:"}</p>
                                    <ul class="help-list">
                                        <li class="help-list-item"><code>{"name: Your Program Name"}</code></li>
                                        <li class="help-list-item"><code>{"tape: initial_tape_content"}</code></li>
                                        <li class="help-list-item"><code>{"rules:"}</code> {" section with states and rules"}</li>
                                    </ul>

                                    <h4 class="help-heading">{"Multi-Tape Format:"}</h4>
                                    <p class="help-paragraph">{"For multi-tape programs, use:"}</p>
                                    <ul class="help-list">
                                        <li class="help-list-item"><code>{"heads: [0, 0, ...]"}</code> {" - initial head positions"}</li>
                                        <li class="help-list-item"><code>{"tapes:"}</code> {" section with initial tape contents"}</li>
                                        <li class="help-list-item">{"Use array notation: "} <code>{"[a, b] -> [c, d], [R, L], next_state"}</code></li>
                                    </ul>

                                    <h4 class="help-heading">{"Transition Rules:"}</h4>
                                    <p class="help-paragraph">{"Format: "} <code>{"current_symbol -> new_symbol, direction, next_state"}</code></p>
                                    <ul class="help-list">
                                        <li class="help-list-item">{"Directions: "} <code>{"L"}</code> {" (left), "} <code>{"R"}</code> {" (right), "} <code>{"S"}</code> {" (stay)"}</li>
                                        <li class="help-list-item">{"Use "} <code>{"_"}</code> {" as a special symbol to match/write the program's blank symbol (e.g., if blank is ' ', then '_' matches ' ')"}</li>
                                    </ul>
                                </div>
                            </div>
                        </dialog>
                    </div>
                </div>
            </div>
        }
    }
}

fn extract_line_info(error: &str) -> (String, String) {
    // Try to extract line and column information from Pest errors
    if let Some(line_start) = error.find(" --> ") {
        if let Some(line_end) = error[line_start + 5..].find('\n') {
            let line_info = &error[line_start + 5..line_start + 5 + line_end];
            if let Some(colon_pos) = line_info.find(':') {
                let line_num = &line_info[..colon_pos];
                if line_num.parse::<u32>().is_ok() {
                    return (format!("line {}", line_num), error.to_string());
                }
            }
        }
    }

    // If no line info found, return empty line info
    ("".to_string(), error.to_string())
}

fn format_parse_error(error: &TuringMachineError) -> String {
    match error {
        TuringMachineError::ParseError(msg) => format!("Parse Error: {msg}"),
        _ => format!("Error: {error}"),
    }
}
