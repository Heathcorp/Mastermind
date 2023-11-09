// compile syntax tree into low-level instructions

use std::collections::HashMap;

use crate::parser::{Clause, VariableSpec};

// probably the meat and potatoes of this rewrite
// when we recurse with this function, we should clone the clauses we're compiling and extend it with the function definition clauses
pub fn compile(clauses: &[Clause]) -> Vec<Instruction> {
	// hoist functions to top
	let scope_functions: Vec<Clause> = clauses
		.iter()
		.filter(|clause| {
			if let Clause::DefineFunction {
				name,
				arguments,
				block,
			} = clause
			{
				true
			} else {
				false
			}
		})
		.map(|c| *c.clone())
		.collect();
}

// this is subject to change
#[derive(Debug)]
pub enum Instruction {
	AllocateCell,
	FreeCell(usize), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(usize), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop,
	AddToCell(usize, i8),
	OutputCell(usize),
}
