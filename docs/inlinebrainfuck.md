In-line Brainfuck allows the programmer to define custom behaviour as if they were writing raw Brainfuck, much in the same 
way as C has in-line assembly syntax. 

The purpose of this inline Brainfuck system is to allow the programmer greater control over their program. Meaning functionality can be 
tuned to be smaller or a programmer can use custom functionality that may not be currently supported by the Mastermind language

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

Since the compiler cannot understand the Brainfuck being written it is the programmer's responsibility to clear used cells and return back to the cell in which they started the in-line Brainfuck context.
If the programmer does not do this, any mastermind code after the in-line Brainfuck command will likely break.