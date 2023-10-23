# Mastermind

A programming language designed to compile to brainfuck.

The reason I say "compile" instead of "transpile" is because, in my mental model brainfuck is the machine language of a theoretical virtual machine. In it's current state it is probably closer to an assembler.

### Note

This is my first Rust project so feedback is appreciated, except if you want to argue about my use of hard tabs.
I've also never written a compiler before and deliberately tried not to read any literature before attempting this, so it may seem fairly primitive.

### Future Ideas

- Optimisations: optimise for speed and efficiency, or for minimal output size (golf mode), could have an automated tool that detects stretches of code that are equivalent to each other.

### Requirements for v1 release

1. Multi-byte integers
2. Automatic consumption and freeing of variables when leaving scope
3. Some kind of stack or array implementation
4. Maybe to get 2 and 3 implement a "with" construction so it's clear when a variable is in scope
5. Whitespace-agnostic syntax
6. Get somebody else to playtest and fix any issues that arise
