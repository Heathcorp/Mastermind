# Mastermind

A programming language designed to compile to brainfuck.

The reason I say "compile" instead of "transpile" is because, in my mental model, brainfuck is the machine language of a theoretical virtual machine.

### Note

This is my first Rust project so feedback is appreciated, except if you want to argue about my use of hard tabs.
I've also never written a compiler before and deliberately tried not to read any literature before attempting this, so it may seem fairly primitive.

### Compiler TODO:

1. ~~Add input command~~
2. ~~Remove need for semicolons after curly braces~~
3. ~~Make multi-byte variables contiguous~~
4. ~~Add string literals for multi-byte variables~~
5. ~~Make output clause work with multi-byte variables by reference and strings/arrays by expression~~
6. ~~Convert panics to result types?~~
7. Keep track of source code line for better panic messages
8. ~~Fix issue with variable/function names that start with keywords~~
9. ~~Simple include system~~, with example files for "standard library" style code
10. In-line brainfuck (in-line assembly) so we can make optimisations for the compiler and use unsafe looping moves.
   - Idea: asm(a, b, c[3], d[(contiguous)5, etc (need to make syntax for this)]) {#goto(a)+++>[[]--]};
11. Some kind of stack or array implementation
12. Multi-byte integers
13. n-length strings?
14. ~~Maybe make functions clearer that they are more like macros?~~
15. ~~Make if statements and drain loops able to use expressions.~~

### Web-interface TODO:

1. ~~Make tab button work in editor~~
2. ~~Reorderable file tabs~~
3. ~~Button to copy compiled code~~
4. ~~Compiled code size indicator~~
5. ~~More robust github actions workflows~~?
6. ~~Pass panic messages from compiler into output panel~~
7. ~~Live I/O for running program~~
8. ~~More robust non-blocking compiler calls. Currently if the "run_code" function hangs, the whole webpage is frozen. Maybe look into web workers for this.~~
9. Set of loadable example (stdlib) files
10. Documentation?
11. ~~Syntax highlighting?~~
12. Vim keybindings (someone suggested as a joke but is actually quite easy)
13. ~~Button to kill running brainfuck program~~
14. ~~Improve file name editing~~
15. ~~Fix weird top tab height changing issue~~
16. ~~Check browser compatibility~~
17. Playtest
