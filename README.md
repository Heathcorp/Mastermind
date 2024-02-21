# Mastermind

A programming language designed to compile to Brainfuck.

Imagine if C was designed for computer architectures that run brainfuck, that is what Mastermind is intended to be.

### Note
In my implementation, the Brainfuck tape extends infinitely in both directions, with 8-bit wrapping cells.

## Usage

### Variables & Values

Variables can be defined as single cells, or as contiguous sets of cells:
```
// single-cell:
let var = 56;
let c = 'g';
let bool = true; // true/false equivalent to 1/0
// multi-cell:
let array[4] = [1, 2, 3, 4];
let string[5] = "hello";
```
If left uninitialised, a cell is assumed to have a value of 0.

Expressions consist of simple adds and subtractions, arrays and strings cannot be used in expressions:
```
// supported:
var += 4 + (5 - 4 + (3 - 2));
var = 'g' - 23 + true;
var = var + 5; // inefficient but supported
var += array[0] + 5;
let arr[3] = [4 + 3 - ('5' - 46), 1, 3];

// NOT supported:
var += [4, 5, 7][1] + 3;
```

##### Note: Array indices must be compile-time constant integers
This is a limitation of Brainfuck, getting around this problem requires more runtime code than I want to include for the sake of optimisations. If you want to implement runtime array index access, read the section on [in-line Brainfuck](#in-line-brainfuck-features).

### Input & Output

Single bytes can be output using the `output` operator:
```
output 'h';
output var;
output var + 6;
output array[0];
// the following are equivalent:
output '\n';
output 10;
```
Likewise for `input`:
```
input var;
input arr[2];
```
There is also `*` spread syntax and support for strings and arrays for ease of use:
```
output "hello";
output ['1', '2', '3'];
output *array;

input *array;
```

### Loops

The simplest is the `while` loop, which only supports cell references, not expressions:
```
while var {
   // do stuff
   // var -= 1;
   // etc
}
```

#### Draining loops
Next there is a very common Brainfuck pattern which I call the `drain` loop:
```
drain var {

}
// shorthand for following:
while var {
   // do stuff
   var -= 1;
}
```
This destructively loops as many times as the value in the cell being referenced, this can be used with expressions:
```
drain 10 {}
drain var - 6 {}
```
There is also shorthand syntax for adding to other cells:
```
drain var into other_var other_var_2 *spread_array etc;

// example of typical "for loop":
let i;
drain 10 into i {
   output '0' + i; // inefficient for the example
}
// "0123456789"
// equivalent to the following:
let i;
let N = 10;
while N {
   output '0' + i;
}
```

#### Copying loops
Sometimes you want to loop a variable number of times but do not want to destruct the variables value, this is what the `copy` loop is for:
```
copy var into other_var *spread_var etc;

// examples:
copy var {
   // this will output the original var value, var times
   output var;
}

let rows = 3;
let columns = 6;
let total;
drain rows {
   copy columns into total {
      output '.';
   }
}
// ......
// ......
// ......
```

### If/Else
Note: These are currently a little bit inefficient but work for most use cases.

If/Else statements are very simple, they check if a value is positive. If you want to invert then you can use the `not` keyword.
Examples:
```
if 13 {
   output "13";
}

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

### Functions
Currently functions work more like templates/macros, as they do not perform any passing by value. All functions are essentially inlined at compile time. This means multiple calls to a large function will significantly increase your compiled code size.

For this reason, function arguments are given using `<` angled bracket `>` syntax, much like generic functions in other languages:
```
def quote<arg> {
   output 39; // ASCII single quote
   output arg;
   output 39;
}

let N = 'g';
quote<N>;
N += 3;
quote<N>;
// gj
```

Functions have a quirk with scoping variables: currently code inside functions can access variables defined in the calling scope, I am not sure if this is erroneous or should be supported, advice is appreciated. 

### Imports

Imports work much like the C preprocessor:
```
#include "other_file"
```
This copies the contents of "other_file" into the current file.

### In-line Brainfuck features
In-line Brainfuck allows the programmer to define custom behaviour as if they were writing raw Brainfuck, much in the same way as C has in-line assembly syntax.

```
// This is its most basic form:
// find the next cell that equals -1
bf {
   +[->+]-
}

// This is its more advanced form:
// input a line of lowercase letters and output the uppercase version
// this is an intentionally inefficient example
bf @3 clobbers var *spread_var etc {
   ,----------[++++++++++>,----------]
   <[<]>
   [
      {
         let g @0;
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
It is the programmer's responsibility to clear used cells and return back to the cell in which they started the in-line Brainfuck context. If the programmer does not do this, any mastermind code after the in-line Brainfuck command will likely break.

#### Memory location specifiers
For hand-tuning optimisations and in-line Brainfuck that reads from Mastermind variables, you can specify the location on the Brainfuck tape:
```
let var @3 = 4;
// compiled: >>>++++

bf @4 {
   <><><>
}
// compiled: >>>><><><>
```

#### Clobbering and Assertions
Mastermind will try to predict the value of cells at compile-time, so it can prevent unnecessary cell clean-ups and unreachable code (with optimisations turned on). If your in-line Brainfuck affects existing Mastermind variables, you should tell the compiler using the `clobbers` keyword, the syntax is similar to the `drain into` list:
```
bf clobbers var *spread_var other_var etc {}
```
The compiler will now assume nothing about the values of those variables afterwards.

If instead you want to tell the compiler specifically that a value has become a certain value, you can use `assert`:
```
assert var equals 3;
// most common use cases:
assert var equals 0;
assert var unknown;
```
Asserting a variable as `unknown` is equivalent to clobbering

#### Embedded Mastermind
Embedding Mastermind into your in-line Brainfuck allows you to use Mastermind syntax features for programs within your Brainfuck, this is useful for N-length string based programs, or anything not currently possible in pure Mastermind:

```
let sum @0;

bf @0 {
   >>
   // read input (until eof) to the tape, nullifying any spaces or newlines
   // (this is probably not a good practical example, ideas are appreciated)
   ,[
      {
         let c @0;
         assert c unknown; // needed otherwise the compiler assumes c = 0

         if not (c - '\n') {
            c = 0;
         }
         if not (c - ' ') {
            c = 0;
         }
      }
      >,
   ]
}
```
Memory location specifiers are relative to the current Mastermind context. Also, top-level variables are not cleared by default in Mastermind contexts, this allows you to "leave" variables in cells for your Brainfuck to use. If you want your embedded Mastermind to clean itself up, you can simply open a scope at the top level:
```
bf {
   ++----++[][][<><><>] // the program doesn't matter for this example
   {
      // variables here will not be cleared
      let g @2;
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

#### Craziness
You can put in-line Brainfuck inside your embedded Mastermind.
```
bf {
   ++++[
      {
         let i @0;
         assert i unknown;
         let j @1 = i + 1;

         bf @1 {
            [.+]
            {
               // even more layers are possible
               bf {
                  {
                     output "h"
                  }
               }
            }
         }
      }
   -]
}
```