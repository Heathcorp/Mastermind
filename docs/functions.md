Functions in Mastermind work more like templates/macros, as they do not perform any passing by value. All functions are essentially inlined at compile time. This means multiple calls to a large function will significantly increase your compiled code size.

Functions are created using the def command followed by the function name and a list of typed arguments inside of
( ). 

Functions currently do not support return values since they function as marcos and will instead just update the value of variables passed in

```
def quote(cell arg) {
   output 39; // ASCII single quote
   output arg;
   output 39;
}

cell N = 'g';
quote<N>;
N += 3;
quote<N>;
//OUTPUT 
// 'g''j'
```

When Structs are used in conjunction with Functions you are allowed to define multiple functions with the same name and it 
will use the input to perform function overloading.

```
struct test {
    cell value;
}

def quote(struct test structarg) {
   output 34; // ASCII double quote
   output structarg.value;
   output 34;
}

def quote(cell arg) {
   output 39; // ASCII single quote
   output arg;
   output 39;
}

struct test A;
a.value = 'p';
quote<A>;
cell N = 'g';
quote<N>;
N += 3;
quote<N>;

//OUTPUT
// "p"'g''j'
```