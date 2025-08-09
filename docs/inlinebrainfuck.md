### In-line Brainfuck

In-line Brainfuck allows the programmer to define custom behaviour as if writing raw Brainfuck, much in the same way as C has in-line assembly syntax.

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

It is the programmer's responsibility to clear used cells and return back to the cell in which they started the in-line Brainfuck context. If the programmer does not do this, any mastermind code after the in-line Brainfuck command will likely break.

#### Memory location specifiers

For hand-tuning optimisations and in-line Brainfuck that reads from Mastermind variables, you can specify the location on the Brainfuck tape:

```
cell var @3 = 4;
// compiled: >>>++++

bf @4 {
   <><><>
}
// compiled: >>>><><><>
```

Alternatively if using the 2D grid you can use a comma seperated list with a second value:

```

bf @4,3 {
   <><><>
}
// compiled: >>>>^^^<><><>
```

#### Clobbering and Assertions

Mastermind will try to predict the value of cells at compile-time, so it can prevent unnecessary cell clean-ups and unreachable code (with optimisations turned on). If your in-line Brainfuck affects existing Mastermind variables, you should tell the compiler using the `clobbers` keyword, the syntax is similar to the `drain into` list:

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

#### Embedded Mastermind

You can embed high-level Mastermind code within a Brainfuck context, this allows you to control precisely what the generated Brainfuck code is doing, whilst also taking advantage of the syntax features of Mastermind.

```
cell sum @0;

bf @0 {
   >>
   // read input (until eof) to the tape, nullifying any spaces or newlines
   // (this is probably not a good practical example, ideas are appreciated)
   ,[
      {
         cell c @0;
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

#### Craziness

You can put in-line Brainfuck inside your embedded Mastermind.

```
bf {
   ++++[
      {
         cell i @0;
         assert i unknown;
         cell j @1 = i + 1;

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
