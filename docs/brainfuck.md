### WIP

In this implementation, each cell is an 8-bit cell that wraps if an increment or decrement overflows or underflows. Example:

If the current tape cell value is `255`, and the program increments (`+`), then the cell becomes `0`. Similarly, `0` decremented with `-` becomes `255`.
