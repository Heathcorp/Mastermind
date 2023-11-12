// compile syntax tree into low-level instructions

use std::collections::HashMap;

use crate::parser::{Clause, VariableSpec};

// probably the meat and potatoes of this rewrite
// when we recurse with this function, we should clone the clauses we're compiling and extend it with the function definition clauses
pub fn compile(clauses: &[Clause]) -> Vec<Instruction> {
	let mut instructions = Vec::new();

	// hoist functions to top
	let mut scope_functions: Vec<Clause> = Vec::new();
	let mut scope_clauses: Vec<Clause> = Vec::new();

	clauses.iter().for_each(|clause| {
		if let Clause::DefineFunction {
			name,
			arguments,
			block,
		} = clause
		{
			scope_functions.push(clause.clone());
		} else {
			scope_clauses.push(clause.clone());
		}
	});

	for clause in scope_clauses {
		match clause {
			Clause::OutputByte { var } => todo!(),
			Clause::DefineVariable { var, initial_value } => {}
			Clause::AddToVariable { var, value } => todo!(),
			Clause::SetVariable { var, value } => todo!(),
			Clause::WhileLoop { var, block } => todo!(),
			Clause::CopyLoop {
				source,
				targets,
				block,
				is_draining,
			} => todo!(),
			Clause::IfStatement {
				var,
				if_block,
				else_block,
			} => todo!(),
			Clause::DefineFunction {
				name,
				arguments,
				block,
			} => todo!(),
			Clause::CallFunction {
				function_name,
				arguments,
			} => todo!(),
		}
	}

	instructions
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
