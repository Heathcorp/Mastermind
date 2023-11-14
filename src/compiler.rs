// TODO: refactor, lot's of clones, not great on memory allocations and not very rusty

// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip, num::Wrapping};

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
				// so many clones
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
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
				scope.allocate_variable(var.clone());

				if let Some(expr) = &initial_value {
					match (&var.arr_num, expr) {
						(
							None,
							Expression::NaturalNumber(_)
							| Expression::VariableReference(_)
							| Expression::SumExpression {
								sign: _,
								summands: _,
							},
						) => {
							// TODO: make a function for this, common pattern

							let mut scopes = scopes.clone();
							scopes.push(scope.clone());
							let mem = get_variable_mem(&scopes, var);

							let (imm, adds, subs) = flatten_expression(expr.clone());
							instructions.push(Instruction::AddToCell(mem, imm));
							if (adds.len() + subs.len()) > 0 {
								todo!();
							}
						}
						(Some(len), Expression::ArrayLiteral(expr_arr)) => {
							// again need to figure out a good way of doing this
							let mut scopes = scopes.clone();
							scopes.push(scope.clone());

							assert_eq!(*len, expr_arr.len(), "Array literal should be same length as variable: {var:#?} = {initial_value:#?}");

							for (i, expr) in expr_arr.iter().enumerate() {
								let mem = get_variable_mem(
									&scopes,
									VariableSpec {
										name: var.name.clone(),
										arr_num: Some(i),
									},
								);

								let (imm, adds, subs) = flatten_expression(expr.clone());
								instructions.push(Instruction::AddToCell(mem, imm));
								if (adds.len() + subs.len()) > 0 {
									todo!();
								}
							}
						}
						(Some(len), Expression::StringLiteral(str)) => {}
						_ => {
							panic!(
								"Invalid initial value for variable: {var:#?} = {initial_value:#?}"
							);
						}
					}
				}
			}
			Clause::SetVariable { var, value } => todo!(),
			Clause::OutputByte { var } => {
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
				let mem: MemoryPointer = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OutputCell(mem));
			}
			Clause::WhileLoop { var, block } => {
				// open loop on variable
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
				let mem = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OpenLoop(mem));

				// recursively compile instructions
				{
					// TODO: when recursively compiling, check which things changed based on a return info value
					// TODO: make a function or something for this common pattern
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
	AddToCell(MemoryPointer, u8),
	OutputCell(MemoryPointer),
	// TODO: contiguous cells for quick iterations?
	// AllocateContiguousCells(usize),
	// FreeContiguousCells(usize), // number indicates
}

// not sure if this is the compiler's concern or if it should be the parser
// (constant to add, variables to add, variables to subtract)
// currently multiplication is not supported so order of operations and flattening is very trivial
// If we add multiplication in future it will likely be constant multiplication only, so no variable on variable multiplication
fn flatten_expression(expr: Expression) -> (u8, Vec<VariableSpec>, Vec<VariableSpec>) {
	let mut imm_sum = Wrapping(0u8);
	let mut additions = Vec::new();
	let mut subtractions = Vec::new();

	match expr {
		Expression::SumExpression { sign, summands } => {
			let flattened = summands
				.into_iter()
				.map(flatten_expression)
				.reduce(|acc, (imm, adds, subs)| {
					(
						(Wrapping(acc.0) + Wrapping(imm)).0,
						[acc.1, adds].concat(),
						[acc.2, subs].concat(),
					)
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
			let number: u8 = number.try_into().unwrap();
			imm_sum = Wrapping(number);
		}
		Expression::VariableReference(var) => {
			additions.push(var);
		}
		Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => {
			panic!("Unable to flatten arrays or string expressions: {expr:#?}");
		}
	}

	(imm_sum.0, additions, subtractions)
}

// TOOD: Vec<Scope> impl?
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
				&scopes,
				VariableSpec {
					name: alias.clone(),
					arr_num: var.arr_num,
				},
			);
		} else {
			return get_variable_mem(&scopes[0..(scopes.len() - 1)], var);
		}
	}

	panic!("Variable not found: {var:#?}");
}

// cloning is probably very inefficient but this is a compiler so eh
#[derive(Clone)]
pub struct Scope {
	// stack of memory/cells that have been allocated (peekable stack)
	allocation_stack: Vec<bool>,
	// array length of each variable (not sure if this is needed, probably not the best way of doing it either)
	variable_sizes: HashMap<String, Option<usize>>,
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
			variable_sizes: HashMap::new(),
			variable_map: HashMap::new(),
			translations_map: HashMap::new(),
			functions_map: HashMap::new(),
		}
	}

	fn allocate_variable(&mut self, var: VariableSpec) {
		if let Some(len) = &var.arr_num {
			for i in 0..*len {
				self.allocate_variable_cell(VariableSpec {
					name: var.name.clone(),
					arr_num: Some(i),
				});
			}
		} else {
			self.allocate_variable_cell(VariableSpec {
				name: var.name.clone(),
				arr_num: None,
			});
		}
		self.variable_sizes.insert(var.name, var.arr_num);
	}

	// not sure if this is even needed tbh, (TODO: refactor?)
	fn allocate_variable_cell(&mut self, var: VariableSpec) {
		let mem = self.allocate_memory_cell();
		self.variable_map.insert(var, mem);
	}

	fn allocate_memory_cell(&mut self) -> MemoryPointer {
		let mem = self.allocation_stack.len();
		self.allocation_stack.push(true);
		mem
	}

	fn free_variable(&mut self, var: VariableSpec) {
		// get the stored variable size from the sizes map
		let len = self.variable_sizes.remove(&var.name).unwrap();

		if var.arr_num.is_some() {
			panic!("Must free entire multi-byte variable, not just one cell: {var:#?}");
			// Why not? TODO
		};

		if let Some(len) = len {
			for i in 0..len {
				self.free_variable_cell(VariableSpec {
					name: var.name.clone(),
					arr_num: Some(i),
				});
			}
		} else {
			self.free_variable_cell(VariableSpec {
				name: var.name.clone(),
				arr_num: None,
			});
		}
		self.variable_sizes.insert(var.name, var.arr_num);
	}

	// should never be called by the compiler, unless we change things
	// TODO: refactor, is this really needed?
	fn free_variable_cell(&mut self, var: VariableSpec) {
		let mem = self.variable_map.remove(&var).unwrap();
		self.free_memory_cell(mem);
	}

	fn free_memory_cell(&mut self, mem: MemoryPointer) {
		self.allocation_stack[mem] = false;
	}
}
