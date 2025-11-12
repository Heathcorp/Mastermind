# Mastermind reference

## Introduction

Mastermind is a programming language designed to compile to the well-known esoteric language "Brainfuck".

Brainfuck is essentially a modern interpretation of the classical Turing machine. It consists of a tape of 8-bit values, with simple increment/decrement, move left/right, input/output, and looping operations. The full language only uses 8 control characters: `+-><.,[]`.

Imagine if C was designed for computer architectures that run Brainfuck, that is what Mastermind is intended to be.

## Contents

- [Brainfuck](#brainfuck)
- [Variables](#variables)
- [Conditionals](#conditionals)
- [Loops](#loops)
- [Functions](#functions)
- [Inline Brainfuck](#inline-brainfuck)
- [Standard Library](#standard-library)
- [Variants](#variants)
- [Optimisations](#optimisations)

## Brainfuck

Brainfuck is an esoteric programming language, originally designed as a theoretical example of a Turing complete language with an extremely minimal compiler. The name is due to its difficulty, it is significantly more difficult to create complex programs than in any popular modern language.

### Specification

When a Brainfuck program is run, it operates on a array/tape of cells, performing operations on the tape. Each cell contains an integer, initialised to 0 by default. The program operates on one cell at a time based on the position of a "tape head". Brainfuck supports the following operations:

- `+`: increment the value of the current cell
- `-`: decrement the value of the current cell
- `>`: move the tape head one cell to the right
- `<`: move the tape head one cell to the left
- `.`: output the current cell as a byte to stdout
- `,`: input a byte from stdin, overwriting the current cell
- `[`: jump to the corresponding `]` if the current cell is 0
- `]`: jump to the corresponding `[` if the current cell is not 0

A Brainfuck program consists of a list of these commands, which are executed sequentially. The program terminates if the final operation in the list is executed.

### Interpreter Implementation Details

The Mastermind IDE and compiler library contains an implementation of a Brainfuck interpreter. This implementation is intended to match the behaviour of the most popular Brainfuck implementations:

#### 8-bit Wrapping Cells

In this implementation, each cell is an 8-bit integer that wraps if an increment or decrement operation overflows or underflows.

E.g. given the current tape cell value is `255`, after an increment (`+`), the cell value is now `0`.

Similarly: `0`, after a decrement (`-`) becomes `255`

#### Infinite Bidirectional Tape

In this implementation, the tape extends infinitely in both directions.

## Variables

### Cells

The base data type in Mastermind is the `cell`, this corresponds to a a single 8-bit cell on the Brainfuck tape.

```
cell var = 56;
cell c = 'g';
cell bool = true; // true/false equivalent to 1/0
```

Cells default to `0`.

### Input/Output

The `input` and `output` keywords in Mastermind correspond to the `,` and `.` operators in Brainfuck. `input` simply inputs the next byte from stdin, and `output` outputs a byte to stdout.

```
// stdin: 00abc
cell g;
drain 5 {
  // do this 5 times
  input g;
  g += 1;
  output g;
}
// stdout: 11bcd
```

The simplest way to display text is to output valid ASCII characters. If your Brainfuck implementation supports unicode, that is also possible by outputting multiple bytes.

```
output 240;
output 159;
output 164;
output 145;
output 10;
// displays 🤑 (emoji with green cash for tongue)
```

### Cell Arrays

Variables can also be defined as contiguous arrays of cells.

```
// multi-cell:
cell[4] array_example = [1, 2, 3, 4];
cell[5] string_example = "hello";
cell[2] foo;
foo[0] = 204;
```

### Structs

Structure types can be defined with named fields, then instantiated as variables.

```
struct struct_name {
  cell x;
  cell y;
  cell[5] zzzzz;
}

struct struct_name s;
s.x = 4;
s.y = 123;
s.zzzzz[0] += 3;
s.zzzzz[4] = 180;

// nested struct:
struct Nested {
  struct struct_name n;
}
```

### Structs and Arrays

Any type can be repeated into an array/contiguous allocation. This includes cells, structs, arrays of cells, and arrays of structs.

```
cell[4][6] mult_arr; // a 6-length array of cell[4] arrays
cell[4][6][2] mult_arr; // 2 * (6-length arrays of cell[4] arrays)

struct T {
  cell a;
  cell[4][2] b;
}

struct T[10] ten_T_structs;
ten_T_structs[4].b[1][3] = 45;

struct S {
  struct T[2][4] matrix_of_T_structs;
  cell other;
}

struct S[3] three_S_structs;
three_S_structs[1].matrix_of_T_structs[3][0] = '5';
```

#### Note: Array indices must be compile-time constant integers

This is a limitation of Brainfuck, getting around this problem requires more runtime code than is reasonable to include by default, due to the goals of Mastermind. You can implement equivalent behaviour using in-line Brainfuck, structs, and functions.

### Location specifiers

The exact memory cells occupied by a variable can be specified:

```
// value 1 at tape position 4
cell a @4 = 1;
// contiguous array of 1s, starting at cell -1
cell[3] a @-1 = [1, 1, 1];
```

#### Struct subfields

The byte-order and positioning of a struct's subfields can be specified:

```
struct T {
  cell a @1;
  cell b[2] @3;
}
// struct T's layout:
// (-, a, -, b[0], b[1])
// '-' denotes an untracked padding cell
```

## Conditionals

Mastermind supports basic `if`/`else` statements. An `if` statement accepts an expression that evaluates to a `cell` type, if the expression is evaluated to be truthy (i.e. not equal to `0`), then the `if` block is executed, otherwise the optional `else` block is executed. This behaviour can be inverted using the `not` keyword.

```
if 13 {
  output "13";
}

if not true {
  // unreachable
}

cell var = 4;
if var {
  output "true";
} else {
  output "false";
}

// typical equivalence use-case:
if not var - 10 {
  // ==
} else {
  // !=
}
```

## Loops

Mastermind currently supports three forms of loops: `while`, `drain` and `copy`.

It should be noted that there is no early breaking in any of these forms, so all clauses in a loop body are always executed in each iteration.

### While

The `while` loop operates similarly to other languages, accepting a condition expression, and a loop body.

The clauses inside the loop body are executed until the condition is falsy (i.e. equal to `0`). The condition is checked before each iteration.

Note: currently `while` conditions must be direct variable references, this is subject to future compiler updates.

```
cell n = 5;
while n {
  // do stuff
  n -= 1;
}
// n is now equal to 0
```

### Drain

The `drain` loop mirrors a very common pattern found in Brainfuck programs: decrementing a cell. `drain` accepts an expression, a list of variables to 'drain into', and/or a loop body.

If the expression is a direct variable reference, then the variable is decremented after each iteration. If not, it is evaluated in a temporary cell, then decremented after each iteration.

```
drain var {
  // do stuff
}

// equivalent to:
while var {
  // do stuff
  var -= 1;
}
```

With expressions:

```
drain 6 {
  output 'a';
}
// aaaaaa
```

The following example leaves `x` unchanged:

```
cell x = 7;
drain x - 2 {
  output 'b';
}
// bbbbb
```

#### Into

If the `into` keyword is used, followed by a whitespace-separated list of target variables, the targets will be incremented after each iteration.

```
cell i;
drain 10 into i {
  output '0' + i;
}
// 0123456789

// equivalent to:
cell i;
cell ten = 10;
while ten {
  output '0' + i;

  i += 1;
  ten -= 1;
}
```

Another example:

```
drain var into other_var other_var_2 *spread_array;

// equivalent to:
drain var {
  other_var += 1;
  other_var_2 += 1;
  spread_array[0] += 1;
  spread_array[1] += 1;
  spread_array[2] += 1;
  // ...
}
```

### Copy

The `copy` loop acts similarly to the `drain` loop, however the expression must be a direct variable reference, and it is left unchanged afterwards, and its original value is accessible within the loop body.

```
cell y;
copy x into y {
  // loop body
};
```

An equivalence example:

```
cell var = 5;
copy var {
  output '0' + var;
}
// 55555

// equivalent to:
cell var = 5;
cell temp = var;
while temp {
  output '0' + var;

  temp -= 1;
}
```

## Functions

Mastermind supports a minimal function system: functions can be defined with a name and a fixed number of typed arguments.

```
fn newline() { output '\n'; }

fn print_zeros(cell num) {
  copy num {
    output '0';
  }
  newline();
}

// expressions as arguments are currently not supported,
//  i.e. print_zeros(9)
cell g = 9;
print_zeros(g);
```

Unlike most modern programming languages, functions are not considered first-class values. Functions in Mastermind are in-lined at compile-time, and all arguments are passed by reference. Values can be returned by editing passed in arguments, or editing variables in an outer scope, although the latter makes a function less portable.

```
fn is_zero(cell in, cell out) {
  out = true;
  if in {
    out = false;
  }
}

cell value = 'h';
cell falsy;
is_zero(value, falsy);
```

Example showing a function reading a variable from an outer scope:

```
fn print_global_g(cell count) {
  copy count {
    output chr;
  }
}

cell chr = 'g';
cell count = 3;
print_global_g(count);
// ggg

{
  // inner scope with a new 'g' allocation
  cell chr = 'G';
  count = 5;
  print_global_g(count);
  // GGGGG
}

// same call again, now the inner chr has been freed
print_global_g(count);
// ggg
```

### Types and Overloads

Functions support overloads with different types or number of arguments. Examples of supported behaviour:

```
fn func1() {
  output '1';
}
fn func1(cell a) {
  output '2';
}
fn func1(cell a, cell b) {
  output '3';
}
struct X { cell a; }
fn func1(struct X x) {
  output '4';
}
struct Y { cell a; }
fn func1(struct Y y) {
  output '5';
}
fn func1(cell a, struct X x, struct Y y) {
  output '6';
}
cell n;
struct X x;
struct Y y;
func1();
func1(n);
func1(n, n);
func1(x);
func1(y);
func1(n, x, y);
// 123456
```

## In-Line Brainfuck

In-line Brainfuck allows the programmer to define custom behaviour as if writing raw Brainfuck, inspired by in-line assembly in C.

Basic example:

```
// find the next cell that equals -1
bf {
   +[->+]-
}
```

More advanced example:

```
// input a line of lowercase letters and output the uppercase version
// this is an intentionally inefficient example
bf @3 clobbers var *spread_var etc {
   ,----------[++++++++++>,----------]
   <[<]>
   [
      {
         cell g @0;
         assert g unknown;
         output g + ('A' - 'a');
         // embedded Mastermind!
      }
      >
   ]
   // now clear and return
   <[[-]<]>
}
```

It is the programmer's responsibility to clear used cells and return back to the cell in which they started the in-line Brainfuck context. If the programmer does not do this, any following Mastermind code may break.

### Memory location specifiers

The exact location to start an in-line Brainfuck context can be specified:

```
cell var @3 = 4;
// compiled: >>>++++

bf @4 {
   <><><>
}
// compiled: >>>><><><>
```

Variables can also be used:

```
cell d;
bf @d {
  // brainfuck code here
}

struct G {
  cell h;
  cell i;
  cell j;
}
struct G g;

bf @g {
  // starts on the first cell of g's allocation
}
// equivalent to:
bf @g.h {}
```

### Clobbering and Assertions

With optimisations enabled, Mastermind will try to predict the value of cells at compile-time, so it can prevent unnecessary cell clean-ups and unreachable code. If your in-line Brainfuck affects existing Mastermind variables, you should tell the compiler using the `clobbers` keyword, the syntax is similar to the `drain into` target list:

```
bf clobbers var *spread_var other_var etc {}
```

The compiler will now assume nothing about the values of those variables afterwards.

If instead you want to tell the compiler specifically that a variable has become a certain value, you can use `assert`:

```
assert var equals 3;
// most common use cases:
assert var equals 0;
assert var unknown;
```

Asserting a variable as `unknown` is equivalent to clobbering.

### Embedded Mastermind

You can embed high-level Mastermind code within a Brainfuck context. During compilation the embedded Mastermind is compiled and the generated Brainfuck is inserted in place.

```
// input 3 n-length lines of input
bf {
  >+++<,[
    {
      cell input_char @0;
      assert input_char unknown;
      cell length_remaining @1;
      assert length_remaining unknown;

      cell next_char @2;
      cell next_length_remaining @3;
      if not input_char - '\n' {
        length_remaining -= 1;
      }
      if length_remaining {
        drain length_remaining into next_length_remaining;
        input next_char;
      }
    }
  >>]
}
```

Embedded Mastermind can include in-line Brainfuck, this is recursive. For example:

```
// top-level Mastermind context
bf {
  ++>>
  {
    // inner Mastermind context
    bf {
      ++>>
      {
        // inner inner Mastermind context
        bf {
          ++>>
          {
            //...
          }
          <<--
        }
      }
      <<--
    }
  }
  <<--
}
```

The compiler cannot guarantee the global head position at compile time within an in-line Brainfuck context. Therefore memory location specifiers are relative to the current embedded Mastermind context, not the entire program.

Also, top-level variables are not cleared by default in Mastermind contexts, this allows you to "leave" variables in cells for your Brainfuck to use. If you want variables in your embedded Mastermind to be automatically cleared, you can open a scope at the top level:

```
bf {
   ++----++[][][<><><>] // the program doesn't matter for this example
   {
      // variables here will not be cleared
      cell g @2;
      assert g unknown;
      {
         // variables here will be cleared
         let b = 32;
      }
   }
   {{
      // self-cleaning Mastermind code here
   }}
}
```

## Standard Library

Currently the Mastermind standard library is very limited, and is effectively a set of example programs included in the web IDE and source repository.

### Including files

You can include/import other files using preprocessor directives. The Mastermind preprocessor is intended to mirror the C preprocessor, however it currently only supports the `#include` directive.

The following is a basic example:

```
// file1.mmi
struct H {
  cell a;
}
fn print(struct H h) {
  output h.a;
}
```

```
// main file being compiled
#include "file1.mmi"

struct H h;
h.a = 64;
print(h);
// @
```

### Standard Library Examples

The most mature files in the included examples are the following:

- `bitops`: bitshifting operations for cell types
- `i8`: signed type for 8-bit integers and supporting functions
- `u8`: common supporting functions for cell types
- `u16`: a 16-bit unsigned integer type and supporting functions
- `ifp16`: a signed 16-bit fixed-point number type and supporting functions

NOTE: due to current lack of header-guard support, importing multiple of these will likely cause a compiler error, until this is implemented, the best way to work around this is to only include `ifp16` as that includes the others.

Example usage:

```
#include <u16>

// read a 16 bit number from stdin, add 55, then print

struct u16 n;
read(n);

cell ff = 55;
add(n, ff);
print(n);
output ' ';
debug(n); // print the binary representation
// example input: 16000
// output: 16055 0011111010110111
```

Example fixed-point usage:

```
#include <ifp16>

struct ifp16 n;
_99p99609375(n); // constant 99.99609375
struct ifp16 m;
__1p5(m); // constant -1.5

divide(n, m);
print(n);
output ' ';
debug(n);
// -66.66 10111101.01010110
```

## Variants

The Mastermind compiler can be extended to support Brainfuck variants.

### Supported Variants:

#### Classic (1D) Brainfuck

This is the default behaviour, typical Brainfuck implementation as described in [Brainfuck](#brainfuck).

#### 2D Brainfuck

Mastermind currently supports two-dimensional Brainfuck, this is a Brainfuck variant with an additional dimension in the memory array.

2D Brainfuck support can be enabled in the compiler settings in the web IDE, adding the following features:

- New opcodes for in-line Brainfuck contexts and in generated Brainfuck code:
  - `^`: move up one cell in the grid
  - `v`: move down one cell in the grid
- The ability to specify 2D coordinates for location specifiers:
  ```
  cell var @(5, -7) = 'a';
  bf @var {[-<<<<<^^^^^^^+>>>>>vvvvvvv]}
  bf @(0, 0) {.....}
  // aaaaa
  ```
- Three new memory allocation strategies for generated 2D code:
  - Zig Zag
  - Spiral
  - Tiles
  <!-- // TODO: explain what these do -->

## Optimisations

The Mastermind compiler includes optional optimisations for generated code. The original goal of Mastermind was to generate very minimal Brainfuck for use in Code Golf competitions, so most of these are aimed at reducing generated code length.

<!-- TODO: redo this document once planned optimisations are added, separate into frontend, backend, post categories -->

### Cell Clearing

<!-- backend -->

Optimises clearing cells after they are de-allocated, it does this by tracking their values at compile-time and acting based on a cell's known value. For instance, if a cell can be proven at compile-time to have the value `2`, it is more efficient to clear with `--`, than the typical Brainfuck clear: `[-]`.

### Constants

<!-- backend -->

When large values are added in Brainfuck, the naive approach is to use the increment `-` operator for as many times as needed. The constants optimiser will use multiplication to shorten the code needed to add/subtract large values. Example: the value `46` can be achieved by either `++++++++++++++++++++++++++++++++++++++++++++++` or the shorter: `+++++[>+++++++++<-]>+` (5 \* 9 + 1).

### Generated Code

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

### Empty Blocks

<!-- frontent -->

Detects if a code block is empty or has no effect on the program, and prunes the associated clause.

### Unreachable Loops

<!-- backend -->

Brainfuck loops will be omitted if the cell they start on can be proven to be `0` at compile-time.
