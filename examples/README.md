# Turing Machine Programs

This directory contains example Turing machine programs in `.tur` format. These programs can be loaded and executed by the Turing machine simulator.

## File Format

Turing machine programs are defined in a simple text format with the `.tur` extension. The format consists of three main sections:

1. **Name**: The name of the program
2. **Tape**: The initial tape content
3. **Transitions**: The state transition rules

### Example

```
name: Simple Test
tape: a
transitions:
  start:
    a -> b, R, halt
  halt:
```

### Syntax

The `.tur` file format uses a structured syntax parsed by a Pest grammar:

- **Name**: Specified with `name:` followed by the program name
- **Tape**: Specified with `tape:` followed by comma-separated initial tape symbols (for single-tape machines)
- **Tapes**: Specified with `tapes:` followed by multiple lines of tape definitions for multi-tape machines:
  ```
  tapes:
    - [a, b, c]
    - [x, y, z]
  ```
- **Head**: Specified with `head:` followed by the initial head position (default: 0)
- **Heads**: Specified with `heads:` followed by an array of initial head positions for multi-tape machines:
  ```
  heads: [0, 0]
  ```
- **Blank**: Specified with `blank:` followed by the symbol to use for blank cells (default: '-')
- **Transitions**: Specified with `transitions:` followed by state definitions
  - Each state is defined by its name followed by a colon
  - Each transition rule is indented (2 spaces or tabs) and specifies:
    - For single-tape machines: `symbol -> new_symbol, direction, next_state`
    - For multi-tape machines: `[symbol1, symbol2, ...] -> [new1, new2, ...], [dir1, dir2, ...], next_state`
    - The direction to move: `R`/`>` (right), `L`/`<` (left), `S`/`-` (stay)
    - The next state to transition to
  - If `-> new_symbol` is omitted, the read symbol is preserved

### Advanced Features

- **Comments**: Use `#` for line comments (ignored by parser)
- **Trailing Comments**: Support for comments at the end of transition lines
- **Symbol Quoting**: Use single quotes for special characters: `'#'`, `','`, `' '`
- **Symbol Conversion**: Special characters are automatically converted using the `convert_symbol_for_parser` function:
  - `$` → 'S' (Start marker)
  - `#` → 'H' (Hash/separator)
  - `-` → 'M' (Minus)
  - `+` → 'P' (Plus)
  - `=` → 'E' (Equals)
  - `*` → 'T' (Star)
  - `/` → 'D' (Divide)
  - `.` → 'O' (Dot)
  - `,` → 'C' (Comma)
  - Alphanumeric characters and spaces remain unchanged
  - Other special characters are converted to 'X'
- **Flexible Indentation**: Supports both spaces and tabs for indentation
- **Optional Write**: If `-> symbol` is omitted, the read symbol is preserved
- **Multiple Direction Formats**: Supports `L`/`<` (left), `R`/`>` (right), `S`/`-` (stay)
- **Error Handling**: Parser provides detailed error messages with line numbers and context
- **Program Validation**: Comprehensive validation including state reference checking
- **Multi-Tape Support**: Define and operate on multiple tapes simultaneously
- **Cross-Platform Loading**: Programs embedded in binary and loaded from filesystem
- **Static Analysis**: Programs are validated for correctness before execution:
  - Ensures the "start" state is defined
  - Verifies that halt states are properly referenced
  - Checks that the initial head position is valid
  - Detects transitions to undefined states
  - Identifies unreachable states in the program
  - Validates that all symbols in the initial tape have defined transitions
  - Detects transitions to states that don't exist in the program
  - Identifies states that can never be reached during execution
  - Validates that all symbols used in transitions are valid

### Grammar Details

The parser uses a Pest grammar (`src/src/grammar.pest`) with the following key rules:

- **Program Structure**: `program = { SOI ~ (name)? ~ (head | heads)? ~ blank? ~ (tape | tapes) ~ (transitions) ~ EOI }`
- **Indentation Handling**: Flexible indentation using `INDENT = { ("  " | "\t")+ }`
- **Comment Support**: `comment = { "#" ~ (!NEWLINE ~ ANY)* }` and `trailing_comment = { WHITESPACE+ ~ "#" ~ (!NEWLINE ~ ANY)* }`
- **Symbol Parsing**: Supports quoted and unquoted symbols with reserved character handling
- **Direction Parsing**: Multiple direction formats (`L`/`<`, `R`/`>`, `S`/`-`) with proper Stay direction support
- **Block Structure**: Proper indentation tracking with `block_start = _{ NEWLINE ~ PEEK_ALL ~ PUSH(INDENT) }` and `block_end = _{ DROP }`
- **Action Rules**: Enhanced action parsing with support for both single-tape and multi-tape operations:
  - Single-tape: `single_tape_action = { symbol ~ ("->" ~ symbol)? ~ "," ~ direction ~ "," ~ state }`
  - Multi-tape: `multi_tape_action = { multi_tape_symbols ~ "->" ~ multi_tape_symbols ~ "," ~ directions ~ "," ~ state }`

## Creating Your Own Programs

To create your own Turing machine program:

1. Create a new file with the `.tur` extension
2. Define the program using the format described above
3. Place the file in the `examples` directory
4. The program will be automatically loaded when the application starts (TUI platform) or embedded at build time (Web platform)
5. Use the program manager API to access your custom programs programmatically

When creating a program, remember that all required fields must be specified or default values will be used:
- If `head:` is not specified, the default position is 0
- If `blank:` is not specified, the default blank symbol is '-'

These defaults are consistent across all platforms (TUI, Web, and Vite frontend).

### Platform-Specific Loading

- **TUI Platform**: Programs are loaded from the file system at runtime using `ProgramManager::initialize(Some(path))` with unified display for both single and multi-tape programs featuring enhanced head visualization with bracket notation `[symbol]` and yellow background for improved visibility across all terminal environments
- **Web Platform (Yew)**: Programs are embedded in the binary at build time and initialized automatically at startup using `ProgramManager::initialize(None)` with advanced tape visualization featuring sliding tape animation, infinite tape effect with 20 padding cells on each side for seamless boundary transitions, fixed head design for intuitive visual feedback, integrated execution controls with speed control, and state information display, plus URL sharing capabilities for easy program distribution and triple graph visualization systems:
- **SolidJS Frontend**: Modern TypeScript frontend with reactive state management and comprehensive URL sharing:
  - **Dynamic Program Loading**: Programs loaded via WebAssembly bindings with real-time parsing
  - **URL Sharing Integration**: Complete ShareButton component with base64 encoding and clipboard support
  - **State Machine Visualization**: XState v5 integration for professional state machine representation
  - **Interactive Graph Rendering**: Cytoscape.js integration with smooth animations and transitions
  - **Modern Development Experience**: Vite build system with HMR and TypeScript support
  - **ShareButton Component**: Reactive component with loading states, error handling, and clipboard integration
  - **URL Encoding**: URL-safe base64 encoding with automatic padding and character replacement
  - **Clipboard API**: Modern Clipboard API with fallback support for older browsers
  - **Real-time Feedback**: Success/error messages with automatic timeout and visual indicators
  - **Unicode Support**: Full support for international characters and emojis in program names
- **WASM Bindings**: Programs are available immediately after module initialization without explicit loading

### URL Sharing (Web Platform)

The web platform includes comprehensive URL sharing functionality through the optimized `platforms/web/src/url_sharing.rs` module for easy program distribution:

**Sharing Programs:**
- Create shareable URLs for any custom program written in the editor
- URLs use URL-safe base64 encoding (`URL_SAFE_NO_PAD`) for compact, reliable sharing
- Full Unicode support for international characters and emojis in program names
- Automatic clipboard integration using modern Clipboard API with graceful fallbacks
- Optimized implementation with minimal dependencies and clean error handling

**Loading Shared Programs:**
- Shared programs are automatically detected and loaded from URLs on page load
- Programs load directly into the editor with full syntax highlighting preserved
- Original program name and formatting are maintained through JSON serialization
- Seamless integration with the existing program management system
- Comprehensive error handling and UTF-8 validation for safe data processing

**Technical Features:**
- Custom URL parameter parser with proper decoding support
- History API integration for URL updates without page reload
- Browser compatibility with appropriate fallback mechanisms
- Comprehensive test coverage including Unicode and edge case handling

**URL Format:**
```
https://example.com/turing-machine?share=<base64-encoded-program>
```

The encoded data contains both the program name and complete source code as a JSON structure, allowing for full program reconstruction from the URL alone with type safety and data integrity validation.

## Examples

This directory includes several example programs:

- `binary-addition.tur`: Performs binary addition
- `palindrome.tur`: Checks if a string is a palindrome
- `binary-counter.tur`: Implements a binary counter
- `subtraction.tur`: Performs subtraction
- `busy-beaver-3.tur`: The famous 3-state Busy Beaver program that writes the maximum number of 1s (6) before halting in 14 steps - a classic example from computability theory
- `simple.tur`: Basic example for testing
- `multi-tape-example.tur`: Demonstrates multi-tape operations with proper Stay direction support

Feel free to use these as templates for creating your own programs.

## Package Information

This Turing machine simulator is available on crates.io:

- **Core Library**: [`tur`](https://crates.io/crates/tur) - Parser, interpreter, and execution engine
- **CLI Tool**: [`tur-cli`](https://crates.io/crates/tur-cli) - Command-line interface
- **TUI Tool**: [`tur-tui`](https://crates.io/crates/tur-tui) - Terminal user interface
- **Documentation**: [docs.rs/tur](https://docs.rs/tur)
- **Repository**: [github.com/rezigned/turing-machine](https://github.com/rezigned/turing-machine)

## Recent Improvements

The parser, validation system, and visualization components have been enhanced with several improvements:

- **Program Editor Cleanup and Enhancement**: The web application's program editor has been significantly cleaned up and enhanced with streamlined code structure, improved error handling, and better user experience. The component now features debounced validation with adaptive delays, 64KB program size limits for performance protection, optimized CodeMirror integration with smart update detection that prevents unnecessary refreshes during user typing (only updating when switching between programs), dynamic help interface with a context-aware help button (?) that displays in accent color normally and changes to red when parse errors occur for immediate visual feedback, enhanced modal design with a properly centered close button (×) using the standard multiplication symbol, and cleaner error message formatting. Over 1,200 lines of duplicated code were removed, including 13+ nested container elements and repeated help modal definitions. Debug console logging has been removed for cleaner production code and improved performance. The keyboard event handling has been simplified with live program updates, removing unnecessary Escape key handling from the main editor while maintaining modal functionality.

- **Enhanced Tape Visualization**: Major improvements to the web application's tape display component:
  - **Sliding Tape Animation**: Tape slides smoothly left/right while head pointer remains fixed at center for intuitive visual feedback
  - **Stable Visual Buffer**: Creates a much larger visual tape buffer (4x padding cells) that can accommodate expansion in both directions, preventing animation jumps
  - **Fixed Head Design**: Head pointer stays fixed at the center while tape moves, providing clear visual indication of tape movement
  - **Centered Mapping Algorithm**: Maps logical tape content onto the visual buffer with the head always positioned at the visual center:
    ```rust
    let visual_tape_size = padding_cells * 4; // Much larger buffer
    let visual_center = visual_tape_size / 2;
    let visual_start = visual_center.saturating_sub(head_position);
    ```
  - **Hardware Acceleration**: Uses CSS transforms for smooth 60fps animations with hardware acceleration
  - **Smooth Boundary Transitions**: Eliminates visual discontinuities during tape expansion with stable buffer architecture
  - **Intuitive Animation**: Provides clear visual feedback of tape movement during Turing machine execution
  - **Integrated Controls**: Execution controls embedded directly in the tape header for better UX
  - **Speed Control**: Built-in speed selector with 5 preset speeds (0.25x to 4x) for execution control
  - **State Information Display**: Integrated state display showing current state, steps, symbols, and execution status
  - **Multi-Tape Support**: Full support for multiple tapes with individual head tracking and labeling
  - **Optimized Rendering**: Enhanced key generation using logical positions for efficient React reconciliation and reduced unnecessary re-renders
  - **Function Component Architecture**: Complete rewrite using `#[function_component(TapeView)]` for better performance and maintainability
  - **Mathematical Precision**: Transform calculation ensures the active cell appears exactly under the fixed head pointer
  - **Developer Experience**: Clearer code structure makes the positioning algorithm easier to understand and maintain
- **XState Inspector Integration**: Major upgrade from custom JSON visualization to XState's native inspector integration for professional state machine visualization
- **Enhanced Graph Visualization**: Major improvements to the web application's state transition graph:
  - **Triple Visualization Systems**: Added XState v5-based visualization and HybridGraphView alongside existing Cytoscape.js implementation
  - **Advanced Animation System**: Smooth 60fps animation loop with optimized performance and consistent alpha transparency management
  - **Robust Initialization**: Enhanced graph initialization with automatic retry mechanism (up to 10 attempts), comprehensive dependency checking for Cytoscape.js and theme availability, and graceful error recovery with progressive retry delays for reliable graph loading
  - **Hybrid Graph Integration**: New component combining XState v5 state machine logic with Cytoscape.js rendering for optimal performance with comprehensive debugging support
  - **State Transition Tracking**: Enhanced transition highlighting with previous state tracking for accurate visualization of executed transitions
  - **Optimized Property Comparison**: Custom `PartialEq` implementation for GraphView reduces unnecessary re-renders by comparing only essential fields (current_state, previous_state, last_transition, step_count, and program name)
  - **Intelligent Edge Coloring**: Semantic color coding based on transition characteristics:
    - Blue for left moves, green for right moves, orange for stay operations
    - Purple for self-loops, red for halt transitions
    - Symbol-based coloring for binary operations (blue-grey) and letter operations (brown)
  - **Active Transition Highlighting**: Bright red pulsing effects with filled arrowheads and enhanced label styling during execution
  - **XState v5 Integration**: Alternative visualization using XState v5 machine definitions with native inspector integration, actor-based state tracking, and clean HTML fallback
  - **Optimized Canvas Rendering**: Enhanced WebAssembly canvas bindings with proper `JsValue::from()` usage for better type safety and performance
  - **Circular Node Layout**: Optimal state positioning algorithm for clear visualization
  - **Enhanced Visual Feedback**: Subtle pulsing effects for current states with dynamic scaling and improved readability
  - **Smart Self-Loop Positioning**: Advanced geometric calculation for self-loop directions using raw angles relative to graph center, with intelligent program change detection to optimize performance by only recalculating positions when switching between different programs
  - **XState Inspector Upgrade**: Major enhancement from custom JSON visualization to XState's native inspector integration with professional state machine visualization, actor-based state tracking, and clean HTML fallback
  - **Template String Escaping**: Fixed quote escaping in HTML template literals within JavaScript code generation for consistent rendering across all browsers
  - **Enhanced Debugging**: Comprehensive console logging for edge detection, animation state tracking, and class application monitoring in HybridGraphView
  - **Template String Integrity**: Fixed JavaScript code generation with proper parameter mapping to ensure all placeholders are correctly filled

- **Stay Direction Support**: Added proper support for the Stay direction (`S`/`-`) across all platforms, including the web interface
- **Symbol Conversion**: Improved character handling with dedicated `convert_symbol_for_parser` function for consistent special character conversion
- **Error Handling Fix**: Corrected error dereferencing in `TuringMachine::validate_program()` for proper analysis error handling
- **Enhanced Grammar**: Improved Pest grammar with better error recovery and flexible syntax support
- **CodeMirror Syntax Highlighting**: Browser-based syntax highlighting system in the web platform's program editor:
  - Custom Turing machine mode definition with comprehensive token recognition
  - Priority-based pattern matching for accurate highlighting precedence
  - Support for all `.tur` format elements including multi-tape syntax
  - Section headers, state names, directions, and symbols with distinct colors
  - Dynamic initialization with DOM mutation observer
  - Optimized performance with CodeMirror's native rendering engine
- **Multi-Tape Support**: Full support for multi-tape Turing machines with proper Stay direction handling
- **Comprehensive Validation**: Static analysis validates programs before execution, checking for undefined states, unreachable states, and invalid tape symbols
- **Flexible Direction Syntax**: Support for multiple direction formats (`L`/`<`, `R`/`>`, `S`/`-`)
- **Comment Support**: Line comments using `#` syntax and trailing comments after transition rules
- **Symbol Quoting**: Single quotes for special characters in tape symbols
- **Robust Error Reporting**: Detailed error messages with line numbers and context for easier debugging