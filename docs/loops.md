Mastermind currently supports three forms of loops: `while`, `drain` and `copy`.

It should be noted that there is no early breaking in any of these forms, so all clauses in a loop body are always executed in each iteration.

## While

The `while` loop operates similarly to other languages, accepting a condition expression, and a loop body.

The clauses inside the loop body are executed until the condition is falsy (i.e. equal to `0`). The condition is checked before each iteration.

Note: currently `while` conditions must be direct variable references, this is subject to future compiler updates.

```
cell n = 5;
while n {
  // do stuff
  n -= 1;
}
// n is now equal to 0
```

## Drain

The `drain` loop mirrors a very common pattern found in Brainfuck programs: decrementing a cell. `drain` accepts an expression, a list of variables to 'drain into', and/or a loop body.

If the expression is a direct variable reference, then the variable is decremented after each iteration. If not, it is evaluated in a temporary cell, then decremented after each iteration.

```
drain var {
  // do stuff
}

// equivalent to:
while var {
  // do stuff
  var -= 1;
}
```

With expressions:

```
drain 6 {
  output 'a';
}
// aaaaaa
```

```
cell x = 7;
drain x - 2 {
  output 'b';
}
// bbbbb
```

In the above example, `x` is left unchanged.

### Into

If the `into` keyword is used, followed by a whitespace-separated list of target variables, the targets will be incremented after each iteration.

```
cell i;
drain 10 into i {
  output '0' + i;
}
// 0123456789

// equivalent to:
cell i;
cell ten = 10;
while ten {
  output '0' + i;

  i += 1;
  ten -= 1;
}
```

```
drain var into other_var other_var_2 *spread_array;

// equivalent to:
drain var {
  other_var += 1;
  other_var_2 += 1;
  spread_array[0] += 1;
  spread_array[1] += 1;
  spread_array[2] += 1;
  // ...
}
```

## Copy

The `copy` loop acts similarly to the `drain` loop, however the expression must be a direct variable reference, and it is left unchanged afterwards, and its original value is accessible within the loop body.

```
cell var = 5;
copy var {
  output '0' + var;
}
// 55555

// equivalent to:
cell var = 5;
cell temp = var;
while temp {
  output '0' + var;

  temp -= 1;
}
```

```
cell y;
copy x into y {

};
```
