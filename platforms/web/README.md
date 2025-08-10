# Turing Machine Web Platform

This is the web frontend for the Turing Machine simulator, built with Yew (Rust WebAssembly) and CodeMirror for syntax highlighting.

## Architecture Overview

The web platform uses a hybrid architecture combining:
- **Yew** for the main UI components and state management
- **CodeMirror** for the program editor with syntax highlighting
- **JavaScript bridge** to connect CodeMirror with Yew's event system
- **Cytoscape.js** for interactive state transition graph visualization

## Program Editor Flow Diagrams

### User Typing Flow

When a user types in the program editor, the following flow occurs:

```
┌─────────────────┐
│ User types 'a'  │
└─────────┬───────┘
          │
          ▼
┌─────────────────────────────┐
│ CodeMirror "change" event   │
│ fires                       │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ JavaScript change handler:  │
│ - Check programmaticUpdate  │
│ - Update textarea.value     │
│ - Dispatch "input" event    │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ Yew "oninput" handler       │
│ receives InputEvent         │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ Yew sends UpdateText        │
│ message to component        │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ ProgramEditor::update()     │
│ - Updates program_text      │
│ - Calls on_text_change      │
│ - Starts validation timer   │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ App receives text change    │
│ - Updates editor_text       │
│ - Triggers re-render        │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ Validation timer fires      │
│ - Parses program            │
│ - Updates tapes/graph       │
│ - Shows errors              │
└─────────────────────────────┘
```

### Program Switch Flow

When a user selects a different program from the dropdown:

```
┌─────────────────────────────┐
│ User selects program from   │
│ dropdown                    │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ App::SelectProgram          │
│ - Updates current_program   │
│ - Updates editor_text       │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ ProgramEditor::changed()    │
│ detects program change      │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ Calls updateCodeMirrorValue │
│ via js_sys::eval            │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ JavaScript:                 │
│ - Set programmaticUpdate=true│
│ - Call editor.setValue()    │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ CodeMirror "change" event   │
│ fires                       │
└─────────┬───────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ JavaScript change handler:  │
│ - See programmaticUpdate=true│
│ - Set programmaticUpdate=false│
│ - Return early (no input)   │
└─────────────────────────────┘
```

## Key Components

### JavaScript Bridge (`turing-editor.js`)

The JavaScript bridge handles:
- CodeMirror initialization and configuration
- Custom Turing machine syntax highlighting mode with comprehensive token recognition
- Event translation between CodeMirror and Yew
- Preventing infinite update loops with `programmaticUpdate` flag
- Graph theme initialization for Cytoscape.js visualization

### Core Rust Components

**Main Application (`src/main.rs`)**:
- Initializes the ProgramManager with embedded programs
- Sets up error handling and logging
- Renders the main App component

**App Component (`src/app.rs`)**:
- Main application state management
- Handles Turing machine execution and control
- Manages program selection and editor state
- Coordinates between all UI components

**Program Editor (`src/components/program_editor.rs`)**:
- Manages program text state with debounced validation
- Integrates with CodeMirror for syntax highlighting
- Handles program parsing and error reporting
- Only updates CodeMirror during program switches (not user typing)

**Tape View (`src/components/tape_view.rs`)**:
- Advanced multi-tape visualization with smooth animations and integrated controls
- Sliding tape animation with fixed head pointer for intuitive visual feedback
- Stable visual buffer architecture using 4x padding cells (80 cells total) for seamless infinite tape experience
- Improved positioning algorithm with enhanced variable naming and logic clarity for better maintainability
- Centered mapping algorithm that maps logical tape content onto visual buffer with head at stable visual coordinate
- Hardware-accelerated animations with mathematical precision for exact head positioning
- Eliminates visual discontinuities during tape expansion with stable buffer architecture
- Integrated execution controls (Step, Reset, Auto/Pause) embedded in tape header
- Built-in speed control with 5 preset speeds (0.25x to 4x) for execution control
- State information display showing current state, steps, symbols, and execution status
- Multi-tape support with individual head tracking and labeling
- Optimized rendering using logical positions for efficient React reconciliation
- Clean code architecture with improved variable naming and documentation for easier maintenance

### Key Design Decisions

1. **CodeMirror as Source of Truth for User Input**: When users type, CodeMirror maintains cursor position and handles all editing operations.

2. **Yew as Source of Truth for Program Switching**: When switching between predefined programs, Yew updates CodeMirror content.

3. **Separation of Concerns**: User typing and program switching are handled differently to prevent cursor jumping issues.

4. **Safe String Escaping**: Uses `serde_json::to_string` to safely pass program text from Rust to JavaScript, handling special characters like quotes, newlines, and backticks.

## Development

```bash
# Install dependencies
cargo build

# Run development server
trunk serve --port 8080

# Build for production
trunk build --release
```

## Troubleshooting

### Cursor Jumping Issues
If you experience cursor jumping while typing, check:
1. The `programmaticUpdate` flag is working correctly
2. `ProgramEditor::changed()` only updates CodeMirror on program switches
3. CodeMirror initialization is properly synced with textarea content

### Validation Not Working
If real-time validation stops working:
1. Check that the `change` event handler is dispatching input events
2. Verify the validation timer is being set up correctly
3. Ensure the parsing logic in `ValidateProgram` is functioning