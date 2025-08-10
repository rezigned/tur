use keymap::KeyMap;

#[derive(KeyMap, Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// Quit the application
    #[key("q")]
    Quit,
    /// Reset the machine to its initial state
    #[key("r")]
    Reset,
    /// Advance the machine by one step
    #[key("space")]
    Step,
    /// Toggle auto-play
    #[key("p")]
    ToggleAutoPlay,
    /// Toggle help display
    #[key("h")]
    ToggleHelp,
    /// Load the previous program
    #[key("left")]
    PreviousProgram,
    /// Load the next program
    #[key("right")]
    NextProgram,
    /// Scroll tape view up
    #[key("up")]
    ScrollUp,
    /// Scroll tape view down
    #[key("down")]
    ScrollDown,
}
