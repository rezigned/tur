# Turing Machine Programs

This directory contains example Turing machine programs in `.tur` format. These programs demonstrate both single-tape and multi-tape Turing machine capabilities and can be loaded and executed by the Turing machine simulator across all platforms (CLI, TUI, and Web).

## File Format

Turing machine programs are defined in a simple text format with the `.tur` extension. The format consists of three main sections:

1. **Name**: The name of the program
2. **Tape**: The initial tape content
3. **Transitions**: The state transition rules

### Example

```tur
name: Simple Test
tape: a
rules:
  start:
    a -> b, R, halt
  halt:
```

### Syntax

The `.tur` file format uses a structured syntax parsed by a Pest grammar with comprehensive validation:

- **Name**: Specified with `name:` followed by the program name
- **Tape Configuration**:
  - **Single-tape**: `tape: symbol1, symbol2, symbol3`
  - **Multi-tape**: 
    ```tur
    tapes:
      [a, b, c]
      [x, y, z]
    ```
- **Head Positions** (optional):
  - **Single-tape**: `head: 0` (defaults to 0)
  - **Multi-tape**: `heads: [0, 0]` (defaults to all zeros)
- **Blank Symbol**: `blank: _` (defaults to space character)
- **Transition Rules**: Specified with `rules:` followed by state definitions
  - Each state is defined by its name followed by a colon
  - Each transition rule is indented and specifies:
    - **Single-tape**: `symbol -> new_symbol, direction, next_state`
    - **Multi-tape**: `[sym1, sym2] -> [new1, new2], [dir1, dir2], next_state`
    - **Directions**: `R`/`>` (right), `L`/`<` (left), `S`/`-` (stay)
    - **Write-only transitions**: If `-> new_symbol` is omitted, the read symbol is preserved
  - The first state defined is automatically the initial state
- **Comments**: Use `#` for line comments and inline comments
- **Special Symbols**: 
  - `_` represents the blank symbol in program definitions
  - Any Unicode character can be used as tape symbols
  - The blank symbol can be customized with the `blank:` directive

## Available Examples

### Single-Tape Programs
- **binary-addition.tur**: Adds two binary numbers
- **binary-counter.tur**: Increments a binary number
- **busy-beaver-3.tur**: Classic 3-state busy beaver
- **event-number-checker.tur**: Checks if a number is even
- **palindrome.tur**: Checks if input is a palindrome
- **subtraction.tur**: Subtracts two numbers

### Multi-Tape Programs
- **multi-tape-addition.tur**: Addition using multiple tapes
- **multi-tape-compare.tur**: Compares content across tapes
- **multi-tape-copy.tur**: Copies content from one tape to another
- **multi-tape-example.tur**: Basic multi-tape demonstration

Each program includes comprehensive comments explaining the algorithm and state transitions.
