Two-dimensional Brainfuck is an extension of normal Brainfuck to include an additional dimension. 
Some renditions of this include 2D Tapes and 2D programs but for our implementation we will be using
only a 2D Tape. 

The addition of a 2D tape requires 2 additional instructions for moving the tape
head up and down which for our implementation will be '^' for up and 'v' for down.

## Using 2D Brainfuck in Mastermind
To ensure separation between 1D and 2D Brainfuck there is an option in the settings
called 'Enable 2D Brainfuck' when this is enabled it allows our Brainfuck Virtual Machine to read the up and down
characters. If 2D Brainfuck characters are used as input while this setting is off the virtual machine will error.

However, when this setting is enabled our compiler will not move the tape head up or down by default. To make
the compiler use multiple dimensions you must either:
- Use a 2D specific memory allocation algorithm
- Use a 2D location specifier on a variable
- Use inline Brainfuck with 2D instructions

## Memory Allocation Algorithms
There is currently 4 implemented with one being the basic 1D and 3 supporting 2D Brainfuck. While ultimately a fixed algorithm
is not optimal these serve as starting points for the generation of 2D code.

### 1D Mastermind
The 1D Mastermind memory allocation algorithm will allocate the closest free memory location to the right of the origin. 
This means that it will first allocate one spot to the right then the next to the right.

### 2D Mastermind - Zig Zag
The 2D Mastermind Zig Zag allocation algorithm will treat the memory as a grid and will fill in values from x 0 and some 
y value diagonally until it reaches y 0 and the same x value as the starting y. The table below shows how this is populated

| 7   |     |     |     |
| --- | --- | --- | --- |
| 4   | 8   |     |     |
| 2   | 5   | 9   |     |
| 1   | 3   | 6   | 10  | 


### 2D Mastermind - Spiral
The 2D Mastermind Spiral allocation algorithm will start from 0,0 and move in a Spiral such that each subsequent memory 
value is only 1 step away from the last. This means that it will start by filling a 2x2 grid then from the bottom corner of 
that grid it will iterate around that 2x2 filling a 4x4 area

| 10  | 11  | 12  |
| --- | --- | --- |
| 9   | 2   | 3   |
| 8   | 1   | 4   |
| 7   | 6   | 5   |

### 2D Mastermind - Tiles
The 2D Mastermind tiles allocation algorithm will allocate a tile of memory and check all cells in that area before 
expanding to check new cells. This algorith starts at 0,0 with a 1x1 area then will move down to -1, -1 and check a new 3x3 area
it will check each area column by column from the bottom row up so (-1, -1), (0, -1), (1, -1), (-1, 0)...

| 4  | 6  | 9  |
|----|----|----|
| 3  | 1  | 8  |
| 2  | 5  | 7  |