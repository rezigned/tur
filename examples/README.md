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
