Looping in Mastermind has 3 main forms. These are the:
- While Loop
- Drain Loop
- Copy Loop

all 3 looping styles are essentially variations of a while loop

## While Loop

The simplest is the while loop, which only supports cell references, not expressions:
```
while var {
    //do stuff
    var -= 1;
    //etc
}
```
This loop does require the programmer to ensure that the conditional variable is changing otherwise it will loop infinitely

## Drain Loop

The Drain loop is a form of syntax sugar for a self decrementing while loop. This form of loop is extremely common in Brainfuck 
so it has been shortened with this syntax

```
drain var {

}
```
shorthand for following:
```
while var {
// do stuff
var -= 1;
}
```
This destructively loops as many times as the value in the cell being referenced, this can be used with expressions:

drain 10 {}

drain var - 6 {}


Drain additionally supports the ability to destruct a variable into multiple other variables

``
drain var into other_var other_var_2 *spread_array etc;
``


```
// example of typical "for loop":
cell i;
drain 10 into i {
    output '0' + i; // inefficient for the example
}
// "0123456789"
// equivalent to the following:
cell i = 0;
cell N = 10;
while N {
    output '0' + i;
    i += 1;
    N -= 1;
}
```

## Copy Loop

The copy loop is similar to a Drain loop however it is designed to preserve the initial state of the loop variable.
A copy loop is shorthand designed to replace the usage of a temporary variable in a drain loop.
```
copy var {
    //Do Stuff
}
```
is equivalent to:
```
cell temp = var;
while temp {
    // do stuff
    temp -= 1;
}
```

Similar to a Drain loop this can also be used to transfer state into another variable

``
copy var into other_var other_var_2 *spread_array etc;
``

