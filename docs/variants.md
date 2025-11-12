The Mastermind compiler can be extended to support Brainfuck variants.

### 2D Brainfuck

Mastermind currently supports two-dimensional Brainfuck, this is a Brainfuck variant with an additional dimension in the memory array.

2D Brainfuck support can be enabled in the compiler settings in the web IDE, adding the following:

New opcodes for in-line Brainfuck contexts and in generated Brainfuck code:

- `^`: move up one cell in the grid
- `v`: move down one cell in the grid
-

#### Memory Allocation Algorithms

##### Default

Allocates the closest free cells to the right of the origin.

##### Zig Zag

// TODO

<!-- Treats the memory as a grid and fills in values from x 0 and some y value diagonally until it reaches y 0 and the same x value as the starting y. The table below shows the order that this is populated

| 7   |     |     |     |
| --- | --- | --- | --- |
| 4   | 8   |     |     |
| 2   | 5   | 9   |     |
| 1   | 3   | 6   | 10  | -->

##### Spiral

// TODO

<!-- _2D Mastermind - Spiral_ starts from 0,0 and move in a Spiral such that each subsequent memory
value is only 1 step away from the last. This means that it will start by filling a 2x2 grid then from the bottom corner of
that grid it will iterate around that 2x2 filling a 4x4 area

| 10  | 11  | 12  |
| --- | --- | --- |
| 9   | 2   | 3   |
| 8   | 1   | 4   |
| 7   | 6   | 5   | -->

##### Tiles

// TODO

<!-- _2D Mastermind - Tiles_ allocates a tile of memory and check all cells in that area before expanding to check new cells. This algorithm starts at 0,0 with a 1x1 area then will move down to -1, -1 and check a new 3x3 area it will check each area column by column from the bottom row up so (-1, -1), (0, -1), (1, -1), (-1, 0)...

| 4   | 6   | 9   |
| --- | --- | --- |
| 3   | 1   | 8   |
| 2   | 5   | 7   | -->
