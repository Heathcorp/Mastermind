// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip};

use crate::{
	builder::{Builder, Opcode, TapeCell},
	macros::macros::{r_assert, r_panic},
	parser::{
		Clause, Expression, ExtendedOpcode, Reference, VariableDefinition, VariableTarget,
		VariableTargetReferenceChain, VariableTypeReference,
	},
	MastermindConfig,
};

// memory stuff is all WIP and some comments may be incorrect

pub struct Compiler<'a> {
	pub config: &'a MastermindConfig,
}

impl Compiler<'_> {
	pub fn compile<'a>(
		&'a self,
		clauses: &[Clause],
		outer_scope: Option<&'a Scope>,
	) -> Result<Scope<'a>, String> {
		let mut scope = if let Some(outer) = outer_scope {
			outer.open_inner()
		} else {
			Scope::new()
		};

		// hoist functions to top
		let mut filtered_clauses: Vec<Clause> = Vec::new();
		for clause in clauses {
			if let Clause::DefineFunction {
				name,
				arguments,
				block,
			} = clause
			{
				scope.functions.insert(
					name.clone(),
					Function {
						arguments: arguments.clone(),
						block: block.clone(),
					},
				);
			} else {
				filtered_clauses.push(clause.clone());
			}
		}

		for clause in filtered_clauses {
			match clause {
				Clause::DeclareVariable {
					var,
					location_specifier,
				} => {
					// create an allocation in the scope
					scope.allocate_variable(var, location_specifier)?;
				}
				Clause::DeclareStructType { name, fields } => {
					scope.register_struct_definition(&name, fields)?;
				}
				Clause::DefineVariable {
					var,
					location_specifier,
					value,
				} => {
					// same as above except we initialise the variable
					let absolute_type = scope.allocate_variable(var.clone(), location_specifier)?;

					match (absolute_type, &value) {
						(
							ValueType::Cell,
							Expression::NaturalNumber(_)
							| Expression::SumExpression {
								sign: _,
								summands: _,
							}
							| Expression::VariableReference(_),
						) => {
							let cell = scope.get_cell(&VariableTarget::from_definition(&var))?;
							_add_expr_to_cell(&mut scope, &value, cell)?;
						}

						// multi-cell arrays and (array literals or strings)
						(ValueType::Array(_, _), Expression::ArrayLiteral(expressions)) => {
							let cells =
								scope.get_array_cells(&VariableTarget::from_definition(&var))?;
							r_assert!(
								expressions.len() == cells.len(),
								"Variable \"{var}\" cannot be initialised to array of length {}",
								expressions.len()
							);
							for (cell, expr) in zip(cells, expressions) {
								_add_expr_to_cell(&mut scope, expr, cell)?;
							}
						}
						(ValueType::Array(_, _), Expression::StringLiteral(s)) => {
							let cells =
								scope.get_array_cells(&VariableTarget::from_definition(&var))?;
							r_assert!(
								s.len() == cells.len(),
								"Variable \"{var}\" cannot be initialised to string of length {}",
								s.len()
							);
							for (cell, chr) in zip(cells, s.bytes()) {
								scope.push_instruction(Instruction::AddToCell(cell, chr));
							}
						}

						(
							ValueType::Array(_, _),
							Expression::VariableReference(variable_target),
						) => r_panic!(
							"Cannot assign array \"{var}\" from variable reference \
\"{variable_target}\". Unimplemented."
						),
						(
							ValueType::Array(_, _),
							Expression::NaturalNumber(_)
							| Expression::SumExpression {
								sign: _,
								summands: _,
							},
						) => r_panic!("Cannot assign single value to array \"{var}\"."),

						(
							ValueType::DictStruct(_),
							Expression::SumExpression {
								sign: _,
								summands: _,
							}
							| Expression::NaturalNumber(_)
							| Expression::VariableReference(_)
							| Expression::ArrayLiteral(_)
							| Expression::StringLiteral(_),
						) => r_panic!(
							"Cannot assign value to struct type \"{var}\", initialise it instead."
						),

						(ValueType::Cell, Expression::ArrayLiteral(_)) => {
							r_panic!("Cannot assign array to single-cell variable \"{var}\".")
						}
						(ValueType::Cell, Expression::StringLiteral(_)) => {
							r_panic!("Cannot assign string to single-cell variable \"{var}\".")
						}
					}

					// 					match (&var.var_type, &value) {
					// 						(
					// 							VariableType::Cell,
					// 							Expression::SumExpression {
					// 								sign: _,
					// 								summands: _,
					// 							}
					// 							| Expression::NaturalNumber(_)
					// 							| Expression::VariableReference(_),
					// 						) => {
					// 							_add_expr_to_cell(
					// 								&mut scope,
					// 								value,
					// 								CellReference {
					// 									memory_id,
					// 									index: None,
					// 								},
					// 							)?;
					// 						}
					// 						(
					// 							VariableType::Array(inner_type, len),
					// 							Expression::ArrayLiteral(expressions),
					// 						) => {
					// 							let VariableType::Cell = **inner_type else {
					// 								r_panic!(
					// 									"Variable \"{var}\" cannot be initialised with array literal."
					// 								);
					// 							};
					// 							// for each expression in the array, perform above operations
					// 							r_assert!(
					// 								expressions.len() == *len,
					// 								"Variable \"{var}\" cannot be initialised to array of length {}",
					// 								expressions.len()
					// 							);
					// 							for (i, expr) in zip(0..*len, expressions) {
					// 								_add_expr_to_cell(
					// 									&mut scope,
					// 									expr.clone(),
					// 									CellReference {
					// 										memory_id,
					// 										index: Some(i),
					// 									},
					// 								)?;
					// 							}
					// 						}
					// 						(VariableType::Array(inner_type, len), Expression::StringLiteral(s)) => {
					// 							let VariableType::Cell = **inner_type else {
					// 								r_panic!(
					// 									"Variable \"{var}\" cannot be initialised with string literal."
					// 								);
					// 							};
					// 							// for each byte of the string, add it to its respective cell
					// 							r_assert!(
					// 								s.len() == *len,
					// 								"Variable \"{var}\" cannot be initialised to string of length {}",
					// 								s.len()
					// 							);
					// 							for (i, c) in zip(0..*len, s.bytes()) {
					// 								scope.push_instruction(Instruction::AddToCell(
					// 									CellReference {
					// 										memory_id,
					// 										index: Some(i),
					// 									},
					// 									c,
					// 								));
					// 							}
					// 						}
					// 						(VariableType::Cell, _) | (VariableType::Array(_, _), _) => r_panic!(
					// 							"Something went wrong when initialising variable \"{var}\". \
					// This error should never occur."
					// 						),
					// 						_ => r_panic!(
					// 							"Cannot initialise variable \"{var}\" with expression {value:#?}"
					// 						),
					// 					};
				}
				Clause::SetVariable {
					var,
					value,
					self_referencing,
				} => match (var.is_spread, self_referencing) {
					(false, false) => {
						let cell = scope.get_cell(&var)?;
						scope.push_instruction(Instruction::ClearCell(cell.clone()));
						_add_expr_to_cell(&mut scope, &value, cell)?;
					}
					(false, true) => {
						let cell = scope.get_cell(&var)?;
						_add_self_referencing_expr_to_cell(&mut scope, value, cell, true)?;
					}
					(true, _) => {
						r_panic!("Unsupported operation, assigning to spread variable: {var}");
						// TODO: support spread assigns?
						// let cells = scope.get_array_cells(&var)?;
						// etc...
					}
				},
				Clause::AddToVariable {
					var,
					value,
					self_referencing,
				} => {
					let Ok(cell) = scope.get_cell(&var) else {
						r_panic!("Invalid target \"{var}\" for add-assign operation, target should be a cell.");
					};

					_add_expr_to_cell(&mut scope, &value, cell)?;
				}

				Clause::AssertVariableValue { var, value } => {
					// unfortunately no array assertions due to a limitation with my data-structure/algorithm design
					let imm = {
						match value {
							Some(expr) => {
								let (imm, adds, subs) = expr.flatten()?;

								r_assert!(
									adds.len() == 0 && subs.len() == 0,
									"Expected compile-time constant expression in assertion for {var}"
								);

								Some(imm)
							}
							None => None,
						}
					};

					let cell = scope.get_cell(&var)?;
					scope.push_instruction(Instruction::AssertCellValue(cell, imm));
				}
				Clause::InputVariable { var } => {
					let cell = scope.get_cell(&var)?;
					// TODO: support spread operations here again
					scope.push_instruction(Instruction::InputToCell(cell))
				}
				Clause::OutputValue { value } => {
					match value {
						Expression::VariableReference(var) => match var.is_spread {
							false => {
								let cell = scope.get_cell(&var)?;
								scope.push_instruction(Instruction::OutputCell(cell));
							}
							true => {
								let cells = scope.get_array_cells(&var)?;
								for cell in cells {
									scope.push_instruction(Instruction::OutputCell(cell));
								}
							}
						},
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_) => {
							// allocate a temporary cell and add the expression to it, output, then clear
							let temp_mem_id = scope.push_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = CellReference {
								memory_id: temp_mem_id,
								index: None,
							};

							_add_expr_to_cell(&mut scope, &value, cell)?;
							scope.push_instruction(Instruction::OutputCell(cell));
							scope.push_instruction(Instruction::ClearCell(cell));

							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
						Expression::ArrayLiteral(expressions) => {
							// same as above, except reuse the temporary cell after each output
							let temp_mem_id = scope.push_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = CellReference {
								memory_id: temp_mem_id,
								index: None,
							};

							for value in expressions {
								_add_expr_to_cell(&mut scope, &value, cell)?;
								scope.push_instruction(Instruction::OutputCell(cell));
								scope.push_instruction(Instruction::ClearCell(cell));
							}

							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
						Expression::StringLiteral(s) => {
							// same as above, allocate one temporary cell and reuse it for each character
							let temp_mem_id = scope.push_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = CellReference {
								memory_id: temp_mem_id,
								index: None,
							};

							for c in s.bytes() {
								scope.push_instruction(Instruction::AddToCell(cell, c));
								scope.push_instruction(Instruction::OutputCell(cell));
								scope.push_instruction(Instruction::ClearCell(cell));
							}

							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
					}
				}
				Clause::WhileLoop { var, block } => {
					let cell = scope.get_cell(&var)?;

					// open loop on variable
					scope.push_instruction(Instruction::OpenLoop(cell));

					// recursively compile instructions
					// TODO: when recursively compiling, check which things changed based on a return info value
					let loop_scope = self.compile(&block, Some(&scope))?;
					scope
						.instructions
						.extend(loop_scope.finalise_instructions(true));

					// close the loop
					scope.push_instruction(Instruction::CloseLoop(cell));
				}
				Clause::CopyLoop {
					source,
					targets,
					block,
					is_draining,
				} => {
					// TODO: refactor this, there is duplicate code with copying the source value cell
					let (source_cell, free_source_cell) = match (is_draining, &source) {
						// draining loops can drain from an expression or a variable
						(true, Expression::VariableReference(var)) => (scope.get_cell(var)?, false),
						(true, _) => {
							// any other kind of expression, allocate memory for it automatically
							let id = scope.push_memory_id();
							scope
								.push_instruction(Instruction::Allocate(Memory::Cell { id }, None));
							let new_cell = CellReference {
								memory_id: id,
								index: None,
							};
							_add_expr_to_cell(&mut scope, &source, new_cell)?;
							(new_cell, true)
						}
						(false, Expression::VariableReference(var)) => {
							let cell = scope.get_cell(var)?;

							let new_mem_id = scope.push_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: new_mem_id },
								None,
							));

							let new_cell = CellReference {
								memory_id: new_mem_id,
								index: None,
							};

							_copy_cell(&mut scope, cell, new_cell, 1);

							(new_cell, true)
						}
						(false, _) => {
							r_panic!("Cannot copy from {source:#?}, use a drain loop instead")
						}
					};
					scope.push_instruction(Instruction::OpenLoop(source_cell));

					// recurse
					let loop_scope = self.compile(&block, Some(&scope))?;
					// TODO: refactor, make a function in scope trait to do this automatically
					scope
						.instructions
						.extend(loop_scope.finalise_instructions(true));

					// copy into each target and decrement the source
					for target in targets {
						match target.is_spread {
							false => {
								let cell = scope.get_cell(&target)?;
								scope.push_instruction(Instruction::AddToCell(cell, 1));
							}
							true => {
								let cells = scope.get_array_cells(&target)?;
								for cell in cells {
									scope.push_instruction(Instruction::AddToCell(cell, 1));
								}
							}
						}
					}

					scope.push_instruction(Instruction::AddToCell(source_cell, -1i8 as u8)); // 255
					scope.push_instruction(Instruction::CloseLoop(source_cell));

					// free the source cell if it was a expression we just created
					if free_source_cell {
						scope.push_instruction(Instruction::Free(source_cell.memory_id));
					}
				}
				Clause::IfElse {
					condition,
					if_block,
					else_block,
				} => {
					if if_block.is_none() && else_block.is_none() {
						panic!("Expected block in if/else statement");
					};
					let mut new_scope = scope.open_inner();

					let condition_mem_id = new_scope.push_memory_id();
					new_scope.push_instruction(Instruction::Allocate(
						Memory::Cell {
							id: condition_mem_id,
						},
						None,
					));
					let condition_cell = CellReference {
						memory_id: condition_mem_id,
						index: None,
					};

					let else_condition_cell = match else_block {
						Some(_) => {
							let else_mem_id = new_scope.push_memory_id();
							new_scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: else_mem_id },
								None,
							));
							let else_cell = CellReference {
								memory_id: else_mem_id,
								index: None,
							};
							new_scope.push_instruction(Instruction::AddToCell(else_cell, 1));
							Some(else_cell)
						}
						None => None,
					};

					// copy the condition expression to the temporary condition cell
					_add_expr_to_cell(&mut new_scope, &condition, condition_cell)?;

					new_scope.push_instruction(Instruction::OpenLoop(condition_cell));
					// TODO: think about optimisations for clearing this variable, as the builder won't shorten it for safety as it doesn't know this loop is special
					new_scope.push_instruction(Instruction::ClearCell(condition_cell));

					// set the else condition cell
					// above comment about optimisations also applies here
					if let Some(cell) = else_condition_cell {
						new_scope.push_instruction(Instruction::ClearCell(cell));
					};

					// recursively compile if block
					if let Some(block) = if_block {
						let if_scope = self.compile(&block, Some(&new_scope))?;
						new_scope
							.instructions
							.extend(if_scope.finalise_instructions(true));
					};

					// close if block
					new_scope.push_instruction(Instruction::CloseLoop(condition_cell));
					new_scope.push_instruction(Instruction::Free(condition_cell.memory_id));

					// else block:
					if let Some(cell) = else_condition_cell {
						new_scope.push_instruction(Instruction::OpenLoop(cell));
						// again think about how to optimise this clear in the build step
						new_scope.push_instruction(Instruction::ClearCell(cell));

						// recursively compile else block
						// TODO: fix this bad practice unwrap
						let block = else_block.unwrap();
						let else_scope = self.compile(&block, Some(&new_scope))?;
						new_scope
							.instructions
							.extend(else_scope.finalise_instructions(true));

						new_scope.push_instruction(Instruction::CloseLoop(cell));
						new_scope.push_instruction(Instruction::Free(cell.memory_id));
					}

					// extend the inner scopes instructions onto the outer one
					scope
						.instructions
						.extend(new_scope.finalise_instructions(true));
				}
				Clause::Block(clauses) => {
					let new_scope = self.compile(&clauses, Some(&scope))?;
					scope
						.instructions
						.extend(new_scope.finalise_instructions(true));
				}
				Clause::InlineBrainfuck {
					location_specifier,
					clobbered_variables,
					operations,
				} => {
					// loop through the opcodes
					let mut expanded_bf: Vec<Opcode> = Vec::new();
					for op in operations {
						match op {
							ExtendedOpcode::Block(mm_clauses) => {
								// create a scope object for functions from the outside scope
								let functions_scope = scope.open_inner_templates_only();
								// compile the block and extend the operations

								let compiler = Compiler {
									config: &self.config,
								};
								let instructions = compiler
									.compile(&mm_clauses, Some(&functions_scope))?
									.finalise_instructions(false);
								// compile without cleaning up top level variables, this is the brainfuck programmer's responsibility
								// TODO: figure out how to make the compiler return to the initial head position before building and re-adding?
								// IMPORTANT!!!!!!!!!!!!
								let builder = Builder {
									config: &self.config,
								};
								let built_code = builder.build(instructions, true)?;
								// IMPORTANT TODO: MAKE SURE IT RETURNS TO THE SAME POSITION
								expanded_bf.extend(built_code);
							}
							ExtendedOpcode::Add => expanded_bf.push(Opcode::Add),
							ExtendedOpcode::Subtract => expanded_bf.push(Opcode::Subtract),
							ExtendedOpcode::Right => expanded_bf.push(Opcode::Right),
							ExtendedOpcode::Left => expanded_bf.push(Opcode::Left),
							ExtendedOpcode::OpenLoop => expanded_bf.push(Opcode::OpenLoop),
							ExtendedOpcode::CloseLoop => expanded_bf.push(Opcode::CloseLoop),
							ExtendedOpcode::Output => expanded_bf.push(Opcode::Output),
							ExtendedOpcode::Input => expanded_bf.push(Opcode::Input),
						}
					}
					scope.push_instruction(Instruction::InsertBrainfuckAtCell(
						expanded_bf,
						location_specifier,
					));
					// assert that we clobbered the variables
					// not sure whether this should go before or after the actual bf code
					for var in clobbered_variables {
						todo!();
						// let mem = scope.get_memory(&var)?;
						// // little bit of duplicate code from the copyloop clause here:
						// match mem {
						// 	Memory::Cell { id } => {
						// 		scope.push_instruction(Instruction::AssertCellValue(
						// 			CellReference {
						// 				memory_id: id,
						// 				index: None,
						// 			},
						// 			None,
						// 		))
						// 	}
						// 	Memory::Cells { id, len } => match target_index {
						// 		None => {
						// 			// should only happen if the spread operator is used, ideally this should be obvious with the code
						// 			for i in 0..len {
						// 				scope.push_instruction(Instruction::AssertCellValue(
						// 					CellReference {
						// 						memory_id: id,
						// 						index: Some(i),
						// 					},
						// 					None,
						// 				));
						// 			}
						// 		}
						// 		Some(index) => {
						// 			scope.push_instruction(Instruction::AssertCellValue(
						// 				CellReference {
						// 					memory_id: id,
						// 					index: Some(index),
						// 				},
						// 				None,
						// 			))
						// 		}
						// 	},
						// }
					}
				}
				Clause::CallFunction {
					function_name,
					arguments,
				} => {
					// create variable translations and recursively compile the inner variable block
					let function_definition = scope.get_function(&function_name)?;

					todo!();

					// let mut new_scope = scope.open_inner();
					// let zipped: Result<Vec<ArgumentTranslation>, String> =
					// 	zip(function_definition.arguments.clone().into_iter(), arguments)
					// 		.map(|(arg_def, calling_arg)| {
					// 			Ok(match (arg_def, calling_arg) {
					// 				(
					// 					VariableDefinition::Single { name: def_name },
					// 					VariableTarget::Single { name: call_name },
					// 				) => ArgumentTranslation::SingleFromSingle(def_name, call_name),
					// 				(
					// 					// this is a minor hack, the parser will parse a calling argument as a single even though it is really targeting a multi
					// 					VariableDefinition::Multi {
					// 						name: def_name,
					// 						len: _,
					// 					},
					// 					VariableTarget::Single { name: call_name },
					// 				) => ArgumentTranslation::MultiFromMulti(def_name, call_name),
					// 				(
					// 					VariableDefinition::Single { name: def_name },
					// 					VariableTarget::MultiCell {
					// 						name: call_name,
					// 						index,
					// 					},
					// 				) => ArgumentTranslation::SingleFromMultiCell(
					// 					def_name,
					// 					(call_name, index),
					// 				),
					// 				(def_var, call_var) => {
					// 					r_panic!(
					// 						"Cannot translate {call_var} as argument {def_var}"
					// 					)
					// 				}
					// 			})
					// 		})
					// 		.collect();
					// new_scope.variable_aliases.extend(zipped?);

					// // recurse
					// let loop_scope = self.compile(&function_definition.block, Some(&new_scope))?;
					// new_scope
					// 	.instructions
					// 	.extend(loop_scope.finalise_instructions(true));

					// // extend the inner scope instructions onto the outer scope
					// // maybe function call compiling should be its own function?
					// scope
					// 	.instructions
					// 	.extend(new_scope.finalise_instructions(false));
				}
				Clause::DefineFunction {
					name: _,
					arguments: _,
					block: _,
				} => (),
			}
		}

		Ok(scope)
	}
}

// not sure if this should be in the scope impl?
// helper function for a common use-case
// flatten an expression and add it to a specific cell (using copies and adds, etc)
fn _add_expr_to_cell(
	scope: &mut Scope,
	expr: &Expression,
	cell: CellReference,
) -> Result<(), String> {
	let (imm, adds, subs) = expr.flatten()?;

	scope.push_instruction(Instruction::AddToCell(cell.clone(), imm));

	let mut adds_set = HashMap::new();
	for var in adds {
		let n = adds_set.remove(&var).unwrap_or(0);
		adds_set.insert(var, n + 1);
	}
	for var in subs {
		let n = adds_set.remove(&var).unwrap_or(0);
		adds_set.insert(var, n - 1);
	}

	for (source, constant) in adds_set {
		let source_cell = scope.get_cell(&source)?;
		_copy_cell(scope, source_cell, cell.clone(), constant);
	}

	Ok(())
}

//This function allows you to add a self referencing expression to the cell
//Separate this to ensure that normal expression don't require the overhead of copying
fn _add_self_referencing_expr_to_cell(
	scope: &mut Scope,
	expr: Expression,
	cell: CellReference,
	pre_clear: bool,
) -> Result<(), String> {
	//Create a new temp cell to store the current cell value
	let temp_mem_id = scope.push_memory_id();
	scope.push_instruction(Instruction::Allocate(
		Memory::Cell { id: temp_mem_id },
		None,
	));
	let temp_cell = CellReference {
		memory_id: temp_mem_id,
		index: None,
	};
	// TODO: make this more efficent by not requiring a clear cell after,
	// i.e. simple move instead of copy by default for set operations (instead of +=)
	_copy_cell(scope, cell, temp_cell, 1);
	// Then if we are doing a += don't pre-clear otherwise Clear the current cell and run the same actions as _add_expr_to_cell
	if pre_clear {
		scope.push_instruction(Instruction::ClearCell(cell.clone()));
	}

	let (imm, adds, subs) = expr.flatten()?;

	scope.push_instruction(Instruction::AddToCell(cell.clone(), imm));

	let mut adds_set = HashMap::new();
	for var in adds {
		let n = adds_set.remove(&var).unwrap_or(0);
		adds_set.insert(var, n + 1);
	}
	for var in subs {
		let n = adds_set.remove(&var).unwrap_or(0);
		adds_set.insert(var, n - 1);
	}

	for (source, constant) in adds_set {
		let source_cell = scope.get_cell(&source)?;
		//If we have an instance of the original cell being added simply use our temp cell value
		// (crucial special sauce)
		if source_cell.memory_id == cell.memory_id && source_cell.index == cell.index {
			_copy_cell(scope, temp_cell, cell.clone(), constant);
		} else {
			_copy_cell(scope, source_cell, cell.clone(), constant);
		}
	}
	//Cleanup
	scope.push_instruction(Instruction::ClearCell(temp_cell));
	scope.push_instruction(Instruction::Free(temp_mem_id));

	Ok(())
}

/// Helper function to copy a cell from one to another leaving the original unaffected
// TODO: make one for draining a cell
fn _copy_cell(
	scope: &mut Scope,
	source_cell: CellReference,
	target_cell: CellReference,
	constant: i32,
) {
	if constant == 0 {
		return;
	}
	// allocate a temporary cell
	let temp_mem_id = scope.push_memory_id();
	scope.push_instruction(Instruction::Allocate(
		Memory::Cell { id: temp_mem_id },
		None,
	));
	let temp_cell = CellReference {
		memory_id: temp_mem_id,
		index: None,
	};
	// copy source to target and temp
	scope.push_instruction(Instruction::OpenLoop(source_cell));
	scope.push_instruction(Instruction::AddToCell(target_cell, constant as u8));
	scope.push_instruction(Instruction::AddToCell(temp_cell, 1));
	scope.push_instruction(Instruction::AddToCell(source_cell, -1i8 as u8));
	scope.push_instruction(Instruction::CloseLoop(source_cell));
	// copy back from temp
	scope.push_instruction(Instruction::OpenLoop(temp_cell));
	scope.push_instruction(Instruction::AddToCell(source_cell, 1));
	scope.push_instruction(Instruction::AddToCell(temp_cell, -1i8 as u8));
	scope.push_instruction(Instruction::CloseLoop(temp_cell));
	scope.push_instruction(Instruction::Free(temp_mem_id));
}

// this is subject to change
#[derive(Debug, Clone)]
pub enum Instruction {
	Allocate(Memory, Option<TapeCell>), // most of the below comments are wrong, usize is a unique id of an allocated cell
	Free(MemoryId), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(CellReference), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop(CellReference), // pass in the cell id, this originally wasn't there but may be useful later on
	AddToCell(CellReference, u8),
	InputToCell(CellReference),
	ClearCell(CellReference), // not sure if this should be here, seems common enough that it should be
	AssertCellValue(CellReference, Option<u8>), // allows the user to hand-tune optimisations further
	OutputCell(CellReference),
	InsertBrainfuckAtCell(Vec<Opcode>, Option<TapeCell>),
}

#[derive(Debug, Clone)]
pub enum Memory {
	Cell { id: MemoryId },
	Cells { id: MemoryId, len: usize },
	// TODO: MappedCells? Maybe hold a list of subfield positions? could be cooked
	// infinite cell something (TODO?)
}
pub type MemoryId = usize;

#[derive(Debug, Clone, Copy)]
pub struct CellReference {
	pub memory_id: MemoryId,
	pub index: Option<usize>,
}

impl Memory {
	pub fn id(&self) -> MemoryId {
		match self {
			Memory::Cell { id } => *id,
			Memory::Cells { id, len: _ } => *id,
		}
	}
	pub fn len(&self) -> usize {
		match self {
			Memory::Cell { id: _ } => 1,
			Memory::Cells { id: _, len } => *len,
		}
	}
}

#[derive(Clone, Debug)]
/// Scope type represents a Mastermind code block,
/// any variables or functions defined within a {block} are owned by the scope and cleaned up before continuing
pub struct Scope<'a> {
	/// a reference to the parent scope, for accessing things defined outside of this scope
	outer_scope: Option<&'a Scope<'a>>,
	/// fn_only: true if syntactic context instead of normal context.
	/// Used for embedded mm so that the inner mm can use outer functions but not variables.
	fn_only: bool,

	/// Number of memory allocations in current scope
	allocations: usize,

	/// Mappings for variable names to memory allocation IDs in current scope
	variable_memory: HashMap<String, (ValueType, Memory)>,

	/// Translations from outer scope variables to current scope variables, used for function arguments
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	// originally this was just string to string, but we need to be able to map a single-bit variable to a cell of an outer array variable
	variable_aliases: Vec<ArgumentTranslation>,

	/// Functions accessible by any code within or in the current scope
	functions: HashMap<String, Function>,
	/// Struct types definitions
	structs: HashMap<String, DictStructType>,

	/// Intermediate instructions generated by the compiler
	instructions: Vec<Instruction>,
}

// TODO: make this work for structs and array and new types
#[derive(Clone, Debug)]
enum ArgumentTranslation {
	SingleFromSingle(String, String),
	SingleFromMultiCell(String, (String, usize)),
	MultiFromMulti(String, String),
}
impl ArgumentTranslation {
	fn get_def_name(&self) -> &String {
		let (ArgumentTranslation::SingleFromSingle(def_name, _)
		| ArgumentTranslation::SingleFromMultiCell(def_name, _)
		| ArgumentTranslation::MultiFromMulti(def_name, _)) = self;
		def_name
	}
	fn get_call_name(&self) -> &String {
		match self {
			ArgumentTranslation::SingleFromSingle(_, call_name)
			| ArgumentTranslation::MultiFromMulti(_, call_name) => call_name,
			ArgumentTranslation::SingleFromMultiCell(_, (call_var, _)) => call_var,
		}
	}
}

#[derive(Clone, Debug)] // probably shouldn't be cloning here but whatever
struct Function {
	arguments: Vec<VariableDefinition>,
	block: Vec<Clause>,
}

#[derive(Clone, Debug)]
/// an absolute definition of a type, as opposed to `VariableTypeReference` which is more of a reference
enum ValueType {
	Cell,
	Array(usize, Box<ValueType>),
	DictStruct(Vec<(String, ValueType)>),
	// TupleStruct(Vec<ValueType>),
}

#[derive(Clone, Debug)]
/// equivalent to ValueType::DictStruct enum variant,
/// Rust doesn't support enum variants as types yet so need this workaround for struct definitions in scope object
struct DictStructType(Vec<(String, ValueType)>);
impl ValueType {
	fn from_struct(struct_def: DictStructType) -> Self {
		ValueType::DictStruct(struct_def.0)
	}

	/// get the cell index of a specific variable target
	fn _get_target_cell_index(&self, subfield_chain: &[Reference]) -> usize {
		unimplemented!()
	}

	/// get a subfield's type as well as memory cell index
	pub fn get_subfield(
		&self,
		subfield_chain: &VariableTargetReferenceChain,
	) -> Result<(&ValueType, usize), String> {
		let mut cur_field = self;
		let mut cur_index = 0;
		for subfield_ref in subfield_chain.0.iter() {
			match (cur_field, subfield_ref) {
				(ValueType::Array(len, element_type), Reference::Index(index)) => {
					r_assert!(
						index < len,
						"Index \"{subfield_ref}\" must be less than array length ({len})."
					);
					cur_index += element_type.size() * index;
					cur_field = element_type;
				}
				(ValueType::DictStruct(items), Reference::NamedField(subfield_name)) => {
					let mut cell_offset_tally = 0;
					let Some((_, subfield_type)) = items.iter().find(|(item_name, item_type)| {
						match item_name == subfield_name {
							true => true,
							false => {
								cell_offset_tally += item_type.size();
								false
							}
						}
					}) else {
						r_panic!("Could not find subfield \"{subfield_ref}\" in struct type")
					};
					cur_index += cell_offset_tally;
					cur_field = subfield_type;
				}

				(ValueType::DictStruct(_), Reference::Index(_)) => {
					r_panic!("Cannot read index subfield \"{subfield_ref}\" of struct type.")
				}
				(ValueType::Array(_, _), Reference::NamedField(_)) => {
					r_panic!("Cannot read named subfield \"{subfield_ref}\" of array type.")
				}
				(ValueType::Cell, subfield_ref) => {
					r_panic!("Attempted to get subfield \"{subfield_ref}\" of cell type.")
				}
			}
		}
		Ok((cur_field, cur_index))
	}
}

impl ValueType {
	/// return the type size in cells
	fn size(&self) -> usize {
		match self {
			ValueType::Cell => 1,
			ValueType::Array(len, value_type) => *len * value_type.size(),
			ValueType::DictStruct(items) => items
				.iter()
				.map(|(_field_name, field_type)| field_type.size())
				.sum(),
		}
	}
}

impl Scope<'_> {
	pub fn new() -> Scope<'static> {
		Scope {
			outer_scope: None,
			fn_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			variable_aliases: Vec::new(),
			functions: HashMap::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	// I don't love this system of deciding what to clean up at the end in this specific function, but I'm not sure what the best way to achieve this would be
	// this used to be called "get_instructions" but I think this more implies things are being modified
	pub fn finalise_instructions(mut self, clean_up_variables: bool) -> Vec<Instruction> {
		if !clean_up_variables {
			return self.instructions;
		}

		// optimisations could go here?
		// TODO: add some optimisations from the builder to here

		// create instructions to free cells
		let mut clear_instructions = vec![];
		for (_var_name, (_var_type, memory)) in self.variable_memory.iter() {
			match memory {
				Memory::Cell { id } => {
					clear_instructions.push(Instruction::ClearCell(CellReference {
						memory_id: *id,
						index: None,
					}))
				}
				Memory::Cells { id, len } => {
					for i in 0..*len {
						clear_instructions.push(Instruction::ClearCell(CellReference {
							memory_id: *id,
							index: Some(i),
						}))
					}
				}
			}
			clear_instructions.push(Instruction::Free(memory.id()));
		}
		for instr in clear_instructions {
			self.push_instruction(instr);
		}

		self.instructions
	}

	fn push_instruction(&mut self, instruction: Instruction) {
		self.instructions.push(instruction);
	}

	/// open a scope within the current one, any time there is a {} in Mastermind, this is called
	fn open_inner(&self) -> Scope {
		Scope {
			outer_scope: Some(self),
			fn_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			variable_aliases: Vec::new(),
			functions: HashMap::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	// syntactic context instead of normal context
	// used for embedded mm so that the inner mm can use outer functions
	fn open_inner_templates_only(&self) -> Scope {
		Scope {
			outer_scope: Some(self),
			fn_only: true,
			allocations: 0,
			variable_memory: HashMap::new(),
			variable_aliases: Vec::new(),
			functions: HashMap::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	/// get the correct variable type and allocate the right amount of cells for it
	fn allocate_variable(
		&mut self,
		var: VariableDefinition,
		location_specifier: Option<i32>,
	) -> Result<&ValueType, String> {
		r_assert!(
			!self.variable_memory.contains_key(&var.name),
			"Cannot allocate variable {var} twice in the same scope"
		);

		// get absolute type
		let full_type: ValueType = self.create_absolute_type(&var.var_type)?;
		// get absolute type size
		let id = self.push_memory_id();
		let memory = match &full_type {
			ValueType::Cell => Memory::Cell { id },
			_ => Memory::Cells {
				id,
				len: full_type.size(),
			},
		};
		// save variable in scope memory
		let None = self
			.variable_memory
			.insert(var.name.clone(), (full_type, memory.clone()))
		else {
			r_panic!("Unreachable error occurred when allocating {var}");
		};

		// allocate
		self.push_instruction(Instruction::Allocate(memory.clone(), location_specifier));

		// return a reference to the created full type
		Ok(&self.variable_memory.get(&var.name).unwrap().0)
	}

	// fn allocate_unnamed_cell(&mut self) -> Memory {
	// 	let mem_id = self.create_memory_id();
	// 	Memory::Cell { id: mem_id }
	// }

	fn push_memory_id(&mut self) -> MemoryId {
		let current_scope_relative = self.allocations;
		self.allocations += 1;
		current_scope_relative + self.allocation_offset()
	}

	/// recursively tally the allocation stack size of the outer scope, does not include this scope
	fn allocation_offset(&self) -> usize {
		// little bit of a hack but works for now
		if self.fn_only {
			return 0;
		}
		if let Some(outer_scope) = self.outer_scope {
			outer_scope.allocations + outer_scope.allocation_offset()
		} else {
			0
		}
	}

	fn get_function(&self, name: &str) -> Result<&Function, String> {
		// this function is unaffected by the self.fn_only flag
		if let Some(func) = self.functions.get(name) {
			Ok(func)
		} else if let Some(outer_scope) = self.outer_scope {
			// again not sure if Ok ? is a good pattern
			Ok(outer_scope.get_function(name)?)
		} else {
			r_panic!("Could not find function \"{name}\" in current scope");
		}
	}

	/// define a struct in this scope
	fn register_struct_definition(
		&mut self,
		struct_name: &str,
		fields: Vec<VariableDefinition>,
	) -> Result<(), String> {
		let absolute_fields = fields
			.into_iter()
			.map(|f| Ok((f.name, self.create_absolute_type(&f.var_type)?)))
			.collect::<Result<Vec<(String, ValueType)>, String>>()?;

		let None = self
			.structs
			.insert(struct_name.to_string(), DictStructType(absolute_fields))
		else {
			r_panic!("Cannot define struct {struct_name} more than once in same scope.");
		};

		Ok(())
	}

	/// recursively find the definition of a struct type by searching up the scope call stack
	fn get_struct_definition(&self, struct_name: &str) -> Result<&DictStructType, String> {
		if let Some(struct_def) = self.structs.get(struct_name) {
			Ok(struct_def)
		} else if let Some(outer_scope) = self.outer_scope {
			// recurse
			outer_scope.get_struct_definition(struct_name)
		} else {
			r_panic!("No definition found for struct \"{struct_name}\".");
		}
	}

	/// construct an absolute type from a type reference
	fn create_absolute_type(&self, type_ref: &VariableTypeReference) -> Result<ValueType, String> {
		Ok(match type_ref {
			VariableTypeReference::Cell => ValueType::Cell,
			VariableTypeReference::Struct(struct_type_name) => {
				ValueType::from_struct(self.get_struct_definition(struct_type_name)?.clone())
			}
			VariableTypeReference::Array(variable_type_reference, len) => ValueType::Array(
				*len,
				Box::new(self.create_absolute_type(variable_type_reference)?),
			),
		})
	}

	/// return a cell reference for a variable target
	fn get_cell(&self, target: &VariableTarget) -> Result<CellReference, String> {
		// get the absolute type of the variable, as well as the memory allocation
		let (full_type, memory) = self.get_variable_memory(&target.name)?;
		// get the correct index within the memory and return
		Ok(match (&target.subfields, full_type, memory) {
			(None, ValueType::Cell, Memory::Cell { id }) => CellReference {
				memory_id: *id,
				index: None,
			},
			(
				Some(subfield_chain),
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cells { id, len },
			) => {
				let (subfield_type, cell_index) = full_type.get_subfield(&subfield_chain)?;
				let ValueType::Cell = subfield_type else {
					r_panic!("Expected cell type in variable target: {target}");
				};
				r_assert!(cell_index < *len, "Cell reference out of bounds on variable target: {target}. This should not occur.");
				CellReference {
					memory_id: *id,
					index: Some(cell_index),
				}
			}
			(Some(_), ValueType::Cell, Memory::Cell { id: _ }) => {
				r_panic!("Cannot get subfields of cell type: {target}")
			}

			(None, ValueType::DictStruct(_), Memory::Cells { id: _, len: _ })
			| (None, ValueType::DictStruct(_), Memory::Cell { id: _ })
			| (None, ValueType::Array(_, _), Memory::Cells { id: _, len: _ })
			| (None, ValueType::Array(_, _), Memory::Cell { id: _ }) => {
				r_panic!("Expected single cell reference in target: {target}")
			}

			// variable memory returned the wrong memory allocation type for the value type, unreachable
			(Some(_), ValueType::Cell, Memory::Cells { id: _, len: _ })
			| (Some(_), ValueType::Array(_, _), Memory::Cell { id: _ })
			| (Some(_), ValueType::DictStruct(_), Memory::Cell { id: _ })
			| (None, ValueType::Cell, Memory::Cells { id: _, len: _ }) => r_panic!(
				"Invalid memory for value type in target: {target}. This should not occur."
			),
		})
	}

	/// return a list of cell references for an array of cells (not an array of structs)
	fn get_array_cells(&self, target: &VariableTarget) -> Result<Vec<CellReference>, String> {
		let (full_type, memory) = self.get_variable_memory(&target.name)?;
		Ok(match (&target.subfields, full_type, memory) {
			(
				None,
				ValueType::Array(arr_len, element_type),
				Memory::Cells {
					id: mem_id,
					len: mem_len,
				},
			) => {
				let ValueType::Cell = **element_type else {
					r_panic!("Cannot get array cells of struct array: {target}");
				};
				r_assert!(
					arr_len == mem_len,
					"Array memory incorrect length {mem_len} for array length {arr_len}."
				);
				(0..*mem_len)
					.map(|i| CellReference {
						memory_id: *mem_id,
						index: Some(i),
					})
					.collect()
			}
			(Some(_), ValueType::Array(_, value_type), Memory::Cells { id, len }) => todo!(),
			(Some(_), ValueType::DictStruct(items), Memory::Cells { id, len }) => todo!(),

			// not addressing array cells, possibly user error
			(None, ValueType::DictStruct(_), Memory::Cells { id: _, len: _ })
			| (None, ValueType::Cell, Memory::Cell { id: _ }) => {
				r_panic!("Expected cell type in variable target: {target}")
			}

			// subfield references on a cell, user error
			(Some(_), ValueType::Cell, Memory::Cell { id }) => {
				r_panic!("Attempted to retrieve array subfield from cell type: {target}")
			}

			// fucked up memory allocations, not user error
			(_, ValueType::Cell, Memory::Cells { id: _, len: _ })
			| (_, ValueType::Array(_, _), Memory::Cell { id: _ })
			| (_, ValueType::DictStruct(_), Memory::Cell { id: _ }) => r_panic!(
				"Unexpected memory type when accessing \"{target}\". This should not occur."
			),
		})
	}

	/// return the absolute type and memory allocation for a variable name
	fn get_variable_memory(&self, var_name: &str) -> Result<(&ValueType, &Memory), String> {
		// TODO: add function argument translations and embedded bf/mmi scope function restrictions
		match (self.outer_scope, self.variable_memory.get(var_name)) {
			(_, Some((value_type, memory))) => Ok((value_type, memory)),
			(Some(outer_scope), None) => outer_scope.get_variable_memory(var_name),
			(None, None) => r_panic!("No variable found with name \"{var_name}\"."),
		}
		// if let Some(var_def) = self.current_level_get_variable_definition(var.name()) {
		// 	let Some(mem_id) = self.variable_memory.get(var_def) else {
		// 		r_panic!("Something went wrong when compiling. This error should never occur.");
		// 	};
		// 	// base case, variable is defined in this scope level
		// 	// Ok(match (var_def, var) {
		// 	// 	(VariableDefinition::Single { name: _ }, VariableTarget::Single { name: _ }) => {
		// 	// 		Memory::Cell { id: *mem_id }
		// 	// 	}
		// 	// 	(
		// 	// 		VariableDefinition::Multi { name: _, len },
		// 	// 		VariableTarget::MultiCell { name: _, index },
		// 	// 	) => {
		// 	// 		r_assert!(
		// 	// 			*index < *len,
		// 	// 			"Memory access attempt: \"{var}\" out of range for variable: \"{var_def}\""
		// 	// 		);
		// 	// 		Memory::Cells {
		// 	// 			id: *mem_id,
		// 	// 			len: *len,
		// 	// 			target_index: Some(*index),
		// 	// 		}
		// 	// 	}
		// 	// 	(
		// 	// 		VariableDefinition::Multi { name: _, len },
		// 	// 		VariableTarget::MultiSpread { name: _ },
		// 	// 	) => Memory::Cells {
		// 	// 		id: *mem_id,
		// 	// 		len: *len,
		// 	// 		target_index: None,
		// 	// 	},
		// 	// 	_ => {
		// 	// 		r_panic!("Malformed variable reference {var} to {var_def}")
		// 	// 	}
		// 	// })
		// 	todo!();
		// } else if self.fn_only {
		// 	r_panic!("Attempted to access variable memory outside of embedded Mastermind context.");
		// } else if let Some(outer_scope) = self.outer_scope {
		// 	// recursive case
		// 	if let Some(translation) = self
		// 		.variable_aliases
		// 		.iter()
		// 		.find(|translation| *translation.get_def_name() == *var.name())
		// 	{
		// 		todo!();
		// 		// let alias_var = match (translation, var) {
		// 		// 	(
		// 		// 		ArgumentTranslation::SingleFromSingle(_, call_name),
		// 		// 		VariableTarget::Single { name: _ },
		// 		// 		// single variable let g;f(g);def f(h){++h;}c
		// 		// 	) => VariableTarget::Single {
		// 		// 		name: call_name.clone(),
		// 		// 	},
		// 		// 	(
		// 		// 		ArgumentTranslation::SingleFromMultiCell(_, (call_name, call_index)),
		// 		// 		VariableTarget::Single { name: _ },
		// 		// 		// referenced byte passed as single let g[9];f(g[0]);def f(h){++h;}
		// 		// 	) => VariableTarget::MultiCell {
		// 		// 		name: call_name.clone(),
		// 		// 		index: *call_index,
		// 		// 	},
		// 		// 	(
		// 		// 		ArgumentTranslation::MultiFromMulti(_, call_name),
		// 		// 		VariableTarget::MultiCell { name: _, index },
		// 		// 		// referenced byte from multi-byte variable let g[9];f(g);def f(h[9]){++h[0];}
		// 		// 	) => VariableTarget::MultiCell {
		// 		// 		name: call_name.clone(),
		// 		// 		index: *index,
		// 		// 	},
		// 		// 	(
		// 		// 		ArgumentTranslation::MultiFromMulti(_, call_name),
		// 		// 		VariableTarget::MultiSpread { name: _ },
		// 		// 		// spread from multi-byte variable let g[9];f(g);def f(h[9]){output *h;}
		// 		// 	) => VariableTarget::MultiSpread {
		// 		// 		name: call_name.clone(),
		// 		// 	},
		// 		// 	_ => r_panic!(
		// 		// 		"Malformed argument/variable translation {translation:#?}, target: {var}. \
		// 		// 	I realise this error doesn't tell you much, this error should not occur anyway."
		// 		// 	),
		// 		// };
		// 		// Ok(outer_scope.get_memory(&alias_var)?)
		// 	} else {
		// 		// again not sure if Ok + ? is a bad pattern
		// 		Ok(outer_scope.get_memory(var)?)
		// 	}
		// } else {
		// 	r_panic!("No variable {var} found in current scope.");
		// }
	}
}
