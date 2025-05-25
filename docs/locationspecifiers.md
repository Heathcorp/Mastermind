Location specifiers are an optimisation tool available to the developer to hand tune the memory allocation of 
variables in your Mastermind code. Location specifiers will override the compilers memory allocation algorithm. 

### Usage
Location specifiers can be added in the form of a memory address given during variable definition or before inline Brainfuck statements.


For 1D Brainfuck you simply provide the x-coordinate where memory should be allocated
```
cell var @3 = 4;
// compiled: >>>++++

bf @4 {
   <><><>
}
// compiled: >>>><><><>
```


Alternatively if using the 2D enhancements you can use a comma seperated list with an x-coordinate and y-coordinate for a 2D location
```

bf @4,3 {
   <><><>
}
// compiled: >>>>^^^<><><>
```

### Conflicts
For location specifiers the responsibility is on the programmer to ensure that any memory overlaps are avoided. This can be especially common
when specifying the location of a static array that might use multiple locations. However,
the compiler can pick up on these errors and will not compile if it notices conflicting memory an example of code that does this and an error is below:

```
cell a @1 = 1;
cell foo @1 = 2;
cell b = 3;
```
```
Location specifier @1,0 conflicts with another allocation
```