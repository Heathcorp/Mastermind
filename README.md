# Mastermind

A programming language designed to compile to brainfuck.

The reason I say "compile" instead of "transpile" is because, in my mental model, brainfuck is the machine language of a theoretical virtual machine.

### Note

This is my first Rust project so feedback is appreciated, except if you want to argue about my use of hard tabs.
I've also never written a compiler before and deliberately tried not to read any literature before attempting this, so it may seem fairly primitive.

### Compiler TODO:

1. Remove need for semicolons after curly braces
1. Make multi-byte variables contiguous
2. Add string literals for multi-byte variables
3. Make output clause work with multi-byte variables by reference and strings/arrays by expression
4. ~~Simple include system~~, with example files for "standard library" style code
5. In-line brainfuck (in-line assembly) so we can make optimisations for the compiler and use unsafe looping moves.
   - Idea: asm(a, b, c[3], d[(contiguous)5, etc (need to make syntax for this]) {#goto(a)+++>[[]--]};
6. Some kind of stack or array implementation
7. Multi-byte integers
8. n-length strings?
3. Maybe make functions clearer that they are more like macros?

### Web-interface TODO:

1. Make tab button work in editor
1. More robust github actions workflows
1. Button to copy compiled code
2. Compiled code size indicator
1. Live I/O for running program
1. More robust non-blocking compiler calls. Currently if the "run_code" function hangs, the whole webpage is frozen. Maybe look into web workers for this.
2. Reorderable file tabs
3. Set of loadable example (stdlib) files

5. Documentation?
