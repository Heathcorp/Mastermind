### Two-Dimensional Brainfuck

Two-dimensional Brainfuck is an extension which provides an additional dimension to the memory tape.

To support this, two new operations have been added to this extended version of the language:

- `^`: move up one cell on the grid
- `v`: move down one cell on the grid

#### Using 2D Brainfuck in Mastermind

This behaviour must be enabled in the included Brainfuck interpreter. In the web IDE this is done via the settings modal.

When this setting is enabled in isolation, the compiler will still generate typical 1D Brainfuck code. To make the compiler use multiple dimensions you must either:

- Use a 2D-specific memory allocation algorithm
- Use a 2D location specifier on a variable
- Use in-line Brainfuck with 2D instructions

### Memory Allocation Algorithms

There are currently four allocation strategies implemented (including the original 1D).

#### 1D Mastermind

_1D Mastermind_ allocates the closest free cells to the right of the origin.

#### 2D Mastermind - Zig Zag

_2D Mastermind - Zig Zag_ treats the memory as a grid and fills in values from x 0 and some y value diagonally until it reaches y 0 and the same x value as the starting y. The table below shows the order that this is populated

| 7   |     |     |     |
| --- | --- | --- | --- |
| 4   | 8   |     |     |
| 2   | 5   | 9   |     |
| 1   | 3   | 6   | 10  |

#### 2D Mastermind - Spiral

_2D Mastermind - Spiral_ starts from 0,0 and move in a Spiral such that each subsequent memory
value is only 1 step away from the last. This means that it will start by filling a 2x2 grid then from the bottom corner of
that grid it will iterate around that 2x2 filling a 4x4 area

| 10  | 11  | 12  |
| --- | --- | --- |
| 9   | 2   | 3   |
| 8   | 1   | 4   |
| 7   | 6   | 5   |

#### 2D Mastermind - Tiles

_2D Mastermind - Tiles_ allocates a tile of memory and check all cells in that area before expanding to check new cells. This algorithm starts at 0,0 with a 1x1 area then will move down to -1, -1 and check a new 3x3 area it will check each area column by column from the bottom row up so (-1, -1), (0, -1), (1, -1), (-1, 0)...

| 4   | 6   | 9   |
| --- | --- | --- |
| 3   | 1   | 8   |
| 2   | 5   | 7   |
