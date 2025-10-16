// compile syntax tree into low-level instructions

use std::{collections::HashMap, fmt::Display, iter::zip};

use crate::{
	backend::Opcode2D,
	cells::{TapeCell, TapeCell2D, TapeCellVariant},
	macros::macros::{r_assert, r_panic},
	misc::MastermindContext,
	parser::{
		Clause, Expression, ExtendedOpcode, LocationSpecifier, Reference, StructFieldDefinition,
		VariableDefinition, VariableTarget, VariableTargetReferenceChain, VariableTypeReference,
	},
};

// TODO: remove the need for this Into:
impl Into<TapeCell2D> for TapeCell {
	fn into(self) -> TapeCell2D {
		TapeCell2D(self.0, 0)
	}
}

impl MastermindContext {
	pub fn create_ir_scope<'a, TC: 'static + TapeCellVariant + Into<TapeCell2D>>(
		&self,
		clauses: &[Clause<TC>],
		outer_scope: Option<&'a ScopeBuilder<TC>>,
	) -> Result<ScopeBuilder<'a, TC>, String> {
		let mut scope = if let Some(outer) = outer_scope {
			outer.open_inner()
		} else {
			ScopeBuilder::new()
		};

		// TODO: fix unnecessary clones, and reimplement this with iterators somehow
		// hoist structs, then functions to top
		let mut filtered_clauses_1: Vec<Clause<TC>> = vec![];
		// first stage: structs (these need to be defined before functions, so they can be used as arguments)
		for clause in clauses {
			match clause {
				Clause::DefineStruct { name, fields } => {
					// convert fields with 2D or 1D location specifiers to valid struct location specifiers
					scope.register_struct_definition(name, fields.clone())?;
				}
				_ => filtered_clauses_1.push(clause.clone()),
			}
		}
		// second stage: functions
		let mut filtered_clauses_2: Vec<Clause<TC>> = vec![];
		for clause in filtered_clauses_1 {
			match clause {
				Clause::DefineFunction {
					name,
					arguments,
					block,
				} => {
					scope.register_function_definition(&name, arguments.clone(), block.clone())?;
				}
				_ => {
					filtered_clauses_2.push(clause);
				}
			}
		}

		for clause in filtered_clauses_2 {
			match clause {
				Clause::DeclareVariable { var } => {
					// create an allocation in the scope
					scope.allocate_variable(var)?;
				}
				Clause::DefineVariable { var, value } => {
					// same as above except we initialise the variable
					let absolute_type = scope.allocate_variable(var.clone())?;

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
							scope._add_expr_to_cell(&value, cell)?;
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
								scope._add_expr_to_cell(expr, cell)?;
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
				}
				Clause::SetVariable {
					var,
					value,
					self_referencing,
				} => match (var.is_spread, self_referencing) {
					(false, false) => {
						let cell = scope.get_cell(&var)?;
						scope.push_instruction(Instruction::ClearCell(cell.clone()));
						scope._add_expr_to_cell(&value, cell)?;
					}
					(false, true) => {
						let cell = scope.get_cell(&var)?;
						scope._add_self_referencing_expr_to_cell(value, cell, true)?;
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
				} => match (var.is_spread, self_referencing) {
					(false, false) => {
						let cell = scope.get_cell(&var)?;
						scope._add_expr_to_cell(&value, cell)?;
					}
					(false, true) => {
						let cell = scope.get_cell(&var)?;
						scope._add_self_referencing_expr_to_cell(value, cell, false)?;
					}
					(true, _) => {
						r_panic!("Unsupported operation, add-assigning to spread variable: {var}");
						// TODO: support spread assigns?
						// let cells = scope.get_array_cells(&var)?;
						// etc...
					}
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

					match var.is_spread {
						false => {
							let cell = scope.get_cell(&var)?;
							scope.push_instruction(Instruction::AssertCellValue(cell, imm));
						}
						true => {
							let cells = scope.get_array_cells(&var)?;
							for cell in cells {
								scope.push_instruction(Instruction::AssertCellValue(cell, imm));
							}
						}
					}
				}
				Clause::InputVariable { var } => match var.is_spread {
					false => {
						let cell = scope.get_cell(&var)?;
						scope.push_instruction(Instruction::InputToCell(cell));
					}
					true => {
						let cells = scope.get_array_cells(&var)?;
						for cell in cells {
							scope.push_instruction(Instruction::InputToCell(cell));
						}
					}
				},
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

							scope._add_expr_to_cell(&value, cell)?;
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
								scope._add_expr_to_cell(&value, cell)?;
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
					let loop_scope = self.create_ir_scope(&block, Some(&scope))?;
					scope.instructions.extend(loop_scope.build_ir(true));

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
							scope._add_expr_to_cell(&source, new_cell)?;
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

							scope._copy_cell(cell, new_cell, 1);

							(new_cell, true)
						}
						(false, _) => {
							r_panic!("Cannot copy from {source:#?}, use a drain loop instead")
						}
					};
					scope.push_instruction(Instruction::OpenLoop(source_cell));

					// recurse
					let loop_scope = self.create_ir_scope(&block, Some(&scope))?;
					// TODO: refactor, make a function in scope trait to do this automatically
					scope.instructions.extend(loop_scope.build_ir(true));

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
					new_scope._add_expr_to_cell(&condition, condition_cell)?;

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
						let if_scope = self.create_ir_scope(&block, Some(&new_scope))?;
						new_scope.instructions.extend(if_scope.build_ir(true));
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
						let else_scope = self.create_ir_scope(&block, Some(&new_scope))?;
						new_scope.instructions.extend(else_scope.build_ir(true));

						new_scope.push_instruction(Instruction::CloseLoop(cell));
						new_scope.push_instruction(Instruction::Free(cell.memory_id));
					}

					// extend the inner scopes instructions onto the outer one
					scope.instructions.extend(new_scope.build_ir(true));
				}
				Clause::Block(clauses) => {
					let new_scope = self.create_ir_scope(&clauses, Some(&scope))?;
					scope.instructions.extend(new_scope.build_ir(true));
				}
				Clause::InlineBrainfuck {
					location_specifier,
					clobbered_variables,
					operations,
				} => {
					// loop through the opcodes
					let mut expanded_bf: Vec<Opcode2D> = Vec::new();
					for op in operations {
						match op {
							ExtendedOpcode::Block(mm_clauses) => {
								// create a scope object for functions from the outside scope
								let functions_scope = scope.open_inner_templates_only();
								// compile the block and extend the operations
								let instructions = self
									.create_ir_scope(&mm_clauses, Some(&functions_scope))?
									// compile without cleaning up top level variables, this is the brainfuck programmer's responsibility
									.build_ir(false);

								// it is also the brainfuck programmer's responsibility to return to the start position
								let bf_code =
									self.ir_to_bf(instructions, Some(TC::origin_cell()))?;
								expanded_bf.extend(bf_code);
							}
							ExtendedOpcode::Add => expanded_bf.push(Opcode2D::Add),
							ExtendedOpcode::Subtract => expanded_bf.push(Opcode2D::Subtract),
							ExtendedOpcode::Right => expanded_bf.push(Opcode2D::Right),
							ExtendedOpcode::Left => expanded_bf.push(Opcode2D::Left),
							ExtendedOpcode::OpenLoop => expanded_bf.push(Opcode2D::OpenLoop),
							ExtendedOpcode::CloseLoop => expanded_bf.push(Opcode2D::CloseLoop),
							ExtendedOpcode::Output => expanded_bf.push(Opcode2D::Output),
							ExtendedOpcode::Input => expanded_bf.push(Opcode2D::Input),
							ExtendedOpcode::Up => expanded_bf.push(Opcode2D::Up),
							ExtendedOpcode::Down => expanded_bf.push(Opcode2D::Down),
						}
					}

					// handle the location specifier
					let location = match location_specifier {
						LocationSpecifier::None => CellLocation::Unspecified,
						LocationSpecifier::Cell(cell) => CellLocation::FixedCell(cell),
						LocationSpecifier::Variable(var) => {
							CellLocation::MemoryCell(scope.get_target_cell_reference(&var)?)
						}
					};

					scope.push_instruction(Instruction::InsertBrainfuckAtCell(
						expanded_bf,
						location,
					));
					// assert that we clobbered the variables
					// not sure whether this should go before or after the actual bf code
					for var in clobbered_variables {
						match var.is_spread {
							false => {
								let cell = scope.get_cell(&var)?;
								scope.push_instruction(Instruction::AssertCellValue(cell, None));
							}
							true => {
								let cells = scope.get_array_cells(&var)?;
								for cell in cells {
									scope
										.push_instruction(Instruction::AssertCellValue(cell, None));
								}
							}
						}
					}
				}
				Clause::CallFunction {
					function_name,
					arguments,
				} => {
					// create variable translations and recursively compile the inner variable block

					let calling_argument_types = arguments
						.iter()
						.map(|a| scope.get_target_type(&a))
						.collect::<Result<Vec<_>, _>>()?;

					let function_definition =
						scope.get_function(&function_name, &calling_argument_types)?;

					let mut argument_translation_scope = scope.open_inner();

					// TODO: refactor this mess
					// deal with argument memory mappings:
					for ((calling_argument, calling_argument_type), (arg_name, expected_type)) in
						zip(
							zip(arguments, calling_argument_types),
							function_definition.arguments.iter(),
						) {
						// TODO: fix this duplicate call, get_target_type() internally gets the memory allocation details
						// 	then these are gotten again in create_mapped_variable()
						r_assert!(calling_argument_type == expected_type, "Expected argument of type \"{expected_type:#?}\" in function call \"{function_name}\", received argument of type \"{calling_argument_type:#?}\". This should not occur");
						// register an argument translation in the scope
						argument_translation_scope
							.create_mapped_variable(arg_name.clone(), &calling_argument)?;
					}

					// recurse
					let function_scope = self.create_ir_scope(
						&function_definition.block,
						Some(&argument_translation_scope),
					)?;
					argument_translation_scope
						.instructions
						.extend(function_scope.build_ir(true));

					// extend the inner scope instructions onto the outer scope
					// maybe function call compiling should be its own function?
					scope
						.instructions
						.extend(argument_translation_scope.build_ir(false));
				}
				Clause::DefineStruct { name: _, fields: _ }
				| Clause::DefineFunction {
					name: _,
					arguments: _,
					block: _,
				} => unreachable!(),
			}
		}

		Ok(scope)
	}
}

// this is subject to change
#[derive(Debug, Clone)]
pub enum Instruction<TC> {
	Allocate(Memory, Option<TC>),
	Free(MemoryId), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(CellReference), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop(CellReference), // pass in the cell id, this originally wasn't there but may be useful later on
	AddToCell(CellReference, u8),
	InputToCell(CellReference),
	ClearCell(CellReference), // not sure if this should be here, seems common enough that it should be
	AssertCellValue(CellReference, Option<u8>), // allows the user to hand-tune optimisations further
	OutputCell(CellReference),
	InsertBrainfuckAtCell(Vec<Opcode2D>, CellLocation<TC>),
}

#[derive(Debug, Clone)]
/// Either a fixed constant cell or a reference to some existing memory
pub enum CellLocation<TC> {
	Unspecified,
	FixedCell(TC),
	MemoryCell(CellReference),
}

#[derive(Debug, Clone)]
pub enum Memory {
	Cell {
		id: MemoryId,
	},
	Cells {
		id: MemoryId,
		len: usize,
	},
	/// A memory cell that references a previously allocated cell in an outer scope, used for function arguments
	MappedCell {
		id: MemoryId,
		index: Option<usize>,
	},
	/// Memory mapped cells, referencing previously allocated cells in an outer scope
	MappedCells {
		id: MemoryId,
		start_index: usize,
		len: usize,
	},
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
			Memory::Cell { id }
			| Memory::Cells { id, len: _ }
			| Memory::MappedCell { id, index: _ }
			| Memory::MappedCells {
				id,
				start_index: _,
				len: _,
			} => *id,
		}
	}
	pub fn len(&self) -> usize {
		match self {
			Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ } => 1,
			Memory::Cells { id: _, len }
			| Memory::MappedCells {
				id: _,
				start_index: _,
				len,
			} => *len,
		}
	}
}

#[derive(Clone, Debug)]
/// Scope type represents a Mastermind code block,
/// any variables or functions defined within a {block} are owned by the scope and cleaned up before continuing
pub struct ScopeBuilder<'a, TapeCell> {
	/// a reference to the parent scope, for accessing things defined outside of this scope
	outer_scope: Option<&'a ScopeBuilder<'a, TapeCell>>,
	/// fn_only: true if syntactic context instead of normal context.
	/// Used for embedded mm so that the inner mm can use outer functions but not variables.
	types_only: bool,

	/// Number of memory allocations in current scope
	allocations: usize,

	/// Mappings for variable names to memory allocation IDs in current scope
	variable_memory: HashMap<String, (ValueType, Memory)>,

	/// Functions accessible by any code within or in the current scope
	functions: Vec<(String, Vec<(String, ValueType)>, Vec<Clause<TapeCell>>)>,
	/// Struct types definitions
	structs: HashMap<String, DictStructType>,

	/// Intermediate instructions generated by the compiler
	instructions: Vec<Instruction<TapeCell>>,
}

#[derive(Clone, Debug)] // probably shouldn't be cloning here but whatever
struct Function<TC> {
	arguments: Vec<(String, ValueType)>,
	block: Vec<Clause<TC>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// an absolute definition of a type, as opposed to `VariableTypeReference` which is more of a reference
enum ValueType {
	Cell,
	Array(usize, Box<ValueType>),
	DictStruct(Vec<(String, ValueType, Option<usize>)>),
	// TupleStruct(Vec<ValueType>),
}

#[derive(Clone, Debug)]
/// equivalent to ValueType::DictStruct enum variant,
/// Rust doesn't support enum variants as types yet so need this workaround for struct definitions in scope object
struct DictStructType(Vec<(String, ValueType, Option<usize>)>);
impl ValueType {
	fn from_struct(struct_def: DictStructType) -> Self {
		ValueType::DictStruct(struct_def.0)
	}

	// TODO: make size() and get_and_validate_subfield_cell_map() more efficient,
	//  currently these two recurse back and forth and are a bit of a monster combo

	/// return the type size in cells
	fn size(&self) -> Result<usize, String> {
		Ok(match self {
			ValueType::Cell => 1,
			ValueType::Array(len, value_type) => *len * value_type.size()?,
			ValueType::DictStruct(fields) => Self::get_and_validate_subfield_cell_map(fields)?.1,
		})
	}

	/// deterministically place all struct subfields on a non-negative cell, return the positions of each and the total length
	/// return Err() if location specified subfields overlap
	fn get_and_validate_subfield_cell_map(
		fields: &Vec<(String, ValueType, Option<usize>)>,
	) -> Result<(HashMap<&String, (usize, &ValueType)>, usize), String> {
		// (set of cells, max cell)
		let mut cell_map = HashMap::new();

		// map of field names and their starting cells
		let mut subfield_map = HashMap::new();
		let mut max_cell = 0usize;
		let mut unfixed_fields = vec![];
		// handle the cells with specified locations
		for (field_name, field_type, field_location) in fields {
			match field_location {
				Some(location) => {
					subfield_map.insert(field_name, (*location, field_type));
					for cell_index in *location..(*location + field_type.size()?) {
						// this assumes the field locations have been validated
						if let Some(other_name) = cell_map.insert(cell_index, field_name) {
							r_panic!(
									"Subfields \"{other_name}\" and \"{field_name}\" overlap in struct."
								);
						};
						max_cell = max_cell.max(cell_index);
					}
				}
				None => {
					unfixed_fields.push((field_name, field_type));
				}
			}
		}

		for (field_name, field_type) in unfixed_fields {
			let field_size = field_type.size()?;
			// repeatedly try to insert the fields into leftover memory locations
			let mut start_index = 0usize;
			for cur_index in 0.. {
				if cell_map.contains_key(&cur_index) {
					start_index = cur_index + 1;
				} else if (cur_index - start_index + 1) >= field_size {
					// found a run with the right amount of cells free
					break;
				}
			}
			subfield_map.insert(field_name, (start_index, field_type));
			for cell_index in start_index..(start_index + field_size) {
				// inefficient but whatever, this insert is not necessary
				cell_map.insert(cell_index, field_name);
				max_cell = max_cell.max(cell_index);
			}
		}

		let size = max_cell + 1;

		Ok((subfield_map, size))
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
					cur_index += element_type.size()? * index;
					cur_field = element_type;
				}
				(ValueType::DictStruct(fields), Reference::NamedField(subfield_name)) => {
					let (subfield_map, _size) = Self::get_and_validate_subfield_cell_map(fields)?;
					let Some((subfield_cell_offset, subfield_type)) =
						subfield_map.get(subfield_name)
					else {
						r_panic!("Could not find subfield \"{subfield_ref}\" in struct type")
					};
					cur_index += subfield_cell_offset;
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

impl<TC> ScopeBuilder<'_, TC>
where
	TC: Display + Clone,
{
	pub fn new() -> ScopeBuilder<'static, TC> {
		ScopeBuilder {
			outer_scope: None,
			types_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			functions: Vec::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	// I don't love this system of deciding what to clean up at the end in this specific function, but I'm not sure what the best way to achieve this would be
	// this used to be called "get_instructions" but I think this more implies things are being modified
	pub fn build_ir(mut self, clean_up_variables: bool) -> Vec<Instruction<TC>> {
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
					}));
					clear_instructions.push(Instruction::Free(*id));
				}
				Memory::Cells { id, len } => {
					for i in 0..*len {
						clear_instructions.push(Instruction::ClearCell(CellReference {
							memory_id: *id,
							index: Some(i),
						}))
					}
					clear_instructions.push(Instruction::Free(*id));
				}
				Memory::MappedCell { id: _, index: _ }
				| Memory::MappedCells {
					id: _,
					start_index: _,
					len: _,
				} => (),
			}
		}
		for instr in clear_instructions {
			self.push_instruction(instr);
		}

		self.instructions
	}

	fn push_instruction(&mut self, instruction: Instruction<TC>) {
		self.instructions.push(instruction);
	}

	/// Open a scope within the current one, any time there is a {} in Mastermind, this is called
	fn open_inner(&self) -> ScopeBuilder<TC> {
		ScopeBuilder {
			outer_scope: Some(self),
			types_only: false,
			allocations: 0,
			variable_memory: HashMap::new(),
			functions: Vec::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	// syntactic context instead of normal context
	// used for embedded mm so that the inner mm can use outer functions
	fn open_inner_templates_only(&self) -> ScopeBuilder<TC> {
		ScopeBuilder {
			outer_scope: Some(self),
			types_only: true,
			allocations: 0,
			variable_memory: HashMap::new(),
			functions: Vec::new(),
			structs: HashMap::new(),
			instructions: Vec::new(),
		}
	}

	/// Get the correct variable type and allocate the right amount of cells for it
	fn allocate_variable(&mut self, var: VariableDefinition<TC>) -> Result<&ValueType, String> {
		r_assert!(
			!self.variable_memory.contains_key(&var.name),
			"Cannot allocate variable {var} twice in the same scope"
		);

		// get absolute type
		let full_type = self.create_absolute_type(&var.var_type)?;
		// get absolute type size
		let id = self.push_memory_id();
		let memory = match &full_type {
			ValueType::Cell => Memory::Cell { id },
			_ => Memory::Cells {
				id,
				len: full_type.size()?,
			},
		};
		// save variable in scope memory
		let None = self
			.variable_memory
			.insert(var.name.clone(), (full_type, memory.clone()))
		else {
			r_panic!("Unreachable error occurred when allocating {var}");
		};

		// verify location specifier
		let location = match var.location_specifier {
			LocationSpecifier::None => None,
			LocationSpecifier::Cell(cell) => Some(cell),
			LocationSpecifier::Variable(_) => r_panic!(
				"Cannot use variable as location specifier target when allocating variable: {var}"
			),
		};

		// allocate
		self.push_instruction(Instruction::Allocate(memory.clone(), location));

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
		if self.types_only {
			return 0;
		}
		if let Some(outer_scope) = self.outer_scope {
			outer_scope.allocations + outer_scope.allocation_offset()
		} else {
			0
		}
	}

	fn get_function(
		&self,
		calling_name: &str,
		calling_arg_types: &Vec<&ValueType>,
	) -> Result<Function<TC>, String> {
		// this function is unaffected by the self.fn_only flag
		Ok(
			if let Some(func) = self.functions.iter().find(|(name, args, _)| {
				if name != calling_name || args.len() != calling_arg_types.len() {
					return false;
				}
				for ((_, arg_type), calling_arg_type) in zip(args, calling_arg_types) {
					if *arg_type != **calling_arg_type {
						return false;
					}
				}
				true
			}) {
				// TODO: stop cloning! This function overload stuff is tacked on and needs refactoring
				let (_, arguments, block) = func;
				Function {
					arguments: arguments.clone(),
					block: block.clone(),
				}
			} else if let Some(outer_scope) = self.outer_scope {
				outer_scope.get_function(calling_name, calling_arg_types)?
			} else {
				r_panic!("Could not find function \"{calling_name}\" with correct arguments in current scope");
			},
		)
	}

	/// Define a struct in this scope
	fn register_struct_definition(
		&mut self,
		struct_name: &str,
		fields: Vec<StructFieldDefinition>,
	) -> Result<(), String> {
		let mut absolute_fields = vec![];

		for field_def in fields {
			let absolute_type = self.create_absolute_type(&field_def.field_type)?;
			absolute_fields.push((
				field_def.name,
				absolute_type,
				field_def.location_offset_specifier,
			));
		}

		let None = self
			.structs
			.insert(struct_name.to_string(), DictStructType(absolute_fields))
		else {
			r_panic!("Cannot define struct {struct_name} more than once in same scope.");
		};

		Ok(())
	}

	/// Define a function in this scope
	fn register_function_definition(
		&mut self,
		new_function_name: &str,
		new_arguments: Vec<VariableDefinition<TC>>,
		new_block: Vec<Clause<TC>>,
	) -> Result<(), String> {
		let absolute_arguments = new_arguments
			.into_iter()
			.map(|f| {
				let LocationSpecifier::None = f.location_specifier else {
					r_panic!("Cannot specify variable location in function argument \"{f}\".");
				};
				Ok((f.name, self.create_absolute_type(&f.var_type)?))
			})
			.collect::<Result<Vec<_>, _>>()?;

		// This is some fucked C-style loop break logic, basically GOTOs
		// basically it only gets to the panic if the functions have identical signature (except argument names)
		'func_loop: for (name, args, _) in self.functions.iter() {
			if name != new_function_name || args.len() != absolute_arguments.len() {
				continue;
			}
			for ((_, new_arg_type), (_, arg_type)) in zip(&absolute_arguments, args) {
				if *new_arg_type != *arg_type {
					// early continue if any of the arguments are different type
					continue 'func_loop;
				}
			}
			r_panic!("Cannot define a function with the same signature more than once in the same scope: \"{new_function_name}\"");
		}

		self.functions
			.push((new_function_name.to_string(), absolute_arguments, new_block));

		Ok(())
	}

	/// Recursively find the definition of a struct type by searching up the scope call stack
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

	/// Construct an absolute type from a type reference
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

	/// Return a cell reference for a variable target
	fn get_cell(&self, target: &VariableTarget) -> Result<CellReference, String> {
		// get the absolute type of the variable, as well as the memory allocations
		let (full_type, memory) = self.get_base_variable_memory(&target.name)?;
		// get the correct index within the memory and return
		Ok(match (&target.subfields, full_type, memory) {
			(None, ValueType::Cell, Memory::Cell { id }) => CellReference {
				memory_id: *id,
				index: None,
			},
			(None, ValueType::Cell, Memory::MappedCell { id, index }) => CellReference {
				memory_id: *id,
				index: *index,
			},
			(
				Some(subfield_chain),
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cells { id, len }
				| Memory::MappedCells {
					id,
					start_index: _,
					len,
				},
			) => {
				let (subfield_type, cell_index) = full_type.get_subfield(&subfield_chain)?;
				let ValueType::Cell = subfield_type else {
					r_panic!("Expected cell type in variable target: {target}");
				};
				r_assert!(cell_index < *len, "Cell reference out of bounds on variable target: {target}. This should not occur.");
				CellReference {
					memory_id: *id,
					index: Some(match memory {
						Memory::Cells { id: _, len: _ } => cell_index,
						Memory::MappedCells {
							id: _,
							start_index,
							len: _,
						} => *start_index + cell_index,
						_ => unreachable!(),
					}),
				}
			}
			// valid states, user error
			(
				Some(_),
				ValueType::Cell,
				Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ },
			) => r_panic!("Cannot get subfields of cell type: {target}"),
			(
				None,
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cells { id: _, len: _ }
				| Memory::MappedCells {
					id: _,
					start_index: _,
					len: _,
				},
			) => r_panic!("Expected single cell reference in target: {target}"),
			// invalid states, indicating an internal compiler issue (akin to 5xx error)
			(
				_,
				ValueType::Cell,
				Memory::Cells { id: _, len: _ }
				| Memory::MappedCells {
					id: _,
					start_index: _,
					len: _,
				},
			)
			| (
				_,
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ },
			) => r_panic!(
				"Invalid memory for value type in target: {target}. This should not occur."
			),
		})
	}

	/// Return a list of cell references for an array of cells (not an array of structs)
	fn get_array_cells(&self, target: &VariableTarget) -> Result<Vec<CellReference>, String> {
		let (full_type, memory) = self.get_base_variable_memory(&target.name)?;
		Ok(match (&target.subfields, full_type, memory) {
			(
				None,
				ValueType::Array(arr_len, element_type),
				Memory::Cells { id, len }
				| Memory::MappedCells {
					id,
					start_index: _,
					len,
				},
			) => {
				let ValueType::Cell = **element_type else {
					r_panic!("Cannot get array cells of struct array: {target}");
				};
				r_assert!(
					*arr_len == *len,
					"Array memory incorrect length {len} for array length {arr_len}."
				);
				(match memory {
					Memory::Cells { id: _, len } => 0..*len,
					Memory::MappedCells {
						id: _,
						start_index,
						len,
					} => *start_index..(*start_index + *len),
					_ => unreachable!(),
				})
				.map(|i| CellReference {
					memory_id: *id,
					index: Some(i),
				})
				.collect()
			}
			(
				Some(subfields),
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cells { id, len: _ }
				| Memory::MappedCells {
					id,
					start_index: _,
					len: _,
				},
			) => {
				let (subfield_type, offset_index) = full_type.get_subfield(subfields)?;
				let ValueType::Array(arr_len, element_type) = subfield_type else {
					r_panic!("Expected array type in subfield variable target \"{target}\".");
				};
				let ValueType::Cell = **element_type else {
					r_panic!("Expected cell array in subfield variable target \"{target}\".");
				};

				(match memory {
					Memory::Cells { id: _, len: _ } => offset_index..(offset_index + *arr_len),
					Memory::MappedCells {
						id: _,
						start_index,
						len: _,
					} => (*start_index + offset_index)..(*start_index + offset_index + *arr_len),
					_ => unreachable!(),
				})
				.map(|i| CellReference {
					memory_id: *id,
					index: Some(i),
				})
				.collect()
			}
			(
				None,
				ValueType::DictStruct(_),
				Memory::Cells { id: _, len: _ }
				| Memory::MappedCells {
					id: _,
					start_index: _,
					len: _,
				},
			)
			| (
				None,
				ValueType::Cell,
				Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ },
			) => {
				r_panic!("Expected cell array type in variable target: {target}")
			}
			(
				Some(_),
				ValueType::Cell,
				Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ },
			) => {
				r_panic!("Attempted to retrieve array subfield from cell type: {target}")
			}
			(
				_,
				ValueType::Cell,
				Memory::Cells { id: _, len: _ }
				| Memory::MappedCells {
					id: _,
					start_index: _,
					len: _,
				},
			)
			| (
				_,
				ValueType::Array(_, _) | ValueType::DictStruct(_),
				Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ },
			) => r_panic!(
				"Invalid memory for value type in target: {target}. This should not occur."
			),
		})
	}

	/// Return the first memory cell of a target allocation, used for location specifiers
	fn get_target_cell_reference(&self, target: &VariableTarget) -> Result<CellReference, String> {
		let (full_type, memory) = self.get_base_variable_memory(&target.name)?;
		Ok(match &target.subfields {
			None => match memory {
				Memory::Cell { id } => CellReference {
					memory_id: *id,
					index: None,
				},
				Memory::MappedCell { id, index } => CellReference {
					memory_id: *id,
					index: *index,
				},
				Memory::Cells { id, len: _ } => CellReference {
					memory_id: *id,
					index: Some(0),
				},
				Memory::MappedCells {
					id,
					start_index,
					len: _,
				} => CellReference {
					memory_id: *id,
					index: Some(*start_index),
				},
			},
			Some(subfield_chain) => {
				let (_subfield_type, offset_index) = full_type.get_subfield(&subfield_chain)?;
				match memory {
					   Memory::Cell { id: _ } |Memory::MappedCell { id: _, index: _ } => r_panic!("Attempted to get cell reference of subfield of single cell memory: {target}"),  
					Memory::Cells { id, len } | Memory::MappedCells { id, start_index: _, len } => {
						r_assert!(offset_index < *len, "Subfield memory index out of allocation range. This should not occur. ({target})");
						let index = match memory {
							Memory::Cells { id: _, len: _ } => offset_index,
							Memory::MappedCells { id: _, start_index, len: _ } => *start_index + offset_index,
										Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ } => unreachable!()
								};
						CellReference {memory_id: *id, index: Some(index)}
					}
				}
			}
		})
	}

	/// Return the absolute type and memory allocation for a variable name
	fn get_base_variable_memory(&self, var_name: &str) -> Result<(&ValueType, &Memory), String> {
		match (self.outer_scope, self.variable_memory.get(var_name)) {
			(_, Some((value_type, memory))) => Ok((value_type, memory)),
			(Some(outer_scope), None) => outer_scope.get_base_variable_memory(var_name),
			(None, None) => r_panic!("No variable found with name \"{var_name}\"."),
		}
	}

	/// Get the absolute type of a full variable target, not just a name like `get_base_variable_memory`
	fn get_target_type(&self, target: &VariableTarget) -> Result<&ValueType, String> {
		let (var_type, _memory) = self.get_base_variable_memory(&target.name)?;
		Ok(match &target.subfields {
			None => var_type,
			Some(subfields) => {
				let (subfield_type, _memory_index) = var_type.get_subfield(subfields)?;
				subfield_type
			}
		})
	}

	/// Create memory mapping between a pre-existing variable and a new one, used for function arguments
	fn create_mapped_variable(
		&mut self,
		mapped_var_name: String,
		target: &VariableTarget,
	) -> Result<(), String> {
		let (base_var_type, base_var_memory) = self.get_base_variable_memory(&target.name)?;
		let (var_type, mapped_memory) = match &target.subfields {
			None => (
				base_var_type,
				match base_var_memory {
					Memory::Cell { id } => Memory::MappedCell {
						id: *id,
						index: None,
					},
					Memory::Cells { id, len } => Memory::MappedCells {
						id: *id,
						start_index: 0,
						len: *len,
					},
					Memory::MappedCell { id, index } => Memory::MappedCell {
						id: *id,
						index: *index,
					},
					Memory::MappedCells {
						id,
						start_index,
						len,
					} => Memory::MappedCells {
						id: *id,
						start_index: *start_index,
						len: *len,
					},
				},
			),
			Some(subfields) => {
				let (subfield_type, offset_index) = base_var_type.get_subfield(subfields)?;
				// let subfield_size = subfield_type.size();
				(
					subfield_type,
					match (subfield_type, base_var_memory) {
						(ValueType::Cell, Memory::Cells { id, len: _ }) => {
							// r_assert!((offset_index + subfield_size) <= *len, "Subfield \"{target}\" size and offset out of memory bounds. This should never occur.");
							Memory::MappedCell {
								id: *id,
								index: Some(offset_index),
							}
						}
						(
							ValueType::Cell,
							Memory::MappedCells {
								id,
								start_index,
								len: _,
							},
						) => Memory::MappedCell {
							id: *id,
							index: Some(*start_index + offset_index),
						},
						(
							ValueType::Array(_, _) | ValueType::DictStruct(_),
							Memory::Cells { id, len: _ },
						) => Memory::MappedCells {
							id: *id,
							start_index: offset_index,
							len: subfield_type.size()?,
						},
						(
							ValueType::Array(_, _) | ValueType::DictStruct(_),
							Memory::MappedCells {
								id,
								start_index,
								len: _,
							},
						) => Memory::MappedCells {
							id: *id,
							start_index: *start_index + offset_index,
							len: subfield_type.size()?,
						},
						(_, Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ }) => {
							r_panic!(
								"Attempted to map a subfield of a single cell in \
mapping: {mapped_var_name} -> {target}"
							)
						}
					},
				)
			}
		};

		self.variable_memory
			.insert(mapped_var_name, (var_type.clone(), mapped_memory));
		Ok(())
	}

	/// helper function for a common use-case:
	/// flatten an expression and add it to a specific cell (using copies and adds, etc)
	fn _add_expr_to_cell(&mut self, expr: &Expression, cell: CellReference) -> Result<(), String> {
		let (imm, adds, subs) = expr.flatten()?;

		self.push_instruction(Instruction::AddToCell(cell.clone(), imm));

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
			let source_cell = self.get_cell(&source)?;
			self._copy_cell(source_cell, cell.clone(), constant);
		}

		Ok(())
	}

	/// helper function to add a self-referencing expression to a cell
	/// this is separated because it requires another copy ontop of normal expressions
	// TODO: refactor/fix underlying logic for this
	fn _add_self_referencing_expr_to_cell(
		&mut self,
		expr: Expression,
		cell: CellReference,
		pre_clear: bool,
	) -> Result<(), String> {
		//Create a new temp cell to store the current cell value
		let temp_mem_id = self.push_memory_id();
		self.push_instruction(Instruction::Allocate(
			Memory::Cell { id: temp_mem_id },
			None,
		));
		let temp_cell = CellReference {
			memory_id: temp_mem_id,
			index: None,
		};
		// TODO: make this more efficent by not requiring a clear cell after,
		// i.e. simple move instead of copy by default for set operations (instead of +=)
		self._copy_cell(cell, temp_cell, 1);
		// Then if we are doing a += don't pre-clear otherwise Clear the current cell and run the same actions as _add_expr_to_cell
		if pre_clear {
			self.push_instruction(Instruction::ClearCell(cell.clone()));
		}

		let (imm, adds, subs) = expr.flatten()?;

		self.push_instruction(Instruction::AddToCell(cell.clone(), imm));

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
			let source_cell = self.get_cell(&source)?;
			//If we have an instance of the original cell being added simply use our temp cell value
			// (crucial special sauce)
			if source_cell.memory_id == cell.memory_id && source_cell.index == cell.index {
				self._copy_cell(temp_cell, cell.clone(), constant);
			} else {
				self._copy_cell(source_cell, cell.clone(), constant);
			}
		}
		//Cleanup
		self.push_instruction(Instruction::ClearCell(temp_cell));
		self.push_instruction(Instruction::Free(temp_mem_id));

		Ok(())
	}

	/// Helper function to copy a cell from one to another, leaving the original unaffected
	// TODO: make one for draining a cell
	fn _copy_cell(
		&mut self,
		source_cell: CellReference,
		target_cell: CellReference,
		constant: i32,
	) {
		if constant == 0 {
			return;
		}
		// allocate a temporary cell
		let temp_mem_id = self.push_memory_id();
		self.push_instruction(Instruction::Allocate(
			Memory::Cell { id: temp_mem_id },
			None,
		));
		let temp_cell = CellReference {
			memory_id: temp_mem_id,
			index: None,
		};
		// copy source to target and temp
		self.push_instruction(Instruction::OpenLoop(source_cell));
		self.push_instruction(Instruction::AddToCell(target_cell, constant as u8));
		self.push_instruction(Instruction::AddToCell(temp_cell, 1));
		self.push_instruction(Instruction::AddToCell(source_cell, -1i8 as u8));
		self.push_instruction(Instruction::CloseLoop(source_cell));
		// copy back from temp
		self.push_instruction(Instruction::OpenLoop(temp_cell));
		self.push_instruction(Instruction::AddToCell(source_cell, 1));
		self.push_instruction(Instruction::AddToCell(temp_cell, -1i8 as u8));
		self.push_instruction(Instruction::CloseLoop(temp_cell));
		self.push_instruction(Instruction::Free(temp_mem_id));
	}
}
