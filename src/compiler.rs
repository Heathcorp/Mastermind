// compile syntax tree into low-level instructions

use std::collections::HashMap;

use crate::parser::{Clause, Expression, Sign, VariableSpec};

pub struct Scope {
	// stack of memory/cells that have been allocated (peekable stack)
	allocation_stack: Vec<bool>,
	// mappings for variable names to places on above stack
	variable_map: HashMap<VariableSpec, StackOffset>,
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	translations_map: HashMap<String, String>,
}
// represents a position in a stack relative to the head/top
type StackOffset = usize;

impl Scope {
	pub fn new() -> Scope {
		Scope {
			allocation_stack: Vec::new(),
			variable_map: HashMap::new(),
			translations_map: HashMap::new(),
		}
	}
}

// probably the meat and potatoes of this rewrite
pub fn compile(clauses: &[Clause], scopes: Vec<Scope>) -> Vec<Instruction> {
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
			Clause::AddToVariable { var, value } => {
				// get variable cell from allocation stack
				let mem: StackOffset = get_variable_mem(&scopes, var);
				// flatten expression to a list of additions or subtractions
				let (imm, adds, subs) = flatten_expression(value);
				// add the immediate numbers
				instructions.push(Instruction::AddToCell(mem, imm));

				// and the variables
				if (adds.len() + subs.len()) > 0 {
					todo!();
				}
			}
			Clause::DefineVariable { var, initial_value } => {
				// allocate a memory on the allocation stack
				// place the variable spec in the hashmap given the memory offset
			}
			Clause::SetVariable { var, value } => todo!(),
			Clause::OutputByte { var } => {
				let mem: StackOffset = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OutputCell(mem));
			}
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
	FreeCell(StackOffset), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(StackOffset), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop,
	AddToCell(StackOffset, i8),
	OutputCell(StackOffset),
	// TODO: contiguous cells for quick iterations?
	// AllocateContiguousCells(usize),
	// FreeContiguousCells(usize), // number indicates
}

// not sure if this is the compiler's concern or if it should be the parser
// (constant to add, variables to add, variables to subtract)
// currently multiplication is not supported so order of operations and flattening is very trivial
// If we add multiplication in future it will likely be constant multiplication only, so no variable on variable multiplication
fn flatten_expression(expr: Expression) -> (i8, Vec<VariableSpec>, Vec<VariableSpec>) {
	let mut imm_sum = 0i8;
	let mut additions = Vec::new();
	let mut subtractions = Vec::new();

	match expr {
		Expression::SumExpression { sign, summands } => {
			let flattened = summands
				.into_iter()
				.map(flatten_expression)
				.reduce(|acc, (imm, adds, subs)| {
					(acc.0 + imm, [acc.1, adds].concat(), [acc.2, subs].concat())
				})
				.unwrap_or((0, vec![], vec![]));

			match sign {
				Sign::Positive => {
					imm_sum += flattened.0;
					additions.extend(flattened.1);
					subtractions.extend(flattened.2);
				}
				Sign::Negative => {
					imm_sum -= flattened.0;
					subtractions.extend(flattened.1);
					additions.extend(flattened.2);
				}
			};
		}
		Expression::NaturalNumber(number) => {
			imm_sum = number.clone().try_into().unwrap();
		}
		Expression::VariableReference(var) => {
			additions.push(var);
		}
	}

	(imm_sum, additions, subtractions)
}

// not sure if should be an impl of Scopes type
// takes in the current compiler scope and a variable/arr reference
// returns the allocation stack offset for that variable
fn get_variable_mem(scopes: &[Scope], var: VariableSpec) -> StackOffset {
	if let Some(scope) = scopes.last() {
		if let Some(mem) = scope.variable_map.get(&var) {
			// this scope has the variable allocated
			// offset this based on the total of all prior stacks, TODO: maybe redesign this? idk
			// not sure about this, it might work out
			// TODO: fix this, this is currently broken because it doesn't properly take into account memory frees and booleans and stuff because the map doesn't change every time a cell is freed but the stack does, hmm
			return scopes[..(scopes.len() - 1)]
				.iter()
				.fold(mem.clone(), |sum, s| {
					sum + s.allocation_stack.iter().filter(|b| **b).count()
				});
		} else if let Some(alias) = scope.translations_map.get(&var.name) {
			// need to translate to an alias and call recursively, could do this iterately as well but whatever (tail recursion)
			return get_variable_mem(
				&scopes[..(scopes.len() - 1)],
				VariableSpec {
					name: alias.clone(),
					arr_num: var.arr_num,
				},
			);
		}
	}

	panic!("Variable not found: {var:#?}");
}
