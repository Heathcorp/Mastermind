#### Cells

The base data type in Mastermind is the `cell`, this corresponds to a a single 8-bit cell on the Brainfuck tape.

```
cell var = 56;
cell c = 'g';
cell bool = true; // true/false equivalent to 1/0
```

#### Input/Output

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

#### Cell Arrays

Variables can also be defined as contiguous arrays of cells.

```
// multi-cell:
cell[4] array_example = [1, 2, 3, 4];
cell[5] string_example = "hello";
cell[2] foo;
foo[0] = 204;
```

#### Structs

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

#### Variable

When using in-line Brainfuck (see other document), the Brainfuck scope's starting position can be specified with variables:

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
