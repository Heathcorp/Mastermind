// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip};

use crate::{
	builder::{Builder, Opcode, TapeCell},
	macros::macros::{r_assert, r_panic},
	parser::{Clause, Expression, ExtendedOpcode, VariableDefinition, VariableTarget},
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
	) -> Result<Scope, String> {
		let mut scope = if let Some(outer) = outer_scope {
			outer.open_inner()
		} else {
			Scope::new()
		};

		// hoist functions to top
		// also keep track of all variables that are allocated
		let mut filtered_clauses: Vec<Clause> = Vec::new();

		for clause in clauses {
			if let Clause::DeclareVariable {
				var,
				location_specifier: _,
			}
			| Clause::DefineVariable {
				var,
				location_specifier: _,
				value: _,
			} = clause
			{
				// hoisting scope allocations to the top
				scope.allocate_variable_memory(var.clone())?;
			}

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
					let memory = scope.get_memory(&var.clone().to_target())?;
					// push instruction to allocate cell(s) (the number of cells is stored in the Memory object)
					scope.push_instruction(Instruction::Allocate(memory, location_specifier));
				}
				Clause::DefineVariable {
					var,
					location_specifier,
					value,
				} => {
					// same as above except we initialise the variable
					let memory = scope.get_memory(&var.clone().to_target())?;
					let memory_id = memory.id();
					scope.push_instruction(Instruction::Allocate(
						memory.clone(),
						location_specifier,
					));

					match (&var, &value, memory) {
						(
							VariableDefinition::Single { name: _ },
							Expression::SumExpression {
								sign: _,
								summands: _,
							}
							| Expression::NaturalNumber(_)
							| Expression::VariableReference(_),
							Memory::Cell { id: _ },
						) => {
							_add_expr_to_cell(
								&mut scope,
								value,
								Cell {
									memory_id,
									index: None,
								},
							)?;
						}
						(
							VariableDefinition::Multi { name, len },
							Expression::ArrayLiteral(expressions),
							Memory::Cells {
								id: _,
								len: _,
								target_index: _,
							},
						) => {
							// for each expression in the array, perform above operations
							r_assert!(
								expressions.len() == *len,
								"Variable \"{name}\" of length {len} cannot be initialised \
to array expression of length {}",
								expressions.len()
							);
							for (i, expr) in zip(0..*len, expressions) {
								_add_expr_to_cell(
									&mut scope,
									expr.clone(),
									Cell {
										memory_id,
										index: Some(i),
									},
								)?;
							}
						}
						(
							VariableDefinition::Multi { name, len },
							Expression::StringLiteral(s),
							Memory::Cells {
								id: _,
								len: _,
								target_index: _,
							},
						) => {
							// for each byte of the string, add it to its respective cell
							r_assert!(
								s.len() == *len,
								"Variable \"{name}\" of length {len} cannot \
be initialised to string of length {}",
								s.len()
							);
							for (i, c) in zip(0..*len, s.bytes()) {
								scope.push_instruction(Instruction::AddToCell(
									Cell {
										memory_id,
										index: Some(i),
									},
									c,
								));
							}
						}
						(
							VariableDefinition::Single { name: _ },
							_,
							Memory::Cells {
								id: _,
								len: _,
								target_index: _,
							},
						)
						| (
							VariableDefinition::Multi { name: _, len: _ },
							_,
							Memory::Cell { id: _ },
						) => r_panic!(
							"Something went wrong when initialising variable \"{var}\". \
This error should never occur."
						),
						_ => r_panic!(
							"Cannot initialise variable \"{var}\" with expression {value:#?}"
						),
					};
				}
				Clause::SetVariable { var, value } => match (&var, &value) {
					(
						VariableTarget::Single { name: _ },
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_)
						| Expression::VariableReference(_),
					) => {
						let mem = scope.get_memory(&var)?;
						let cell = Cell {
							memory_id: mem.id(),
							index: None,
						};
						scope.push_instruction(Instruction::ClearCell(cell.clone()));
						_add_expr_to_cell(&mut scope, value, cell)?;
					}
					(
						VariableTarget::MultiCell { name: _, index },
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_)
						| Expression::VariableReference(_),
					) => {
						let mem = scope.get_memory(&var)?;
						let cell = Cell {
							memory_id: mem.id(),
							index: Some(*index),
						};
						scope.push_instruction(Instruction::ClearCell(cell.clone()));
						_add_expr_to_cell(&mut scope, value, cell)?;
					}
					(
						VariableTarget::MultiSpread { name: _ },
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_)
						| Expression::VariableReference(_),
					) => r_panic!(
						"Cannot set multi-byte variables using \
spread syntax, use drain <val> into {var} instead."
					),
					(_, Expression::ArrayLiteral(_) | Expression::StringLiteral(_)) => r_panic!(
						"Cannot set multi-byte variables after initialisation\
, set individual bytes with [] subscript operator instead."
					),
					// _ => r_panic!("Cannot set variable \"{var}\" to expression {value:#?}"),
				},
				Clause::AddToVariable { var, value } => match (&var, &value) {
					(
						VariableTarget::Single { name: _ }
						| VariableTarget::MultiCell { name: _, index: _ },
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_)
						| Expression::VariableReference(_),
					) => {
						let Some(cell) = scope.get_memory(&var)?.target_cell() else {
							r_panic!("Unreachable error occurred when adding to {var}");
						};
						_add_expr_to_cell(&mut scope, value, cell)?;
					}
					(
						VariableTarget::MultiSpread { name: _ },
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_)
						| Expression::VariableReference(_),
					) => r_panic!(
						"Cannot add to multi-byte variables using \
spread syntax, use drain <val> into {var} instead."
					),
					(_, Expression::ArrayLiteral(_) | Expression::StringLiteral(_)) => r_panic!(
						"Cannot add to multi-byte variables after initialisation\
, set individual bytes with [] subscript operator instead."
					),
					// _ => r_panic!("Cannot add expression {value:#?} to variable \"{var}\""),
				},
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

					let mem = scope.get_memory(&var)?;
					match &var {
						VariableTarget::Single { name: _ }
						| VariableTarget::MultiCell { name: _, index: _ } => {
							let cell = match mem {
								Memory::Cell { id } => Cell {
									memory_id: id,
									index: None,
								},
								Memory::Cells {
									id,
									len: _,
									target_index: Some(idx),
								} => Cell {
									memory_id: id,
									index: Some(idx),
								},
								_ => r_panic!(
									"Could not access {var} in assertion. This should not occur."
								),
							};
							scope.push_instruction(Instruction::AssertCellValue(cell, imm));
						}
						VariableTarget::MultiSpread { name: _ } => match mem {
							Memory::Cells {
								id,
								len,
								target_index: None,
							} => {
								for i in 0..len {
									let cell = Cell {
										memory_id: id,
										index: Some(i),
									};
									scope.push_instruction(Instruction::AssertCellValue(cell, imm));
								}
							}
							_ => r_panic!("Could not access spread variable {var} in assertion."),
						},
					}
					// _ => r_panic!("Unsupported compile-time assertion: {var} = {value:#?}"),
				}
				Clause::InputVariable { var } => {
					let mem = scope.get_memory(&var)?;
					match (&var, mem) {
						(VariableTarget::Single { name: _ }, Memory::Cell { id: memory_id }) => {
							scope.push_instruction(Instruction::InputToCell(Cell {
								memory_id,
								index: None,
							}))
						}
						(
							VariableTarget::Single { name: _ }
							| VariableTarget::MultiCell { name: _, index: _ },
							Memory::Cells {
								id: memory_id,
								len: _,
								target_index: Some(index),
							},
						) => scope.push_instruction(Instruction::InputToCell(Cell {
							memory_id,
							index: Some(index),
						})),
						(
							VariableTarget::MultiSpread { name: _ },
							Memory::Cells {
								id: memory_id,
								len,
								target_index: None,
							},
						) => {
							// run the low level input , operator once for each byte in the variable
							for i in 0..len {
								scope.push_instruction(Instruction::InputToCell(Cell {
									memory_id,
									index: Some(i),
								}));
							}
						}
						_ => r_panic!("Cannot input into variable \"{var}\""),
					}
				}
				Clause::OutputValue { value } => {
					match value {
						Expression::VariableReference(var) => {
							let mem = scope.get_memory(&var)?;
							match (&var, mem) {
								(
									VariableTarget::Single { name: _ },
									Memory::Cell { id: memory_id },
								) => scope.push_instruction(Instruction::OutputCell(Cell {
									memory_id,
									index: None,
								})),
								(
									VariableTarget::Single { name: _ }
									| VariableTarget::MultiCell { name: _, index: _ },
									Memory::Cells {
										id: memory_id,
										len: _,
										// hack
										target_index: Some(index),
									},
								) => scope.push_instruction(Instruction::OutputCell(Cell {
									memory_id,
									index: Some(index),
								})),
								(
									VariableTarget::MultiSpread { name: _ },
									Memory::Cells {
										id: memory_id,
										len,
										// hack
										target_index: None,
									},
								) => {
									// run the low level output . operator once for each byte in the variable
									for i in 0..len {
										scope.push_instruction(Instruction::OutputCell(Cell {
											memory_id,
											index: Some(i),
										}));
									}
								}
								_ => r_panic!("Cannot output variable \"{var}\""),
							}
						}
						Expression::SumExpression {
							sign: _,
							summands: _,
						}
						| Expression::NaturalNumber(_) => {
							// allocate a temporary cell and add the expression to it, output, then clear
							let temp_mem_id = scope.create_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = Cell {
								memory_id: temp_mem_id,
								index: None,
							};

							_add_expr_to_cell(&mut scope, value, cell)?;
							scope.push_instruction(Instruction::OutputCell(cell));
							scope.push_instruction(Instruction::ClearCell(cell));

							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
						Expression::ArrayLiteral(expressions) => {
							// same as above, except reuse the temporary cell after each output
							let temp_mem_id = scope.create_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = Cell {
								memory_id: temp_mem_id,
								index: None,
							};

							for value in expressions {
								_add_expr_to_cell(&mut scope, value, cell)?;
								scope.push_instruction(Instruction::OutputCell(cell));
								scope.push_instruction(Instruction::ClearCell(cell));
							}

							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
						Expression::StringLiteral(s) => {
							// same as above, allocate one temporary cell and reuse it for each character
							let temp_mem_id = scope.create_memory_id();
							scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: temp_mem_id },
								None,
							));
							let cell = Cell {
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
					let mem = scope.get_memory(&var)?;
					let cell = match var {
						VariableTarget::Single { name: _ } => Cell {
							memory_id: mem.id(),
							index: None,
						},
						VariableTarget::MultiCell { name: _, index } => Cell {
							memory_id: mem.id(),
							index: Some(index),
						},
						VariableTarget::MultiSpread { name: _ } => {
							r_panic!("Cannot open while loop on spread variable \"{var}\"")
						}
					};

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
					// TODO: refactor this drain/copy loop business
					match is_draining {
						true => {
							// draining loops can drain from an expression or a variable
							let (source_cell, free_source_cell) = match &source {
								Expression::VariableReference(var) => {
									let mem = scope.get_memory(var)?;
									match (var, mem) {
										(
											VariableTarget::Single { name: _ },
											Memory::Cell { id: memory_id },
										) => (
											Cell {
												memory_id,
												index: None,
											},
											false,
										),
										(
											VariableTarget::Single { name: _ }
											| VariableTarget::MultiCell { name: _, index: _ },
											Memory::Cells {
												id: memory_id,
												len: _,
												target_index: Some(index),
											},
										) => (
											Cell {
												memory_id,
												index: Some(index),
											},
											false,
										),
										_ => r_panic!("Cannot drain from expression {source:#?}"),
									}
								}
								_ => {
									let id = scope.create_memory_id();
									scope.push_instruction(Instruction::Allocate(
										Memory::Cell { id },
										None,
									));
									let new_cell = Cell {
										memory_id: id,
										index: None,
									};
									_add_expr_to_cell(&mut scope, source, new_cell)?;
									(new_cell, true)
								}
							};
							scope.push_instruction(Instruction::OpenLoop(source_cell));

							// recurse
							let loop_scope = self.compile(&block, Some(&scope))?;
							scope
								.instructions
								.extend(loop_scope.finalise_instructions(true));

							// copy into each target and decrement the source
							for target in targets {
								let mem = scope.get_memory(&target)?;
								match mem {
									Memory::Cell { id: memory_id } => {
										scope.push_instruction(Instruction::AddToCell(
											Cell {
												memory_id,
												index: None,
											},
											1,
										))
									}
									Memory::Cells {
										id: memory_id,
										len,
										target_index,
									} => match target_index {
										None => {
											// should only happen if the spread operator is used, ideally this should be obvious with the code, (TODO: refactor target index hack)
											for i in 0..len {
												scope.push_instruction(Instruction::AddToCell(
													Cell {
														memory_id,
														index: Some(i),
													},
													1,
												));
											}
										}
										Some(index) => {
											scope.push_instruction(Instruction::AddToCell(
												Cell {
													memory_id,
													index: Some(index),
												},
												1,
											))
										}
									},
								}
							}

							scope.push_instruction(Instruction::AddToCell(source_cell, -1i8 as u8)); // 255
							scope.push_instruction(Instruction::CloseLoop(source_cell));

							// free the source cell if it was a expression we just created
							if free_source_cell {
								scope.push_instruction(Instruction::Free(source_cell.memory_id));
							}
						}
						false => {
							let source_cell = match &source {
								Expression::VariableReference(var) => {
									let var_mem = scope.get_memory(var)?;
									let var_cell = match (var, var_mem) {
										(
											VariableTarget::Single { name: _ },
											Memory::Cell { id: memory_id },
										) => Cell {
											memory_id,
											index: None,
										},
										(
											VariableTarget::Single { name: _ }
											| VariableTarget::MultiCell { name: _, index: _ },
											Memory::Cells {
												id: memory_id,
												len: _,
												target_index: Some(index),
											},
										) => Cell {
											memory_id,
											index: Some(index),
										},
										_ => r_panic!("Cannot drain from expression {source:#?}"),
									};

									let new_mem_id = scope.create_memory_id();
									scope.push_instruction(Instruction::Allocate(
										Memory::Cell { id: new_mem_id },
										None,
									));

									let new_cell = Cell {
										memory_id: new_mem_id,
										index: None,
									};

									_copy_cell(&mut scope, var_cell, new_cell, 1);

									new_cell
								}
								_ => r_panic!(
									"Cannot copy from {source:#?}, use a drain loop instead"
								),
							};

							scope.push_instruction(Instruction::OpenLoop(source_cell));

							// recurse
							let loop_scope = self.compile(&block, Some(&scope))?;
							scope
								.instructions
								.extend(loop_scope.finalise_instructions(true));

							// copy into each target and decrement the source
							for target in targets {
								let mem = scope.get_memory(&target)?;
								match mem {
									Memory::Cell { id: memory_id } => {
										scope.push_instruction(Instruction::AddToCell(
											Cell {
												memory_id,
												index: None,
											},
											1,
										))
									}
									Memory::Cells {
										id: memory_id,
										len,
										target_index,
									} => match target_index {
										None => {
											// should only happen if the spread operator is used, ideally this should be obvious with the code, (TODO: refactor target index hack)
											for i in 0..len {
												scope.push_instruction(Instruction::AddToCell(
													Cell {
														memory_id,
														index: Some(i),
													},
													1,
												));
											}
										}
										Some(index) => {
											scope.push_instruction(Instruction::AddToCell(
												Cell {
													memory_id,
													index: Some(index),
												},
												1,
											))
										}
									},
								}
							}

							scope.push_instruction(Instruction::AddToCell(source_cell, -1i8 as u8)); // 255
							scope.push_instruction(Instruction::CloseLoop(source_cell));

							// free the temporary cell
							scope.push_instruction(Instruction::Free(source_cell.memory_id));
						}
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

					let condition_mem_id = new_scope.create_memory_id();
					new_scope.push_instruction(Instruction::Allocate(
						Memory::Cell {
							id: condition_mem_id,
						},
						None,
					));
					let condition_cell = Cell {
						memory_id: condition_mem_id,
						index: None,
					};

					let else_condition_cell = match else_block {
						Some(_) => {
							let else_mem_id = new_scope.create_memory_id();
							new_scope.push_instruction(Instruction::Allocate(
								Memory::Cell { id: else_mem_id },
								None,
							));
							let else_cell = Cell {
								memory_id: else_mem_id,
								index: None,
							};
							new_scope.push_instruction(Instruction::AddToCell(else_cell, 1));
							Some(else_cell)
						}
						None => None,
					};

					// copy the condition expression to the temporary condition cell
					_add_expr_to_cell(&mut new_scope, condition, condition_cell)?;

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
						let mem = scope.get_memory(&var)?;
						// little bit of duplicate code from the copyloop clause here:
						match mem {
							Memory::Cell { id } => {
								scope.push_instruction(Instruction::AssertCellValue(
									Cell {
										memory_id: id,
										index: None,
									},
									None,
								))
							}
							Memory::Cells {
								id,
								len,
								target_index,
							} => match target_index {
								None => {
									// should only happen if the spread operator is used, ideally this should be obvious with the code, (TODO: refactor target index hack)
									for i in 0..len {
										scope.push_instruction(Instruction::AssertCellValue(
											Cell {
												memory_id: id,
												index: Some(i),
											},
											None,
										));
									}
								}
								Some(index) => {
									scope.push_instruction(Instruction::AssertCellValue(
										Cell {
											memory_id: id,
											index: Some(index),
										},
										None,
									))
								}
							},
						}
					}
				}
				Clause::CallFunction {
					function_name,
					arguments,
				} => {
					// create variable translations and recursively compile the inner variable block
					let function_definition = scope.get_function(&function_name)?;

					let mut new_scope = scope.open_inner();
					let zipped: Result<Vec<ArgumentTranslation>, String> =
						zip(function_definition.arguments.clone().into_iter(), arguments)
							.map(|(arg_def, calling_arg)| {
								Ok(match (arg_def, calling_arg) {
									(
										VariableDefinition::Single { name: def_name },
										VariableTarget::Single { name: call_name },
									) => ArgumentTranslation::SingleFromSingle(def_name, call_name),
									(
										// this is a minor hack, the parser will parse a calling argument as a single even though it is really targeting a multi
										VariableDefinition::Multi {
											name: def_name,
											len: _,
										},
										VariableTarget::Single { name: call_name },
									) => ArgumentTranslation::MultiFromMulti(def_name, call_name),
									(
										VariableDefinition::Single { name: def_name },
										VariableTarget::MultiCell {
											name: call_name,
											index,
										},
									) => ArgumentTranslation::SingleFromMultiCell(
										def_name,
										(call_name, index),
									),
									(def_var, call_var) => {
										r_panic!(
											"Cannot translate {call_var} as argument {def_var}"
										)
									}
								})
							})
							.collect();
					new_scope.variable_aliases.extend(zipped?);

					// recurse
					let loop_scope = self.compile(&function_definition.block, Some(&new_scope))?;
					new_scope
						.instructions
						.extend(loop_scope.finalise_instructions(true));

					// extend the inner scope instructions onto the outer scope
					// maybe function call compiling should be its own function?
					scope
						.instructions
						.extend(new_scope.finalise_instructions(false));
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
fn _add_expr_to_cell(scope: &mut Scope, expr: Expression, cell: Cell) -> Result<(), String> {
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
		let Some(source_cell) = scope.get_memory(&source)?.target_cell() else {
			r_panic!("Cannot sum variable \"{source}\" in expression");
		};
		_copy_cell(scope, source_cell, cell.clone(), constant);
	}

	Ok(())
}

// another helper function to copy a cell from one to another leaving the original unaffected
fn _copy_cell(scope: &mut Scope, source_cell: Cell, target_cell: Cell, constant: i32) {
	if constant == 0 {
		return;
	}
	// allocate a temporary cell
	let temp_mem_id = scope.create_memory_id();
	scope.push_instruction(Instruction::Allocate(
		Memory::Cell { id: temp_mem_id },
		None,
	));
	let temp_cell = Cell {
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
	OpenLoop(Cell), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop(Cell), // pass in the cell id, this originally wasn't there but may be useful later on
	AddToCell(Cell, u8),
	InputToCell(Cell),
	ClearCell(Cell), // not sure if this should be here, seems common enough that it should be
	AssertCellValue(Cell, Option<u8>), // allows the user to hand-tune optimisations further
	OutputCell(Cell),
	// circular dependency here, TODO: should make an EBrainfuck OPcode object or similar for embedded mastermind
	InsertBrainfuckAtCell(Vec<Opcode>, Option<TapeCell>),
}

#[derive(Debug, Clone)]
pub enum Memory {
	Cell {
		id: MemoryId,
	},
	// this comment was originally in a different place which is why it might be a bit odd, highly relevant still
	// a little hack was added which holds the targeted cell inside the memory, this is for when you pass in a multi-byte cell reference to a function
	Cells {
		id: MemoryId,
		len: usize,
		target_index: Option<usize>,
	},
	// infinite cell something (TODO?)
}
pub type MemoryId = usize;

#[derive(Debug, Clone, Copy)]
pub struct Cell {
	pub memory_id: MemoryId,
	pub index: Option<usize>,
}

impl Memory {
	pub fn id(&self) -> MemoryId {
		match self {
			Memory::Cell { id } => *id,
			Memory::Cells {
				id,
				len: _,
				target_index: _,
			} => *id,
		}
	}
	pub fn len(&self) -> usize {
		match self {
			Memory::Cell { id: _ } => 1,
			Memory::Cells {
				id: _,
				len,
				target_index: _,
			} => *len,
		}
	}

	pub fn target_cell(&self) -> Option<Cell> {
		match self {
			Memory::Cell { id } => Some(Cell {
				memory_id: *id,
				index: None,
			}),
			Memory::Cells {
				id,
				len: _,
				target_index: Some(index),
			} => Some(Cell {
				memory_id: *id,
				index: Some(*index),
			}),
			_ => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Scope<'a> {
	outer_scope: Option<&'a Scope<'a>>,
	// syntactic context instead of normal context
	// used for embedded mm so that the inner mm can use outer functions but not variables
	fn_only: bool,

	// number of memory allocations
	allocations: usize,
	// mappings for variable names to places on above stack
	variable_memory: HashMap<VariableDefinition, MemoryId>,
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	// originally this was just string to string, but we need to be able to map a single-bit variable to a cell of an outer array variable
	variable_aliases: Vec<ArgumentTranslation>,

	// functions accessible by any code within or in the current scope
	functions: HashMap<String, Function>,

	// experimental: This is where we keep track of the instructions generated by the compiler, then we return the scope to the calling function
	instructions: Vec<Instruction>,
	// very experimental: this is used for optimisations to keep track of how variables are used
	// variable_accesses: HashMap<VariableSpec, (usize, usize)>, // (reads, writes)
}

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
// represents a position in a stack relative to the head/top

impl Scope<'_> {
	pub fn new() -> Scope<'static> {
		Scope {
			outer_scope: None,
			fn_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			variable_aliases: Vec::new(),
			functions: HashMap::new(),
			instructions: Vec::new(),
			// variable_accesses: HashMap::new(),
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
		let mut clear_instructions = Vec::new();
		for (var_def, mem_id) in self.variable_memory.iter() {
			match &var_def {
				VariableDefinition::Single { name: _ } => {
					clear_instructions.push(Instruction::ClearCell(Cell {
						memory_id: *mem_id,
						index: None,
					}))
				}
				VariableDefinition::Multi { name: _, len } => {
					for i in 0..*len {
						clear_instructions.push(Instruction::ClearCell(Cell {
							memory_id: *mem_id,
							index: Some(i),
						}))
					}
				}
			}
			clear_instructions.push(Instruction::Free(*mem_id));
		}
		for instr in clear_instructions {
			self.push_instruction(instr);
		}

		self.instructions
	}

	fn push_instruction(&mut self, instruction: Instruction) {
		self.instructions.push(instruction);
	}

	fn open_inner(&self) -> Scope {
		Scope {
			outer_scope: Some(self),
			fn_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			variable_aliases: Vec::new(),
			functions: HashMap::new(),
			instructions: Vec::new(),
			// variable_accesses: HashMap::new(),
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
			instructions: Vec::new(),
			// variable_accesses: HashMap::new(),
		}
	}

	// not sure if this function should create the allocation instruction or not
	fn allocate_variable_memory(&mut self, var: VariableDefinition) -> Result<Memory, String> {
		let id = self.create_memory_id();
		let result = Ok(match &var {
			VariableDefinition::Single { name: _ } => Memory::Cell { id },
			VariableDefinition::Multi { name: _, len } => Memory::Cells {
				id,
				len: *len,
				target_index: None,
			},
		});

		let None = self.current_level_get_variable_definition(&var.name()) else {
			r_panic!("Cannot allocate variable {var} twice in the same scope");
		};

		if let Some(var) = self.variable_memory.insert(var, id) {
			r_panic!("Unreachable error occurred when allocating {var}");
		}

		result
	}

	// fn allocate_unnamed_cell(&mut self) -> Memory {
	// 	let mem_id = self.create_memory_id();
	// 	Memory::Cell { id: mem_id }
	// }

	fn create_memory_id(&mut self) -> MemoryId {
		let current_scope_relative = self.allocations;
		self.allocations += 1;
		current_scope_relative + self.allocation_offset()
	}

	// recursively tallies the allocation stack size of the outer scope, does not include this scope
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

	fn current_level_get_variable_definition(&self, var_name: &str) -> Option<&VariableDefinition> {
		self.variable_memory
			.keys()
			.find(|var_def| var_def.name() == var_name)
	}

	// returns a memory allocation id, unfortunately we also have a little hack to add the length of the variable in here as well because we ended up needing it
	fn get_memory(&self, var: &VariableTarget) -> Result<Memory, String> {
		if let Some(var_def) = self.current_level_get_variable_definition(var.name()) {
			let Some(mem_id) = self.variable_memory.get(var_def) else {
				r_panic!("Something went wrong when compiling. This error should never occur.");
			};
			// base case, variable is defined in this scope level
			Ok(match (var_def, var) {
				(VariableDefinition::Single { name: _ }, VariableTarget::Single { name: _ }) => {
					Memory::Cell { id: *mem_id }
				}
				(
					VariableDefinition::Multi { name: _, len },
					VariableTarget::MultiCell { name: _, index },
				) => {
					r_assert!(
						*index < *len,
						"Memory access attempt: \"{var}\" out of range for variable: \"{var_def}\""
					);
					Memory::Cells {
						id: *mem_id,
						len: *len,
						target_index: Some(*index),
					}
				}
				(
					VariableDefinition::Multi { name: _, len },
					VariableTarget::MultiSpread { name: _ },
				) => Memory::Cells {
					id: *mem_id,
					len: *len,
					target_index: None,
				},
				_ => {
					r_panic!("Malformed variable reference {var} to {var_def}")
				}
			})
		} else if self.fn_only {
			r_panic!("Attempted to access variable memory outside of embedded Mastermind context.");
		} else if let Some(outer_scope) = self.outer_scope {
			// recursive case
			if let Some(translation) = self
				.variable_aliases
				.iter()
				.find(|translation| *translation.get_def_name() == *var.name())
			{
				let alias_var = match (translation, var) {
					(
						ArgumentTranslation::SingleFromSingle(_, call_name),
						VariableTarget::Single { name: _ },
						// single variable let g;f(g);def f(h){++h;}c
					) => VariableTarget::Single {
						name: call_name.clone(),
					},
					(
						ArgumentTranslation::SingleFromMultiCell(_, (call_name, call_index)),
						VariableTarget::Single { name: _ },
						// referenced byte passed as single let g[9];f(g[0]);def f(h){++h;}
					) => VariableTarget::MultiCell {
						name: call_name.clone(),
						index: *call_index,
					},
					(
						ArgumentTranslation::MultiFromMulti(_, call_name),
						VariableTarget::MultiCell { name: _, index },
						// referenced byte from multi-byte variable let g[9];f(g);def f(h[9]){++h[0];}
					) => VariableTarget::MultiCell {
						name: call_name.clone(),
						index: *index,
					},
					(
						ArgumentTranslation::MultiFromMulti(_, call_name),
						VariableTarget::MultiSpread { name: _ },
						// spread from multi-byte variable let g[9];f(g);def f(h[9]){output *h;}
					) => VariableTarget::MultiSpread {
						name: call_name.clone(),
					},
					_ => r_panic!(
						"Malformed argument/variable translation {translation:#?}, target: {var}. \
					I realise this error doesn't tell you much, this error should not occur anyway."
					),
				};
				Ok(outer_scope.get_memory(&alias_var)?)
			} else {
				// again not sure if Ok + ? is a bad pattern
				Ok(outer_scope.get_memory(var)?)
			}
		} else {
			r_panic!("No variable {var} found in current scope.");
		}
	}
}
