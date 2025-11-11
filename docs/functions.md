### Functions

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

#### Structs and Overloads

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
