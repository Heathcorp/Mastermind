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
				// flatten expression to a list of additions or subtractions
				let (imm, adds, subs) = value.flatten();
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

				// initialise variable values
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

							let (imm, adds, subs) = expr.clone().flatten();
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

								let (imm, adds, subs) = expr.clone().flatten();
								instructions.push(Instruction::AddToCell(mem, imm));
								if (adds.len() + subs.len()) > 0 {
									todo!();
								}
							}
						}
						(Some(len), Expression::StringLiteral(str)) => todo!(),
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
				let mem: usize = get_variable_mem(&scopes, var);
				instructions.push(Instruction::OutputCell(mem));
			}
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
				false => todo!("Copying loop unimplemented"),
			},
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
					.unwrap()
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
		let mem = self.allocate_memory_cell();
		self.variable_map.insert(var, mem);
	}

	fn allocate_memory_cell(&mut self) -> usize {
		self.allocations += 1;
		self.allocations - 1
	}
}
