By default Brainfuck includes only single byte input and output function for handling IO. While single byte is only supported 
for input, output has been expanded to include support for arrays and spread operators.

`output` - will output the value of the cell to the right or the array/spread
`input` - will update the cell to the right to be the next byte in the Mastermind input

## Examples

```
cell a;
input a;
output '0' + a;
```
The above code will output any integer inputted so input 7 output 7


```
array = "ABCD"
output "hello";
output ['1', '2', '3'];
output *array;
```
The above code will output 'hello123ABCD'