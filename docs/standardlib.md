### Mastermind Standard Library

Currently the Mastermind standard library is very limited, and is effectively a set of example programs included in the web IDE and source repository.

#### Including files

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

#### Standard Library Examples

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
