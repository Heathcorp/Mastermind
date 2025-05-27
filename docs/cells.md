Cells are 8 bit integer which are the only native data type to the Mastermind language. Cells can be singular memory cells or can be 
an array in the format of a contiguous block and when created without initialisation cells will always initialise to 0. 

Since the input and output use ASCII cells are converted to and from ASCCI when outputting and inputting.

```
// single-cell:
let var = 56;
let c = 'g';
let bool = true; // true/false equivalent to 1/0
// multi-cell:
let array[4] = [1, 2, 3, 4];
let string[5] = "hello";
```

multi cell structures can also be created but this is covered in the 'Structs' page of the documentation