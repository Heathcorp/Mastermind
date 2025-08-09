### Functions

Mastermind supports a minimal functions system: Functions can be defined with a name and a fixed number of typed arguments.

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

Functions are in-lined at compile-time, and all arguments are passed by reference. Values can be returned by editing the arguments, or editing variables in an outer scope, although the latter makes a function less portable.

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
    output g;
    output ' ';
  }
}

cell g = 'g';
cell count = 11;
print_global_g(count);
// g g g g g g g g g g g

{
  // inner scope with a new 'g' allocation
  cell g = 'G';
  count = 4;
  print_global_g(count);
  // G G G G
}

// same call again, now the inner 'G' has been freed
print_global_g(count);
// g g g g
```

#### Structs and Overloads

Example of supported behaviour:

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
