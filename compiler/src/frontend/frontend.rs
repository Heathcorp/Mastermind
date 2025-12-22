// compile syntax tree into low-level instructions

use std::{collections::HashMap, fmt::Display, iter::zip};

use super::types::*;
use crate::{
	backend::common::{
		BrainfuckBuilder, BrainfuckBuilderData, CellAllocator, CellAllocatorData, OpcodeVariant,
		TapeCellVariant,
	},
	macros::macros::*,
	misc::MastermindContext,
	parser::{
		expressions::Expression,
		types::{
			Clause, ExtendedOpcode, LocationSpecifier, Reference, StructFieldTypeDefinition,
			VariableTarget, VariableTargetReferenceChain, VariableTypeDefinition,
			VariableTypeReference,
		},
	},
};

impl MastermindContext {
	pub fn create_ir_scope<'a, TC: 'static + TapeCellVariant, OC: 'static + OpcodeVariant>(
		&self,
		clauses: &[Clause<TC, OC>],
		outer_scope: Option<&'a ScopeBuilder<TC, OC>>,
	) -> Result<ScopeBuilder<'a, TC, OC>, String>
	where
	BrainfuckBuilderData<TC, OC>: BrainfuckBuilder<TC, OC>,
	CellAllocatorData<TC>: CellAllocator<TC>,
	{
		let mut scope = if let Some(outer) = outer_scope {
			outer.open_inner()
		} else {
			ScopeBuilder::new()
		};
		
		// TODO: fix unnecessary clones, and reimplement this with iterators somehow
		// hoist structs, then functions to top
		let mut filtered_clauses_1 = vec![];
		// first stage: structs (these need to be defined before functions, so they can be used as arguments)
		for clause in clauses {
			match clause {
				Clause::DefineStruct { name, fields } => {
					// convert fields with 2D or 1D location specifiers to valid struct location specifiers
					scope.register_struct_definition(name, fields.clone())?;
				}
				// also filter out None clauses (although there shouldn't be any)
				Clause::None => (),
				_ => filtered_clauses_1.push(clause.clone()),
			}
		}
		// second stage: functions
		let mut filtered_clauses_2 = vec![];
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
				Clause::Assign {
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
				Clause::AddAssign {
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
				Clause::Input { var } => match var.is_spread {
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
				Clause::Output { value } => {
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
							
							let mut prev = 0;
							for c in s.bytes() {
								scope.push_instruction(Instruction::AddToCell(cell, c.wrapping_sub(prev)));
								scope.push_instruction(Instruction::OutputCell(cell));
								prev = c;
							}
							scope.push_instruction(Instruction::ClearCell(cell));
							scope.push_instruction(Instruction::Free(temp_mem_id));
						}
					}
				}
				Clause::While { var, block } => {
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
				Clause::DrainLoop {
					source,
					targets,
					block,
					is_copying,
				} => {
					// TODO: refactor this, there is duplicate code with copying the source value cell
					let (source_cell, free_source_cell) = match (is_copying, &source) {
						// draining loops can drain from an expression or a variable
						(false, Expression::VariableReference(var)) => {
							(scope.get_cell(var)?, false)
						}
						(false, _) => {
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
						(true, Expression::VariableReference(var)) => {
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
						(true, _) => {
							r_panic!("Cannot copy from {source:#?}, use a drain loop instead")
						}
					};
					scope.push_instruction(Instruction::OpenLoop(source_cell));
					
					// recurse
					if let Some(block) = block {
						let loop_scope = self.create_ir_scope(&block, Some(&scope))?;
						// TODO: refactor, make a function in scope trait to do this automatically
						scope.instructions.extend(loop_scope.build_ir(true));
					}
					
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
				clause @ (Clause::If {
					condition: _,
					if_block: _,
				}
				| Clause::IfNot {
					condition: _,
					if_not_block: _,
				}
				| Clause::IfElse {
					condition: _,
					if_block: _,
					else_block: _,
				}
				| Clause::IfNotElse {
					condition: _,
					if_not_block: _,
					else_block: _,
				}) => {
					// If-else clause types changed recently, so here is a patch to keep the original frontend code:
					let (condition, if_block, else_block) = match clause {
						Clause::If {
							condition,
							if_block,
						} => (condition, Some(if_block), None),
						Clause::IfNot {
							condition,
							if_not_block,
						} => (condition, None, Some(if_not_block)),
						Clause::IfElse {
							condition,
							if_block,
							else_block,
						} => (condition, Some(if_block), Some(else_block)),
						Clause::IfNotElse {
							condition,
							if_not_block,
							else_block,
						} => (condition, Some(else_block), Some(if_not_block)),
						_ => unreachable!(),
					};
					// end patch //
					
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
				Clause::Brainfuck {
					location_specifier,
					clobbered_variables,
					operations,
				} => {
					// loop through the opcodes
					let mut expanded_bf: Vec<OC> = Vec::new();
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
							ExtendedOpcode::Opcode(opcode) => expanded_bf.push(opcode),
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
					
					// get the calling arguments' types
					let calling_argument_types: Vec<ValueType> = arguments
					.iter()
					.map(|arg| scope.get_expression_type(arg))
					.collect::<Result<Vec<ValueType>, String>>()?;
					
					// find the function based on name * types
					let function_definition =
					scope.get_function(&function_name, &calling_argument_types)?;
					
					// create mappings in a new translation scope, so mappings will be removed once scope closes
					let mut argument_translation_scope = scope.open_inner();
					assert_eq!(arguments.len(), function_definition.arguments.len());
					for (calling_expr, (arg_name, _)) in
						zip(arguments, function_definition.arguments)
						{
							// TODO: allow expressions as arguments: create a new variable instead of mapping when a value needs to be computed
							let calling_arg = match calling_expr {
								Expression::VariableReference(var) => var,
								expr => r_panic!(
									"Expected variable target in function call argument, \
found expression `{expr}`. General expressions as \
function arguments are not supported."
								),
							};
							argument_translation_scope
							.create_mapped_variable(arg_name, &calling_arg)?;
						}
						
						// recursively compile the function block
						let function_scope = self.create_ir_scope(
							&function_definition.block,
							Some(&argument_translation_scope),
						)?;
						argument_translation_scope
						.instructions
						.extend(function_scope.build_ir(true));
						
						// add the recursively compiled instructions to the current scope's built instructions
						// TODO: figure out why this .build_ir() call uses clean_up_variables = false
						scope
						.instructions
						.extend(argument_translation_scope.build_ir(false));
				}
				Clause::DefineStruct { name: _, fields: _ }
				| Clause::DefineFunction {
					name: _,
					arguments: _,
					block: _,
				}
				| Clause::None => unreachable!(),
			}
		}
		
		Ok(scope)
	}
}

#[derive(Clone, Debug)]
/// Scope type represents a Mastermind code block,
/// any variables or functions defined within a {block} are owned by the scope and cleaned up before continuing
pub struct ScopeBuilder<'a, TC, OC> {
	/// a reference to the parent scope, for accessing things defined outside of this scope
	outer_scope: Option<&'a ScopeBuilder<'a, TC, OC>>,
	/// If true, scope is not able to access variables from outer scope.
	/// Used for embedded mm so that the inner mm can use outer functions but not variables.
	types_only: bool,
	
	/// Number of memory allocations in current scope
	allocations: usize,
	
	/// Mappings for variable names to memory allocation IDs in current scope
	variable_memory: HashMap<String, (ValueType, Memory)>,
	
	/// Functions accessible by any code within or in the current scope
	functions: Vec<(String, Vec<(String, ValueType)>, Vec<Clause<TC, OC>>)>,
	/// Struct types definitions
	structs: HashMap<String, DictStructType>,
	
	/// Intermediate instructions generated by the compiler
	instructions: Vec<Instruction<TC, OC>>,
}

impl<TC, OC> ScopeBuilder<'_, TC, OC>
where
TC: Display + Clone,
OC: Clone,
{
	pub fn new() -> ScopeBuilder<'static, TC, OC> {
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
	
	// regarding `clean_up_variables`:
	// I don't love this system of deciding what to clean up at the end in this specific function, but I'm not sure what the best way to achieve this would be
	// this used to be called "get_instructions" but I think this more implies things are being modified
	pub fn build_ir(mut self, clean_up_variables: bool) -> Vec<Instruction<TC, OC>> {
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
	
	fn push_instruction(&mut self, instruction: Instruction<TC, OC>) {
		self.instructions.push(instruction);
	}
	
	/// Open a scope within the current one, any time there is a {} in Mastermind, this is called
	fn open_inner(&self) -> ScopeBuilder<TC, OC> {
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
	fn open_inner_templates_only(&self) -> ScopeBuilder<TC, OC> {
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
	fn allocate_variable(&mut self, var: VariableTypeDefinition<TC>) -> Result<&ValueType, String> {
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
	
	/// find a function definition based on name and argument types (unaffected by the self.fn_only flag)
	fn get_function(
		&self,
		calling_name: &str,
		calling_arg_types: &Vec<ValueType>,
	) -> Result<Function<TC, OC>, String> {
		if let Some(func) = self.functions.iter().find(|(name, args, _)| {
			if name != calling_name || args.len() != calling_arg_types.len() {
				return false;
			}
			for ((_, arg_type), calling_arg_type) in zip(args, calling_arg_types) {
				if *arg_type != *calling_arg_type {
					return false;
				}
			}
			true
		}) {
			// TODO: stop cloning! This function overload stuff is tacked on and needs refactoring
			let (_, arguments, block) = func;
			return Ok(Function {
				arguments: arguments.clone(),
								block: block.clone(),
			});
		}
		
		if let Some(outer_scope) = self.outer_scope {
			return outer_scope.get_function(calling_name, calling_arg_types);
		}
		
		r_panic!(
			"Could not find function \"{calling_name}\" with correct arguments in current scope"
		);
	}
	
	/// Define a struct in this scope
	fn register_struct_definition(
		&mut self,
		struct_name: &str,
		fields: Vec<StructFieldTypeDefinition>,
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
		new_arguments: Vec<VariableTypeDefinition<TC>>,
		new_block: Vec<Clause<TC, OC>>,
	) -> Result<(), String> {
		let absolute_arguments: Vec<(String, ValueType)> = new_arguments
		.into_iter()
		.map(|f| {
			let LocationSpecifier::None = f.location_specifier else {
				r_panic!("Cannot specify variable location in function argument \"{f}\".");
			};
			Ok((f.name, self.create_absolute_type(&f.var_type)?))
		})
		.collect::<Result<Vec<(String, ValueType)>, String>>()?;
		
		// TODO: refactor this:
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
		match (&target.subfields, full_type, memory) {
			(None, ValueType::Cell, Memory::Cell { id }) => Ok(CellReference {
				memory_id: *id,
				index: None,
			}),
			(None, ValueType::Cell, Memory::MappedCell { id, index }) => Ok(CellReference {
				memory_id: *id,
				index: *index,
			}),
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
				Ok(CellReference {
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
				})
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
		}
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
		match (
			self.outer_scope,
			self.types_only,
			self.variable_memory.get(var_name),
		) {
			(_, _, Some((value_type, memory))) => Ok((value_type, memory)),
			(Some(outer_scope), false, None) => outer_scope.get_base_variable_memory(var_name),
			(None, _, None) | (Some(_), true, None) => {
				r_panic!("No variable found in scope with name \"{var_name}\".")
			}
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
	
	/// Create memory mapping between a pre-existing variable and a new one, used for function arguments.
	///  This could be used for copy by reference of subfields in future.
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
	
	/// Get the final type of an expression.
	///  (technically unnecessary right now, but can be used to implement expressions as function arguments in future)
	fn get_expression_type(&self, expr: &Expression) -> Result<ValueType, String> {
		Ok(match expr {
			Expression::NaturalNumber(_) => ValueType::Cell,
			 Expression::SumExpression { sign: _, summands } => {
				 let Some(_) = summands.first() else {
					 r_panic!(
						 "Cannot infer expression type because sum \
expression has no elements: `{expr}`."
					 );
				 };
				 // TODO: decide if the summands' types should be verified here or not
				 for summand in summands {
					 match self.get_expression_type(summand)? {
						 ValueType::Cell => (),
			 summand_type => {
				 r_panic!(
					 "Sum expressions must be comprised of cell-types: \
found `{summand_type}` in `{expr}`"
				 );
			 }
					 };
				 }
				 ValueType::Cell
			 }
			 Expression::VariableReference(var) => self.get_target_type(var)?.clone(),
			 Expression::ArrayLiteral(elements) => {
				 let mut elements_iter = elements.iter();
				 let Some(first_element) = elements_iter.next() else {
					 r_panic!(
						 "Cannot infer expression type because \
array literal has no elements: `{expr}`."
					 );
				 };
				 let first_element_type = self.get_expression_type(first_element)?;
				 for element in elements_iter {
					 let element_type = self.get_expression_type(element)?;
					 r_assert!(
						 element_type == first_element_type,
						 "All elements in array expressions must have the \
same type: found `{element_type}` in `{expr}`"
					 );
				 }
				 ValueType::Array(elements.len(), Box::new(first_element_type))
			 }
			 Expression::StringLiteral(s) => ValueType::Array(s.len(), Box::new(ValueType::Cell)),
		})
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

// TODO: think about where to put these tests, and by extension where to put the scopebuilder
#[cfg(test)]
mod scope_builder_tests {
	use crate::{
		backend::bf::{Opcode, TapeCell},
		parser::expressions::Sign,
	};
	
	use super::*;
	
	#[test]
	fn variable_allocation_1() {
		let mut scope = ScopeBuilder::<TapeCell, Opcode>::new();
		let allocated_type = scope.allocate_variable(VariableTypeDefinition {
			name: String::from("var"),
																								 var_type: VariableTypeReference::Cell,
																								 location_specifier: LocationSpecifier::None,
		});
		assert_eq!(allocated_type, Ok(&ValueType::Cell));
	}
	
	#[test]
	fn get_expression_type_numbers_1() {
		let scope = ScopeBuilder::<TapeCell, Opcode>::new();
		assert_eq!(
			scope
			.get_expression_type(&Expression::NaturalNumber(0))
			.unwrap(),
							 ValueType::Cell
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::NaturalNumber(1))
			.unwrap(),
							 ValueType::Cell
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::NaturalNumber(345678))
			.unwrap(),
							 ValueType::Cell
		);
	}
	
	#[test]
	fn get_expression_type_sums_1() {
		let scope = ScopeBuilder::<TapeCell, Opcode>::new();
		assert_eq!(
			scope
			.get_expression_type(&Expression::SumExpression {
				sign: Sign::Positive,
				summands: vec![Expression::NaturalNumber(0)]
			})
			.unwrap(),
							 ValueType::Cell
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::SumExpression {
				sign: Sign::Negative,
				summands: vec![
					Expression::NaturalNumber(345678),
													 Expression::NaturalNumber(2)
				]
			})
			.unwrap(),
							 ValueType::Cell
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::SumExpression {
				sign: Sign::Positive,
				summands: vec![
					Expression::SumExpression {
						sign: Sign::Negative,
						summands: vec![
							Expression::NaturalNumber(1),
													 Expression::NaturalNumber(2)
						]
					},
					Expression::NaturalNumber(2)
				]
			})
			.unwrap(),
							 ValueType::Cell
		);
	}
	
	#[test]
	fn get_expression_type_variables_1() {
		let mut scope = ScopeBuilder::<TapeCell, Opcode>::new();
		scope
		.allocate_variable(VariableTypeDefinition {
			name: String::from("var"),
											 var_type: VariableTypeReference::Cell,
											 location_specifier: LocationSpecifier::None,
		})
		.unwrap();
		assert_eq!(
			scope
			.get_expression_type(&Expression::VariableReference(VariableTarget {
				name: String::from("var"),
																													subfields: None,
																													is_spread: false
			}))
			.unwrap(),
							 ValueType::Cell
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::SumExpression {
				sign: Sign::Positive,
				summands: vec![
					Expression::VariableReference(VariableTarget {
						name: String::from("var"),
																				subfields: None,
																				is_spread: false
					}),
					Expression::NaturalNumber(123)
				]
			})
			.unwrap(),
							 ValueType::Cell
		);
	}
	
	#[test]
	fn get_expression_type_arrays_1() {
		let mut scope = ScopeBuilder::<TapeCell, Opcode>::new();
		scope
		.allocate_variable(VariableTypeDefinition {
			name: String::from("arr"),
											 var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 3),
											 location_specifier: LocationSpecifier::None,
		})
		.unwrap();
		assert_eq!(
			scope
			.get_expression_type(&Expression::VariableReference(VariableTarget {
				name: String::from("arr"),
																													subfields: None,
																													is_spread: false
			}))
			.unwrap(),
							 ValueType::Array(3, Box::new(ValueType::Cell))
		);
		assert_eq!(
			scope
			.get_expression_type(&Expression::VariableReference(VariableTarget {
				name: String::from("arr"),
																													subfields: Some(VariableTargetReferenceChain(vec![Reference::Index(0)])),
																													is_spread: false
			}))
			.unwrap(),
							 ValueType::Cell
		);
	}
	
	// TODO: make failure tests for expression types
}
