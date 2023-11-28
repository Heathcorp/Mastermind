// TODO: refactor, lot's of clones, not great on memory allocations and not very rusty

// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip, num::Wrapping};

use crate::parser::{Clause, Expression, Sign, VariableSpec};

// memory stuff is all WIP and some comments may be incorrect

// probably the meat and potatoes of this rewrite
// need to rethink how this recursion works, write down and go back to first functional principles
// TODO: here is the answer, redesign scope variable to be a singly-linked list, high priority for refactor
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
				let mem: usize = get_variable_mem(&scopes, var);

				instructions.push(Instruction::AddToCell(mem, value));
			}
			Clause::DeclareVariable(var) => {
				// allocate a memory cell on the allocation stack
				// place the variable spec in the hashmap given the memory offset
				scope.allocate_variable(var.clone());

				// create instructions to allocate cells
				// TODO: obviously needs refactoring when we refactor the scopes stuff
				{
					let mut scopes = scopes.clone();
					scopes.push(scope.clone());

					if let Some(len) = var.arr_num {
						for i in 0..len {
							let mem = get_variable_mem(
								&scopes,
								VariableSpec {
									name: var.name.clone(),
									arr_num: Some(i),
								},
							);
							instructions.push(Instruction::AllocateCell(mem));
						}
					} else {
						let mem = get_variable_mem(&scopes, var.clone());
						instructions.push(Instruction::AllocateCell(mem));
					}
				}
			}
			Clause::ClearVariable(var) => {
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
				let mem: usize = get_variable_mem(&scopes, var);

				instructions.push(Instruction::ClearCell(mem));
			}
			Clause::OutputByte { value } => match value {
				Expression::VariableReference(var) => {
					let mut scopes = scopes.clone();
					scopes.push(scope.clone());
					let mem: usize = get_variable_mem(&scopes, var);
					instructions.push(Instruction::OutputCell(mem));
				}
				Expression::NaturalNumber(num) => {
					let val: u8 = (num % 256).try_into().unwrap();
					// allocate a temporary cell for the byte being output
					let mem = scope.allocate_unnamed_mem(&scopes);
					instructions.push(Instruction::AllocateCell(mem));
					instructions.push(Instruction::AddToCell(mem, val));
					instructions.push(Instruction::OutputCell(mem));
					instructions.push(Instruction::ClearCell(mem));
					instructions.push(Instruction::FreeCell(mem));
				}
				Expression::SumExpression { sign, summands } => todo!(),
				Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => todo!(),
			},
			Clause::WhileLoop { var, block } => {
				// open loop on variable
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
				let mem = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OpenLoop(mem));

				// recursively compile instructions
				// TODO: when recursively compiling, check which things changed based on a return info value
				// TODO: make a function or something for this common pattern
				let loop_instructions = compile(&block, scopes);
				instructions.extend(loop_instructions);

				// close the loop
				instructions.push(Instruction::CloseLoop);
			}
			Clause::CopyVariable {
				target,
				source,
				constant,
			} => {
				// allocate a temporary cell
				let temp_mem = scope.allocate_unnamed_mem(&scopes);
				instructions.push(Instruction::AllocateCell(temp_mem));
				// again this stuff needs to be fixed, this comment has probably been copied as much as this code has
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());

				let source_mem = get_variable_mem(&scopes, source);
				let target_mem = get_variable_mem(&scopes, target);

				// copy source to target and temp
				instructions.push(Instruction::OpenLoop(source_mem));
				instructions.push(Instruction::AddToCell(target_mem, constant as u8));
				instructions.push(Instruction::AddToCell(temp_mem, constant as u8));
				instructions.push(Instruction::AddToCell(source_mem, -1i8 as u8));
				instructions.push(Instruction::CloseLoop);
				// copy back from temp
				instructions.push(Instruction::OpenLoop(temp_mem));
				instructions.push(Instruction::AddToCell(source_mem, 1));
				instructions.push(Instruction::AddToCell(temp_mem, -1i8 as u8));
				instructions.push(Instruction::CloseLoop);
				instructions.push(Instruction::FreeCell(temp_mem));
			}
			Clause::CopyLoop {
				source,
				targets,
				block,
				is_draining,
			} => match is_draining {
				true => {
					// again this stuff needs to be fixed
					let mut scopes = scopes.clone();
					scopes.push(scope.clone());

					let source_mem = get_variable_mem(&scopes, source);

					instructions.push(Instruction::OpenLoop(source_mem));

					// recurse
					let loop_instructions = compile(&block, scopes.clone());
					instructions.extend(loop_instructions);

					// copy into each target and decrement the source
					for target in targets {
						let mem = get_variable_mem(&scopes, target);
						instructions.push(Instruction::AddToCell(mem, 1));
					}
					instructions.push(Instruction::AddToCell(source_mem, -1i8 as u8)); // 255
					instructions.push(Instruction::CloseLoop);
				}
				false => {
					// allocate a temporary cell
					let temp_mem = scope.allocate_unnamed_mem(&scopes);
					instructions.push(Instruction::AllocateCell(temp_mem));

					// again this stuff needs to be fixed
					let mut scopes = scopes.clone();
					scopes.push(scope.clone());

					let source_mem = get_variable_mem(&scopes, source);

					instructions.push(Instruction::OpenLoop(source_mem));

					// recurse
					let loop_instructions = compile(&block, scopes.clone());
					instructions.extend(loop_instructions);

					// copy into each target and decrement the source
					for target in targets {
						let mem = get_variable_mem(&scopes, target);
						instructions.push(Instruction::AddToCell(mem, 1));
					}
					instructions.push(Instruction::AddToCell(temp_mem, 1));
					instructions.push(Instruction::AddToCell(source_mem, -1i8 as u8)); // 255
					instructions.push(Instruction::CloseLoop);

					// copy back the temp cell
					instructions.push(Instruction::OpenLoop(temp_mem));
					instructions.push(Instruction::AddToCell(temp_mem, -1i8 as u8));
					instructions.push(Instruction::AddToCell(source_mem, 1));
					instructions.push(Instruction::CloseLoop);

					instructions.push(Instruction::FreeCell(temp_mem));
				}
			},
			Clause::IfStatement {
				var,
				if_block,
				else_block,
			} => {
				if if_block.is_none() && else_block.is_none() {
					panic!("Expected block in if/else statement");
				};

				let mut scopes = scopes.clone();
				scopes.push(scope.clone());
				let mut new_scope = Scope::new();

				let temp_var_mem = new_scope.allocate_unnamed_mem(&scopes);
				instructions.push(Instruction::AllocateCell(temp_var_mem));
				let else_condition_mem = match else_block {
					Some(_) => {
						let else_mem = new_scope.allocate_unnamed_mem(&scopes);
						instructions.push(Instruction::AllocateCell(else_mem));
						instructions.push(Instruction::AddToCell(else_mem, 1));
						Some(else_mem)
					}
					None => None,
				};

				let original_var_mem = get_variable_mem(&scopes, var.clone());

				instructions.push(Instruction::OpenLoop(original_var_mem));

				// move the variable to the temporary cell
				instructions.push(Instruction::OpenLoop(original_var_mem));
				instructions.push(Instruction::AddToCell(temp_var_mem, 1));
				instructions.push(Instruction::AddToCell(original_var_mem, -1i8 as u8));
				instructions.push(Instruction::CloseLoop);

				// change scope var pointer
				// disgusting, to fix a bug with massive memory jumps, need a more mature scope impl to do this for us
				new_scope.variable_map.insert(
					var.clone(),
					temp_var_mem - scopes.iter().fold(0, |a, s| a + s.allocations),
				);
				// free the original cell temporarily as it isn't being used
				instructions.push(Instruction::FreeCell(original_var_mem));

				// set the else condition cell
				if let Some(mem) = else_condition_mem {
					instructions.push(Instruction::AddToCell(mem, -1i8 as u8));
				}

				// recursively compile if block
				if let Some(block) = if_block {
					// disgusting code, seriously needs refactor
					let mut scopes = scopes.clone();
					scopes.push(new_scope.clone());
					instructions.extend(compile(&block, scopes));
				};

				// reallocate the temporarily freed variable cell
				instructions.push(Instruction::AllocateCell(original_var_mem));
				// move the temporary cell contents back to the variable cell
				instructions.push(Instruction::OpenLoop(temp_var_mem));
				instructions.push(Instruction::AddToCell(original_var_mem, 1));
				instructions.push(Instruction::AddToCell(temp_var_mem, -1i8 as u8));
				instructions.push(Instruction::CloseLoop);
				new_scope.variable_map.remove(&var);

				instructions.push(Instruction::FreeCell(temp_var_mem));

				// close if block
				instructions.push(Instruction::CloseLoop);

				// else block:
				if let Some(else_mem) = else_condition_mem {
					instructions.push(Instruction::OpenLoop(else_mem));
					instructions.push(Instruction::AddToCell(else_mem, -1i8 as u8));

					// recursively compile else block
					let block = else_block.unwrap();
					instructions.extend(compile(&block, scopes));

					instructions.push(Instruction::CloseLoop);
					instructions.push(Instruction::FreeCell(else_mem));
				}
			}
			Clause::CallFunction {
				function_name,
				arguments,
			} => {
				// create variable translations and recursively compile the inner variable block
				// TODO: make this its own function
				// also heavily memory inefficient because cloning scope with function definitions

				// prepare scopes to recurse with, put current scope as well as argument translations
				let mut scopes = scopes.clone();
				scopes.push(scope.clone());

				// horribly inefficient, please refactor at some point
				let function_definition = scopes
					.iter()
					.rev()
					.find(|scope| scope.functions_map.contains_key(&function_name))
					.expect(&format!(
						"No function with name \"{}\" found",
						function_name
					))
					.functions_map
					.get(&function_name)
					.unwrap()
					.clone();

				let mut new_scope = Scope::new();
				new_scope.translations_map.extend(zip(
					function_definition.arguments.clone().into_iter(),
					arguments,
				));
				scopes.push(new_scope);

				// recurse
				let loop_instructions = compile(&function_definition.block, scopes);
				instructions.extend(loop_instructions);
			}
			_ => (),
		}
	}

	// TODO: check if current scope has any leftover memory allocations and free them
	// TODO: check known values and clear them selectively
	// create instructions to allocate cells
	// TODO: obviously needs refactoring when we refactor the scopes stuff
	// This code sucks, duplicated from definevariable
	// all we need is a recursive scope structure but not till later because this has dragged on too long anyway
	for (var_name, arr_len) in &scope.variable_sizes {
		let mut scopes = scopes.clone();
		scopes.push(scope.clone());
		if let Some(len) = arr_len {
			for i in 0..*len {
				let mem = get_variable_mem(
					&scopes,
					VariableSpec {
						name: var_name.clone(),
						arr_num: Some(i),
					},
				);
				instructions.push(Instruction::FreeCell(mem));
			}
		} else {
			let mem = get_variable_mem(
				&scopes,
				VariableSpec {
					name: var_name.clone(),
					arr_num: None,
				},
			);
			instructions.push(Instruction::FreeCell(mem));
		}
	}

	instructions
}

// this is subject to change
#[derive(Debug)]
pub enum Instruction {
	AllocateCell(usize), // most of the below comments are wrong, usize is a unique id of an allocated cell
	FreeCell(usize), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(usize), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop,
	AddToCell(usize, u8),
	ClearCell(usize), // not sure if this should be here, seems common enough that it should be
	OutputCell(usize),
	// TODO: contiguous cells for quick iterations?
	// AllocateContiguousCells(usize),
	// FreeContiguousCells(usize), // number indicates
}

// TOOD: Vec<Scope> impl?
// not sure if should be an impl of Scopes type
// takes in the current compiler scope and a variable/arr reference
// returns the allocation stack offset for that variable
fn get_variable_mem(scopes: &[Scope], var: VariableSpec) -> usize {
	if let Some(scope) = scopes.last() {
		// for some reason we don't have any variable allocations in the top scope from within b()
		if let Some(mem) = scope.variable_map.get(&var) {
			// this scope has the variable allocated
			// offset this based on the total of all prior stacks, TODO: maybe redesign this? idk
			// not sure about this, it might work out
			// TODO: fix this, this is currently broken because it doesn't properly take into account memory frees and booleans and stuff because the map doesn't change every time a cell is freed but the stack does, hmm
			// these comments are wrong I think, this whole allocation stack algorithm is kinda screwed?

			return scopes[..(scopes.len() - 1)]
				.iter()
				.fold(*mem, |sum, s| sum + s.allocations);
		} else if let Some((
			VariableSpec {
				name: _,
				arr_num: arg_len,
			},
			VariableSpec {
				name: alias_name,
				arr_num: alias_index,
			},
		)) = scope
			.translations_map
			.iter()
			.find(|(VariableSpec { name, arr_num: _ }, _)| name.eq(&var.name))
		{
			// TODO: variable spec shit, recursion with f(a[2]) type stuff
			// if var.arr_num = None, we good
			// if var.arr_num = Some, I don't think we need to worry about it unless the argument is arr_num = None? I think that should panic (def f(e){output e[2];};)
			// if alias_arr_num = None, we even more good?
			// if alias_arr_num = Some, we still good, unless the argument is arr_num = Some, because that would mean we would need 2d arrays? panic

			// need to translate to an alias and call recursively, could do this iterately as well but whatever (tail recursion)
			return get_variable_mem(&scopes[..(scopes.len() - 1)], {
				// match should return a VariableSpec
				match (&var.arr_num, alias_index, arg_len) {
					(None, alias_index, None) => VariableSpec {
						name: alias_name.clone(),
						arr_num: alias_index.clone(),
					},
					// alias is same length array as argument definition, basically just change the name to the alias
					// TODO: error check here for incorrect indices? Actually probably should be error checked elsewhere? Maybe a function for the variable translations which handles this
					(Some(index), None, Some(_)) => VariableSpec {
						name: alias_name.clone(),
						arr_num: Some(*index),
					},
					// trying to get the full length object, we just want to return one cell
					(None, _, Some(_)) | (Some(_), _, None) | (Some(_), Some(_), Some(_)) => {
						panic!(
							"Invalid variable translation: {} => {alias_name}[{alias_index:#?}]",
							var.name
						);
					}
				}
			});
		} else {
			return get_variable_mem(&scopes[0..(scopes.len() - 1)], var);
		}
	}

	panic!("Variable not found: {var:#?}");
}

// TODO: make this a recursive structure with an impl for all the important memory things, currently this sucks
#[derive(Clone, Debug)]
pub struct Scope {
	// stack of memory/cells that have been allocated (peekable stack)
	allocations: usize,
	// array length of each variable (not sure if this is needed, probably not the best way of doing it either)
	variable_sizes: HashMap<String, Option<usize>>,
	// mappings for variable names to places on above stack
	variable_map: HashMap<VariableSpec, usize>,
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	// originally this was just string to string, but we need to be able to map a single-bit variable to a cell of an outer array variable
	translations_map: Vec<(VariableSpec, VariableSpec)>,

	// functions accessible by any code within or in the current scope
	functions_map: HashMap<String, Function>,
}

#[derive(Clone, Debug)] // probably shouldn't be cloning here but whatever
struct Function {
	arguments: Vec<VariableSpec>,
	block: Vec<Clause>,
}
// represents a position in a stack relative to the head/top

impl Scope {
	pub fn new() -> Scope {
		Scope {
			allocations: 0,
			variable_sizes: HashMap::new(),
			variable_map: HashMap::new(),
			translations_map: Vec::new(),
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
		let mem = self.push_memory_cell();
		self.variable_map.insert(var, mem);
	}

	fn push_memory_cell(&mut self) -> usize {
		self.allocations += 1;
		self.allocations - 1
	}

	// TOOD: Vec<Scope> impl? REFACtOR!
	fn allocate_unnamed_mem(&mut self, scopes: &[Scope]) -> usize {
		let current_scope_relative = self.push_memory_cell();
		scopes
			.iter()
			.fold(current_scope_relative, |sum, s| sum + s.allocations)
	}
}
