The Mastermind compiler includes optional optimisations for generated code. The original goal of Mastermind was to generate very minimal Brainfuck for use in Code Golf competitions, so most of these are aimed at reducing generated code length.

<!-- TODO: redo this document once planned optimisations are added, separate into frontend, backend, post categories -->

#### Cell Clearing

<!-- backend -->

Optimises clearing cells after they are de-allocated, it does this by tracking their values at compile-time and acting based on a cell's known value. For instance, if a cell can be proven at compile-time to have the value `2`, it is more efficient to clear with `--`, than the typical Brainfuck clear: `[-]`.

#### Constants

<!-- backend -->

When large values are added in Brainfuck, the naive approach is to use the increment `-` operator for as many times as needed. The constants optimiser will use multiplication to shorten the code needed to add/subtract large values. Example: the value `46` can be achieved by either `++++++++++++++++++++++++++++++++++++++++++++++` or the shorter: `+++++[>+++++++++<-]>+` (5 \* 9 + 1).

#### Generated Code

<!-- post -->

Optimises generated Brainfuck code by shortening trivial program segments.

Currently this is limited to optimising segments of Brainfuck programs with the following operations: `+`, `-`, `>`, `<`, `[-]`.

```
--->>><<<++
// becomes:
-
```

An end-to-end example:

```
cell h = 4;
cell j = 3;

h += 10;

drain 10 {
  j = 5;
  h += 4;
  j += 1;
}

// compiles to:
++++>+++<++++++++++>>++++++++++[<+<++++>[-]+++++>-]
// after optimisation:
++++++++++++++>+++>++++++++++[-<[-]+++++<++++>>]
```

This system finds optimal equivalent segments for classic Brainfuck programs, however for the 2D Brainfuck variant it is not guaranteed, as finding the optimal path between memory cells in a 2D grid is more difficult. The _Generated Code Permutations_ setting enables an exhaustive search for the optimal path when using the 2D Brainfuck variant, otherwise a greedy approach is used.

#### Empty Blocks

<!-- frontent -->

Detects if a code block is empty or has no effect on the program, and prunes the associated clause.

#### Unreachable Loops

<!-- backend -->

Brainfuck loops will be omitted if the cell they start on can be proven to be `0` at compile-time.

### Unimplemented Optimisations

#### Memory Allocations

<!-- backend -->

// TODO

<!-- The goal of this is to optimise placing variables in tape memory to minimise movement between them. -->

#### Variable Usage

<!-- frontend -->

// TODO

<!-- The goal of this is to automatically change the order of variable allocations/frees to ensure tape memory is allocated for the smallest amount of execution steps possible. This would allow allocation to be more efficient, as cells can be allocated which would otherwise be taken by variables that are not in use. -->
