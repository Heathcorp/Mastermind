### Optimisations

The optimisations in the Mastermind compiler are aimed at reducing the compiled Brainfuck code length, not necessarily execution speed. This is due to the original goal of the project: Code Golf in Brainfuck.

#### Cell Clearing

This optimises the clearing of cells by tracking their values at compile-time. For instance, if a cell can be proven at compile-time to have the value `2`, it is more efficient to clear with `--`, than the typical Brainfuck clear: `[-]`.

#### Constants

When large values are added in Brainfuck, the naive approach is to use the increment `-` operator for as many times as needed. The constants optimiser will use multiplication to shorten the code needed to add/subtract large values. Example: the value `45` can be achieved by either `+++++++++++++++++++++++++++++++++++++++++++++` or the shorter: `+++++[<+++++++++>-]>`.

#### Empty Blocks

This detects if a code block is empty, and does not compile the clause associated. This is helpful for `if` statements and `copy` loops especially, as those can imply extra overhead for copying cells.

#### Generated Code

This is a final pass optimisation that operates directly on Brainfuck code, optimising subsets of programs which can be shortened while still guaranteeing equivalent behaviour. Example:

```
--->>><<<++
```

Is equivalent to:

```
-
```

It is difficult to analyse the behaviour of a Brainfuck program at compile time, so this optimiser is limited to subsets of a program's operations between I/O operations and loops (with exception). Example:

```
cell h = 4;
cell j = 3;

h += 10;

drain 10 {
  j = 5;
  h += 4;
  j += 1;
}
```

Compiles to:

```
++++>+++<++++++++++>>++++++++++[<+<++++>[-]+++++>-]
```

After optimisation:

```
++++++++++++++>+++>++++++++++[-<[-]+++++<++++>>]
```

For the 2D compiler extensions, this system can use an exhaustive search to determine the least movement between cells. This could become slow depending on the project, so it can be configured to use a greedy approach. This is done via the _Generated Code Permutations_ setting in the web IDE.

#### Unreachable Loops

If a cell is known to have a value of `0` at compile time, and that cell is used to open a Brainfuck loop, then that entire loop is omitted. This is implemented at a low level, so it is agnostic of the syntactic structure that it is optimising, i.e `if`, `while`, `drain`.

### Unimplemented Optimisations

#### Memory Allocations

The goal of this is to optimise placing variables in tape memory to minimise movement between them.

#### Variable Usage

The goal of this is to automatically change the order of variable allocations/frees to ensure tape memory is allocated for the smallest amount of execution steps possible. This would allow allocation to be more efficient, as cells can be allocated which would otherwise be taken by variables that are not in use.
