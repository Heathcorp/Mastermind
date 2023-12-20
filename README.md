# Mastermind

A programming language designed to compile to brainfuck.

The reason I say "compile" instead of "transpile" is because, in my mental model brainfuck is the machine language of a theoretical virtual machine. In it's current state it is probably closer to an assembler.

### Note

This is my first Rust project so feedback is appreciated, except if you want to argue about my use of hard tabs.
I've also never written a compiler before and deliberately tried not to read any literature before attempting this, so it may seem fairly primitive.

### TODO

1. Multi-byte integers
2. ~~Automatic consumption and freeing of variables when leaving scope~~
3. Some kind of stack or array implementation
4. ~~Whitespace-agnostic syntax~~
5. ~~Simple include system~~, with example files for "standard library" style code
6. ~~A simple web interface~~
7. In-line brainfuck (in-line assembly) so we can make optimisations for the compiler and use unsafe looping moves (in order to achieve #1).
   - Idea: asm(a, b, c[3], d[(contiguous)5, etc (need to make syntax for this]) {#goto(a)+++>[[]--]};

### Web-interface TODO:

1. More robust github actions workflows
1. Button to copy compiled code
2. Compiled code size indicator
1. Live I/O for running program
1. More robust non-blocking compiler calls. Currently if the "run_code" function hangs, the whole webpage is frozen. Maybe look into web workers for this.
1. Playtest and fix common errors

### Playtest feedback:

1. Frozen browser on blocking wasm calls
2. Semicolons after curly-braces is annoying
3. Maybe make functions clearer that they are more like macros
4. Tabs not working in editor
