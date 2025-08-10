mod app;
mod components;
mod url_sharing;

use app::App;
use web_sys::wasm_bindgen::JsCast;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    // Initialize the ProgramManager with embedded programs
    if let Err(e) = tur::ProgramManager::load() {
        web_sys::console::error_1(&format!("Failed to initialize ProgramManager: {}", e).into());
    }

    // Get the specific element by ID
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let root = document
        .get_element_by_id("app")
        .unwrap()
        .dyn_into::<web_sys::Element>()
        .unwrap();

    // Render the app onto that specific element
    yew::Renderer::<App>::with_root(root).render();
}
