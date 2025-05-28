Conditional Statements in Mastermind currently only exist in the format of an If/Else block which evaluates to True if a value is non 0 or false if it is 0.
Currently not supported is the usage of logical operators such as && and || to link multiple statements together 


### If/Else

Note: If/Else blocks currently a little bit inefficient but work for most use cases.

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