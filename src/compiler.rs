// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip};

use crate::parser::{Clause, Expression, Sign, VariableSpec};

// memory stuff is all WIP and some comments may be incorrect

// probably the meat and potatoes of this rewrite
pub fn compile(clauses: &[Clause], scopes: Vec<Scope>) -> Vec<Instruction> {
	let mut scope = Scope::new();
	let mut instructions = Vec::new();

	// hoist functions to top
	let clauses: Vec<Clause> = clauses
		.iter()
		.filter_map(|clause| {
			if let Clause::DefineFunction {
				name,
				arguments,
				block,
			} = clause
			{
				scope.functions_map.insert(
					name.clone(),
					Function {
						arguments: arguments.clone(),
						block: block.clone(),
					},
				);
				None
			} else {
				Some(clause.clone())
			}
		})
		.collect();

	for clause in clauses {
		match clause {
			Clause::AddToVariable { var, value } => {
				// get variable cell from allocation stack
				let mem: MemoryPointer = get_variable_mem(&scopes, var);
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
				// allocate a memory cell on the allocation stack
				// place the variable spec in the hashmap given the memory offset
				scope.allocate_variable(var);

				if let Some(expr) = initial_value {
					todo!();
				}
			}
			Clause::SetVariable { var, value } => todo!(),
			Clause::OutputByte { var } => {
				let mem: MemoryPointer = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OutputCell(mem));
			}
			Clause::WhileLoop { var, block } => {
				// open loop on variable
				let mem = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OpenLoop(mem));

				// recursively compile instructions
				{
					// TODO: when recursively compiling, check which things changed based on a return info value
					let mut scopes = scopes.clone();
					scopes.push(scope.clone());
					let loop_instructions = compile(&block, scopes);
					instructions.extend(loop_instructions);
				}
			}
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
			Clause::CallFunction {
				function_name,
				arguments,
			} => {
				// create variable translations and recursively compile the inner variable block
				let function_definition = scope.functions_map.get(&function_name).unwrap_or(
					scopes
						.iter()
						.rev()
						.find(|scope| scope.functions_map.contains_key(&function_name))
						.unwrap()
						.functions_map
						.get(&function_name)
						.unwrap(),
				);
				let mut scopes = scopes.clone();
				let mut cur_scope = scope.clone();

				cur_scope.translations_map.extend(
					zip(function_definition.arguments.iter(), arguments)
						.map(|(arg_def, arg)| (arg_def.name.clone(), arg.name)),
				);
				scopes.push(cur_scope);
				let loop_instructions = compile(&function_definition.block, scopes);
				instructions.extend(loop_instructions);
			}
			_ => (),
		}
	}

	// TODO: check if current scope has any leftover memory allocations

	instructions
}

// this is subject to change
#[derive(Debug)]
pub enum Instruction {
	AllocateCell,
	FreeCell(MemoryPointer), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(MemoryPointer), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop,
	AddToCell(MemoryPointer, i8),
	OutputCell(MemoryPointer),
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
fn get_variable_mem(scopes: &[Scope], var: VariableSpec) -> MemoryPointer {
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

// cloning is probably very inefficient but this is a compiler so eh
#[derive(Clone)]
pub struct Scope {
	// stack of memory/cells that have been allocated (peekable stack)
	allocation_stack: Vec<bool>,
	// mappings for variable names to places on above stack
	variable_map: HashMap<VariableSpec, MemoryPointer>,
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	translations_map: HashMap<String, String>,

	// functions accessible by any code within or in the current scope
	functions_map: HashMap<String, Function>,
}

#[derive(Clone)] // probably shouldn't be cloning here but whatever
struct Function {
	arguments: Vec<VariableSpec>,
	block: Vec<Clause>,
}
// represents a position in a stack relative to the head/top
type MemoryPointer = usize;

impl Scope {
	pub fn new() -> Scope {
		Scope {
			allocation_stack: Vec::new(),
			variable_map: HashMap::new(),
			translations_map: HashMap::new(),
			functions_map: HashMap::new(),
		}
	}

	fn allocate_variable(&mut self, var: VariableSpec) {
		for i in 0..var.arr_num.unwrap_or(0) {
			self.allocate_variable_cell(VariableSpec {
				name: var.name.clone(),
				arr_num: Some(i),
			});
		}
	}

	fn allocate_variable_cell(&mut self, var: VariableSpec) {
		let mem = self.allocate_memory_cell();
		self.variable_map.insert(var, mem);
	}

	fn allocate_memory_cell(&mut self) -> MemoryPointer {
		let mem = self.allocation_stack.len();
		self.allocation_stack.push(true);
		mem
	}

	fn free_variable_cell(&mut self, var: VariableSpec) {
		let mem = self.variable_map.remove(&var).unwrap();
		self.free_memory_cell(mem);
	}

	fn free_memory_cell(&mut self, mem: MemoryPointer) {
		self.allocation_stack[mem] = false;
	}
}
