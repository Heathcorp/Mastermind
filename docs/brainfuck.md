### Brainfuck

Brainfuck is an esoteric programming language, originally designed as a theoretical example of a Turing complete language with an extremely minimal compiler. The name is due to its difficulty, it is significantly more difficult to create complex programs than in any popular modern language.

### Specification

When a Brainfuck program is run, it operates on a array/tape of cells, performing operations on the tape. Each cell contains an integer, initialised to 0 by default. The program operates on one cell at a time based on the position of a "tape head". Brainfuck supports the following operations:

- `+`: increments the value of the current cell
- `-`: decrement the value of the current cell
- `>`: move the tape head one cell to the right
- `<`: move the tape head one cell to the left
- `.`: output the current cell as a byte to stdout
- `,`: input a byte from stdin, overwriting the current cell
- `[`: jump to the corresponding `]` if the current cell is 0
- `]`: jump to the corresponding `[` if the current cell is not 0

A Brainfuck program consists of a list of these commands, which are executed sequentially. The program terminates if the final operation in the list is executed.

### Interpreter Implementation Details

The Mastermind IDE and compiler library contains an implementation of a Brainfuck interpreter. This implementation is intended to match the behaviour of the most popular Brainfuck implementations:

#### 8-bit Wrapping Cells

In this implementation, each cell is an 8-bit integer that wraps if an increment or decrement operation overflows or underflows.

E.g. given the current tape cell value is `255`, after an increment (`+`), the cell value is now `0`.

Similarly: `0`, after a decrement (`-`) becomes `255`

#### Infinite Bidirectional Tape

In this implementation, the tape extends infinitely in both directions.
