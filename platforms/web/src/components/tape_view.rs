use yew::{function_component, html, Callback, Event, Html, Properties, TargetCast};

#[derive(Properties, PartialEq)]
pub struct TapeViewProps {
    pub tapes: Vec<Vec<char>>,
    pub head_positions: Vec<usize>,
    pub auto_play: bool,
    pub is_halted: bool,
    pub is_program_ready: bool,
    pub blank_symbol: char,
    pub state: String,
    pub step_count: usize,
    pub current_symbols: Vec<char>,
    pub on_step: Callback<()>,
    pub on_reset: Callback<()>,
    pub on_toggle_auto: Callback<()>,
    pub speed: u64,
    pub on_speed_change: Callback<u64>,
    pub tape_left_offsets: Vec<usize>,
}

#[function_component(TapeView)]
pub fn tape_view(props: &TapeViewProps) -> Html {
    let cell_width = 42; // Width of each cell (no gap)
    let padding_cells = 15; // Number of blank cells to show on each side for infinite tape effect

    let on_speed_change = props.on_speed_change.clone();

    html! {
        <div class="tape-view">
            <div class="tape-header">
                <h3 class="card-title">{"Tapes"}</h3>
                <div class="tape-controls">
                    <button
                        class="btn btn-primary"
                        onclick={props.on_step.reform(|_| ())}
                        disabled={props.is_halted || !props.is_program_ready}
                    >
                        {"Step"}
                    </button>
                    <button
                        class="btn btn-secondary"
                        onclick={props.on_reset.reform(|_| ())}
                        disabled={!props.is_program_ready}
                    >
                        {"Reset"}
                    </button>
                    <button
                        class={format!("btn {}", if props.auto_play { "btn-warning" } else { "btn-success" })}
                        onclick={props.on_toggle_auto.reform(|_| ())}
                        disabled={props.is_halted || !props.is_program_ready}
                    >
                        {if props.auto_play { "Pause" } else { "Auto" }}
                    </button>
                    <div class="speed-control">
                        <label for="speed-select">{"Speed: "}</label>
                        <select id="speed-select" class="custom-select select select-bordered" onchange={Callback::from(move |e: Event| {
                            let speed_str = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                            on_speed_change.emit(speed_str.parse().unwrap_or(500));
                        })}>
                            <option value="2000" selected={props.speed == 2000}>{"0.25x"}</option>
                            <option value="1000" selected={props.speed == 1000}>{"0.5x"}</option>
                            <option value="500" selected={props.speed == 500}>{"1x"}</option>
                            <option value="250" selected={props.speed == 250}>{"2x"}</option>
                            <option value="125" selected={props.speed == 125}>{"4x"}</option>
                        </select>
                    </div>
                </div>
            </div>
            <div class="tapes-container">
                {props.tapes.iter().enumerate().map(|(tape_index, tape)| {
                    let head_position = props.head_positions.get(tape_index).cloned().unwrap_or(0);
                    let left_offset = *props.tape_left_offsets.get(tape_index).unwrap_or(&0);

                    // Add padding cells on the left
                    let mut visible_tape = vec![props.blank_symbol; padding_cells - left_offset];

                    // Add the actual tape content
                    for &symbol in tape {
                        visible_tape.push(symbol);
                    }

                    // Add padding cells on the right
                    visible_tape.extend(std::iter::repeat_n(props.blank_symbol, padding_cells));

                    // Calculate the transform to center the active cell under the head pointer
                    let active_cell_index = padding_cells + head_position - left_offset;
                    let cell_offset = active_cell_index as i32 * cell_width;

                    // js_sys::eval(&format!("console.log({active_cell_index}, {cell_offset}, {left_offset})")).unwrap();
                    let transform_style = format!(
                        "transform: translateX(calc(50% - {cell_offset}px - {}px)) translateY(-50%)",
                        cell_width / 2
                    );

                    html! {
                        <div key={tape_index} class="single-tape">
                            {html!{
                                <div class="tape-label">
                                    {format!("#{} ", tape_index + 1)}
                                    <span class="head-info-inline">
                                        {format!("(head: {head_position})")}
                                    </span>
                                </div>
                            }}
                            <div class="tape-machine">
                                <div class="tape-container" style={transform_style}>
                                    {visible_tape.iter().enumerate().map(|(i, &symbol)| {
                                        // Check if this cell is under the head
                                        let is_under_head = i == active_cell_index;

                                        let class = if is_under_head {
                                            "tape-cell under-head"
                                        } else {
                                            "tape-cell"
                                        };

                                        html! {
                                            <div key={format!("{tape_index}_{i}_{symbol}_{i}")} class={class}>
                                                {symbol}
                                            </div>
                                        }
                                    }).collect::<Html>()}
                                </div>
                                <div class="tape-head">
                                    <div class="head-pointer"></div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Html>()}
            </div>

            <div class="state-info">
                <div class="state-item">
                    <span class="label">{"State"}</span>
                    <span class="value state-name">{&props.state}</span>
                </div>
                <div class="state-item">
                    <span class="label">{"Steps"}</span>
                    <span class="value">{props.step_count}</span>
                </div>
                <div class="state-item">
                    <span class="label">{"Symbols"}</span>
                    <span class="value">
                        {props.current_symbols.iter().map(|&symbol| {
                            html! { <span class="symbol">{symbol}</span> }
                        }).collect::<Html>()}
                    </span>
                </div>
                <div class="state-item">
                    <span class="label">{"Status"}</span>
                    <span class={
                        if props.is_halted {
                            "value status halted"
                        } else if props.auto_play {
                            "value status running"
                        } else {
                            "value status ready"
                        }
                    }>
                        {
                            if props.is_halted {
                                "HALTED"
                            } else if props.auto_play {
                                "RUNNING"
                            } else {
                                "READY"
                            }
                        }
                    </span>
                </div>
            </div>
        </div>
    }
}
