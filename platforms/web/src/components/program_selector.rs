use tur::ProgramManager;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgramSelectorProps {
    pub current_program: usize,
    pub on_select: Callback<usize>,
}

#[function_component(ProgramSelector)]
pub fn program_selector(props: &ProgramSelectorProps) -> Html {
    let on_change = {
        let on_select = props.on_select.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let value = target.value();
            if value.is_empty() {
                on_select.emit(usize::MAX);
            } else {
                let index = value.parse::<usize>().unwrap_or(0);
                on_select.emit(index);
            }
        })
    };

    let is_custom = props.current_program == usize::MAX;

    html! {
        <select
            class="program-selector-dropdown select select-bordered"
            onchange={on_change}
            value={if is_custom { "".to_string() } else { props.current_program.to_string() }}
        >
            {if is_custom {
                html! {
                    <option value="" selected=true>{"Custom Program Active"}</option>
                }
            } else {
                html! {}
            }}
            {(0..ProgramManager::count()).map(|i| {
                let program = ProgramManager::get_program_by_index(i).unwrap();
                let tape_indicator = if program.is_single_tape() { "📼" } else { "📼📼" };
                html! {
                    <option key={i} value={i.to_string()} selected={i == props.current_program}>
                        {format!("{} {} - {}", i + 1, tape_indicator, program.name)}
                    </option>
                }
            }).collect::<Html>()}
        </select>
    }
}
