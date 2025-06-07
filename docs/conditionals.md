### Conditionals

Mastermind supports basic `if`/`else` statements. An `if` statement takes in a single cell expression, if the expression is evaluated to be truthy, then the `if` block is executed, otherwise the optional `else` block is executed. This behaviour can be inverted using the `not` keyword.

```
if 13 {
  output "13";
}

if not true {
  // unreachable
}

cell var = 4;
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
