### In-line Brainfuck

In-line Brainfuck allows the programmer to define custom behaviour as if writing raw Brainfuck, much in the same way as C has in-line assembly syntax.

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

#### Memory location specifiers

The exact location to start an in-line Brainfuck context can be specified:

```
cell var @3 = 4;
// compiled: >>>++++

bf @4 {
   <><><>
}
// compiled: >>>><><><>
```

#### Clobbering and Assertions

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

#### Embedded Mastermind

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

This can be done recursively, for example:

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
