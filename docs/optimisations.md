## This Page Is A Work In Progress

### Cell Clearing
TBD

### Constants
Constants will optimise constant values by finding the best combination of products and a constant. For example 18 could 
be expressed as 6 * 3 or 2 * 9 which could reduce the amount of increment or decrement instructions required. Additionally 
the overflowing property of our cells makes it more efficient to decrement 0 to wrap around to 255 

### Empty Blocks
The empty blocks optimisation will remove code that the compiler can see will either never be entered or is empty when executed 

### Generated Code
Generated code is an optimisation that runs after Brainfuck code has been compiled. It optimises the movement between 
memory locations in code blocks that are safe to optimise this will be any code between input/output, loops, functions and conditional statements.

This algorithm has 2 options a greedy approach which will find a solution that works and then an exhaustive approach to try all
permutations of movement between memory cells. Since this may be slow for larger code blocks this is configurable

### Generated Code Permutations
This setting swaps between the greedy and exhaustive algorithm for our Generated Code optimisation. If generated code is not enabled
this config will have no impact.

### Memory Allocations
TBD

### Unreachable Loops
TBD

### Variable Usage
TBD