// compile syntax tree into low-level instructions

use std::{collections::HashMap, iter::zip};

use crate::{
	builder::CellId,
	parser::{Clause, Expression, VariableSpec},
	MastermindConfig,
};

// memory stuff is all WIP and some comments may be incorrect

pub struct Compiler<'a> {
	pub config: &'a MastermindConfig,
}

impl Compiler<'_> {
	pub fn compile<'a>(&'a self, clauses: &[Clause], outer_scope: Option<&'a Scope>) -> Scope {
		let mut scope = if let Some(outer) = outer_scope {
			outer.open_inner()
		} else {
			Scope::new()
		};

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
					scope.functions.insert(
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
					let mem = scope.get_variable_mem(&var).unwrap();
					scope.push_instruction(Instruction::AddToCell(mem, value));
				}
				Clause::DeclareVariable(var) => {
					// allocate a memory cell on the allocation stack
					// place the variable spec in the hashmap given the memory offset
					scope.allocate_variable(var.clone());

					// create instructions to allocate cells
					if let Some(len) = var.arr_num {
						let mut var_copy = VariableSpec {
							name: var.name.clone(),
							arr_num: None,
						};
						for i in 0..len {
							var_copy.arr_num = Some(i);
							let mem = scope.get_variable_mem(&var_copy).unwrap();
							scope.push_instruction(Instruction::AllocateCell(mem));
						}
					} else {
						let mem = scope.get_variable_mem(&var).unwrap();
						scope.push_instruction(Instruction::AllocateCell(mem));
					}
				}
				Clause::ClearVariable(var) => {
					let mem = scope.get_variable_mem(&var).unwrap();
					scope.push_instruction(Instruction::ClearCell(mem));
				}
				Clause::InputByte { var } => {
					let mem = scope.get_variable_mem(&var).unwrap();
					scope.push_instruction(Instruction::InputToCell(mem));
				}
				Clause::OutputByte { value } => match value {
					Expression::VariableReference(var) => {
						let mem = scope.get_variable_mem(&var).unwrap();
						scope.push_instruction(Instruction::OutputCell(mem));
					}
					Expression::NaturalNumber(num) => {
						let val: u8 = (num % 256).try_into().unwrap();
						// allocate a temporary cell for the byte being output
						let mem = scope.allocate_unnamed_mem();
						scope.push_instruction(Instruction::AllocateCell(mem));
						scope.push_instruction(Instruction::AddToCell(mem, val));
						scope.push_instruction(Instruction::OutputCell(mem));
						scope.push_instruction(Instruction::ClearCell(mem));
						scope.push_instruction(Instruction::FreeCell(mem));
					}
					Expression::SumExpression {
						sign: _,
						summands: _,
					} => todo!(),
					Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => todo!(),
				},
				Clause::WhileLoop { var, block } => {
					// open loop on variable
					let mem = scope.get_variable_mem(&var).unwrap();
					scope.push_instruction(Instruction::OpenLoop(mem));

					// recursively compile instructions
					// TODO: when recursively compiling, check which things changed based on a return info value
					// TODO: make a function or something for this common pattern
					let loop_scope = self.compile(&block, Some(&scope));
					scope.instructions.extend(loop_scope.get_instructions());

					// close the loop
					scope.push_instruction(Instruction::CloseLoop(mem));
				}
				Clause::CopyVariable {
					target,
					source,
					constant,
				} => {
					// allocate a temporary cell
					let temp_mem = scope.allocate_unnamed_mem();
					scope.push_instruction(Instruction::AllocateCell(temp_mem));

					let source_mem = scope.get_variable_mem(&source).expect(&format!(
						"Source variable '{source}' couldn't be found while attempting copy"
					));
					let target_mem = scope.get_variable_mem(&target).expect(&format!(
						"Target variable '{target}' couldn't be found while attempting copy"
					));

					// copy source to target and temp
					scope.push_instruction(Instruction::OpenLoop(source_mem));
					scope.push_instruction(Instruction::AddToCell(target_mem, constant as u8));
					scope.push_instruction(Instruction::AddToCell(temp_mem, 1));
					scope.push_instruction(Instruction::AddToCell(source_mem, -1i8 as u8));
					scope.push_instruction(Instruction::CloseLoop(source_mem));
					// copy back from temp
					scope.push_instruction(Instruction::OpenLoop(temp_mem));
					scope.push_instruction(Instruction::AddToCell(source_mem, 1));
					scope.push_instruction(Instruction::AddToCell(temp_mem, -1i8 as u8));
					scope.push_instruction(Instruction::CloseLoop(temp_mem));
					scope.push_instruction(Instruction::FreeCell(temp_mem));
				}
				Clause::CopyLoop {
					source,
					targets,
					block,
					is_draining,
				} => match is_draining {
					true => {
						// again this stuff needs to be fixed
						let source_mem = scope.get_variable_mem(&source).unwrap();

						scope.push_instruction(Instruction::OpenLoop(source_mem));

						// recurse
						let loop_scope = self.compile(&block, Some(&scope));
						scope.instructions.extend(loop_scope.get_instructions());

						// copy into each target and decrement the source
						for target in targets {
							let mem = scope.get_variable_mem(&target).unwrap();
							scope.push_instruction(Instruction::AddToCell(mem, 1));
						}
						scope.push_instruction(Instruction::AddToCell(source_mem, -1i8 as u8)); // 255
						scope.push_instruction(Instruction::CloseLoop(source_mem));
						// instructions.push(Instruction::AssertCellValue(source_mem, 0)); // builder already knows the cell is value 0 because it ended a loop on its cell
					}
					false => {
						// allocate a temporary cell
						let temp_mem = scope.allocate_unnamed_mem();
						scope.push_instruction(Instruction::AllocateCell(temp_mem));

						// again this stuff needs to be fixed
						let source_mem = scope.get_variable_mem(&source).unwrap();

						scope.push_instruction(Instruction::OpenLoop(source_mem));

						// recurse
						let loop_scope = self.compile(&block, Some(&scope));
						scope.instructions.extend(loop_scope.get_instructions());

						// copy into each target and decrement the source
						for target in targets {
							let mem = scope.get_variable_mem(&target).unwrap();
							scope.push_instruction(Instruction::AddToCell(mem, 1));
						}
						scope.push_instruction(Instruction::AddToCell(temp_mem, 1));
						scope.push_instruction(Instruction::AddToCell(source_mem, -1i8 as u8)); // 255
						scope.push_instruction(Instruction::CloseLoop(source_mem));

						// copy back the temp cell
						scope.push_instruction(Instruction::OpenLoop(temp_mem));
						scope.push_instruction(Instruction::AddToCell(temp_mem, -1i8 as u8));
						scope.push_instruction(Instruction::AddToCell(source_mem, 1));
						scope.push_instruction(Instruction::CloseLoop(temp_mem));

						scope.push_instruction(Instruction::FreeCell(temp_mem));
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
					let mut new_scope = scope.open_inner();

					let temp_var_mem = new_scope.allocate_unnamed_mem();
					new_scope.push_instruction(Instruction::AllocateCell(temp_var_mem));
					let else_condition_mem = match else_block {
						Some(_) => {
							let else_mem = new_scope.allocate_unnamed_mem();
							new_scope.push_instruction(Instruction::AllocateCell(else_mem));
							new_scope.push_instruction(Instruction::AddToCell(else_mem, 1));
							Some(else_mem)
						}
						None => None,
					};

					let original_var_mem = new_scope.get_variable_mem(&var).unwrap();

					new_scope.push_instruction(Instruction::OpenLoop(original_var_mem));

					// move the variable to the temporary cell
					new_scope.push_instruction(Instruction::OpenLoop(original_var_mem));
					new_scope.push_instruction(Instruction::AddToCell(temp_var_mem, 1));
					new_scope
						.push_instruction(Instruction::AddToCell(original_var_mem, -1i8 as u8));
					new_scope.push_instruction(Instruction::CloseLoop(original_var_mem));

					// change scope var pointer
					new_scope.reassign_variable_mem(var.clone(), temp_var_mem);

					// TODO: think about this?
					// free the original cell temporarily as it isn't being used
					// instructions.push(Instruction::FreeCell(original_var_mem));

					// set the else condition cell
					if let Some(mem) = else_condition_mem {
						new_scope.push_instruction(Instruction::AddToCell(mem, -1i8 as u8));
					};

					// recursively compile if block
					if let Some(block) = if_block {
						let if_scope = self.compile(&block, Some(&new_scope));
						new_scope.instructions.extend(if_scope.get_instructions());
					};

					// close if block
					new_scope.push_instruction(Instruction::CloseLoop(original_var_mem));

					// TODO: think about this?
					// reallocate the temporarily freed variable cell
					// instructions.push(Instruction::AllocateCell(original_var_mem));

					// move the temporary cell contents back to the variable cell
					new_scope.push_instruction(Instruction::OpenLoop(temp_var_mem));
					new_scope.push_instruction(Instruction::AddToCell(original_var_mem, 1));
					new_scope.push_instruction(Instruction::AddToCell(temp_var_mem, -1i8 as u8));
					new_scope.push_instruction(Instruction::CloseLoop(temp_var_mem));
					new_scope.revert_reassignment(&var);

					new_scope.push_instruction(Instruction::FreeCell(temp_var_mem));

					// else block:
					if let Some(else_mem) = else_condition_mem {
						new_scope.push_instruction(Instruction::OpenLoop(else_mem));
						new_scope.push_instruction(Instruction::AddToCell(else_mem, -1i8 as u8));

						// recursively compile else block
						let block = else_block.unwrap();
						let else_scope = self.compile(&block, Some(&new_scope));
						new_scope.instructions.extend(else_scope.get_instructions());

						new_scope.push_instruction(Instruction::CloseLoop(else_mem));
						new_scope.push_instruction(Instruction::FreeCell(else_mem));
					}

					// extend the inner scopes instructions onto the outer one
					// maybe this if statement business should be its own function?
					scope.instructions.extend(new_scope.get_instructions());
				}
				Clause::CallFunction {
					function_name,
					arguments,
				} => {
					// create variable translations and recursively compile the inner variable block
					let Some(function_definition) = scope.get_function(&function_name) else {
						panic!("No function with name \"{}\" found", function_name);
					};

					let mut new_scope = scope.open_inner();
					new_scope.variable_aliases.extend(
						zip(function_definition.arguments.clone().into_iter(), arguments).map(
							|(arg_definition, calling_arg)| match (arg_definition, calling_arg) {
								(
									VariableSpec {
										name: def_name,
										arr_num: None,
									},
									VariableSpec {
										name: call_name,
										arr_num: None,
									},
								) => ArgumentTranslation::SingleToSingle(def_name, call_name),
								(
									VariableSpec {
										name: def_name,
										arr_num: None,
									},
									calling_var,
								) => ArgumentTranslation::SingleToMultiCell(def_name, calling_var),
								(
									def_var,
									VariableSpec {
										name: call_name,
										arr_num: None,
									},
								) => {
									let calling_var_len =
										scope.get_variable_size(&call_name).unwrap();
									if calling_var_len != def_var.arr_num.unwrap() {
										panic!("Cannot translate {call_name} as {def_var} as the lengths do not match");
									}
									ArgumentTranslation::MultiToMulti(def_var.name, call_name)
								}
								(def_var, call_var) => {
									panic!("Cannot translate {call_var} as argument {def_var}");
								}
							},
						),
					);

					// recurse
					let loop_scope = self.compile(&function_definition.block, Some(&new_scope));
					new_scope.instructions.extend(loop_scope.get_instructions());

					// extend the inner scope instructions onto the outer scope
					// maybe function call compiling should be its own function?
					scope.instructions.extend(new_scope.get_instructions());
				}
				Clause::DefineFunction {
					name: _,
					arguments: _,
					block: _,
				} => (),
			}
		}

		// TODO: check if current scope has any leftover memory allocations and free them
		// TODO: check known values and clear them selectively
		// create instructions to free cells
		let mem_offset = scope.allocation_offset();
		for (_, mem_rel) in scope.variable_memory_cells.clone() {
			let mem = mem_rel + mem_offset;
			scope.push_instruction(Instruction::ClearCell(mem));
			scope.push_instruction(Instruction::FreeCell(mem));
		}

		scope
	}
}

// this is subject to change
#[derive(Debug, Clone)]
pub enum Instruction {
	AllocateCell(CellId), // most of the below comments are wrong, usize is a unique id of an allocated cell
	FreeCell(CellId), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(CellId), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop(CellId), // pass in the cell id, this originally wasn't there but may be useful later on
	AddToCell(CellId, u8),
	InputToCell(CellId),
	ClearCell(CellId), // not sure if this should be here, seems common enough that it should be
	// AssertCellValue(CellId, u8), // again not sure if this is the correct place but whatever, or if this is even needed?
	OutputCell(CellId),
	// TODO: contiguous cells for quick iterations?
	// AllocateContiguousCells(usize),
	// FreeContiguousCells(usize), // number indicates
}

#[derive(Clone, Debug)]
pub struct Scope<'a> {
	outer_scope: Option<&'a Scope<'a>>,

	// number of memory cells that have been allocated?
	allocations: usize,
	// mappings for variable names to places on above stack
	variable_memory_cells: HashMap<VariableSpec, CellId>,
	// used for function arguments, translates an outer scope variable to an inner one, assumed they are the same array length if multi-cell
	// originally this was just string to string, but we need to be able to map a single-bit variable to a cell of an outer array variable
	variable_aliases: Vec<ArgumentTranslation>,
	// thought I didn't need this but I do, basically a record of the byte lengths of each variable
	variable_sizes: HashMap<String, Option<usize>>,

	// functions accessible by any code within or in the current scope
	functions: HashMap<String, Function>,

	// experimental: This is where we keep track of the instructions generated by the compiler, then we return the scope to the calling function
	instructions: Vec<Instruction>,
	// very experimental: this is used for optimisations to keep track of how variables are used
	variable_accesses: HashMap<VariableSpec, (usize, usize)>, // (reads, writes)
}

#[derive(Clone, Debug)]
enum ArgumentTranslation {
	SingleToSingle(String, String),
	SingleToMultiCell(String, VariableSpec),
	MultiToMulti(String, String),
}
impl ArgumentTranslation {
	fn get_def_name(&self) -> &String {
		let (ArgumentTranslation::SingleToSingle(def_name, _)
		| ArgumentTranslation::SingleToMultiCell(def_name, _)
		| ArgumentTranslation::MultiToMulti(def_name, _)) = self;
		def_name
	}
}

#[derive(Clone, Debug)] // probably shouldn't be cloning here but whatever
struct Function {
	arguments: Vec<VariableSpec>,
	block: Vec<Clause>,
}
// represents a position in a stack relative to the head/top

impl Scope<'_> {
	pub fn new() -> Scope<'static> {
		Scope {
			outer_scope: None,
			allocations: 0,
			variable_memory_cells: HashMap::new(),
			variable_aliases: Vec::new(),
			variable_sizes: HashMap::new(),
			functions: HashMap::new(),
			instructions: Vec::new(),
			variable_accesses: HashMap::new(),
		}
	}

	pub fn get_instructions(self) -> Vec<Instruction> {
		// optimisations could go here?
		// TODO: move all optimisations from the builder to here
		self.instructions
	}

	fn push_instruction(&mut self, instruction: Instruction) {
		self.instructions.push(instruction);
	}

	fn open_inner(&self) -> Scope {
		Scope {
			outer_scope: Some(self),
			allocations: 0,
			variable_memory_cells: HashMap::new(),
			variable_aliases: Vec::new(),
			variable_sizes: HashMap::new(),
			functions: HashMap::new(),
			instructions: Vec::new(),
			variable_accesses: HashMap::new(),
		}
	}

	fn allocate_variable(&mut self, var: VariableSpec) {
		self.variable_sizes.insert(var.name.clone(), var.arr_num);
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
	}

	// not sure if this is even needed tbh, (TODO: refactor?)
	fn allocate_variable_cell(&mut self, var: VariableSpec) {
		let mem = self.push_memory_cell();
		self.variable_memory_cells.insert(var, mem);
	}

	fn push_memory_cell(&mut self) -> usize {
		// TODO: do we need to track this anywhere for cleanup?
		self.allocations += 1;
		let mem = self.allocations - 1;
		mem
	}

	fn allocate_unnamed_mem(&mut self) -> usize {
		let current_scope_relative = self.push_memory_cell();
		current_scope_relative + self.allocation_offset()
	}

	// recursively tallies the allocation stack size of the outer scope, does not include this scope
	fn allocation_offset(&self) -> usize {
		if let Some(outer_scope) = self.outer_scope {
			outer_scope.allocations + outer_scope.allocation_offset()
		} else {
			0
		}
	}

	fn get_function(&self, name: &str) -> Option<&Function> {
		if let Some(func) = self.functions.get(name) {
			Some(func)
		} else if let Some(outer_scope) = self.outer_scope {
			outer_scope.get_function(name)
		} else {
			None
		}
	}

	fn get_variable_mem(&self, var: &VariableSpec) -> Option<usize> {
		// TODO: check for variable size index out of range errors
		if let Some(mem) = self.variable_memory_cells.get(var) {
			return Some(mem + self.allocation_offset());
		} else if let Some(outer_scope) = self.outer_scope {
			if let Some(alias) = self.variable_aliases.iter().find_map(|translation| {
				if *translation.get_def_name() == var.name {
					match translation {
						ArgumentTranslation::SingleToSingle(_, call_name) => Some(VariableSpec {
							name: call_name.clone(),
							arr_num: None,
						}),
						ArgumentTranslation::SingleToMultiCell(_, call_var) => {
							Some(call_var.clone())
						}
						ArgumentTranslation::MultiToMulti(_, call_name) => Some(VariableSpec {
							name: call_name.clone(),
							arr_num: var.arr_num,
						}),
					}
				} else {
					None
				}
			}) {
				return outer_scope.get_variable_mem(&alias);
			} else {
				return outer_scope.get_variable_mem(var);
			}
		}

		panic!("No variable {var} found in current scope.");
	}

	// TODO: refactor this stuff heavily when we make multi-byte values contiguous and inline asm stuff
	fn get_variable_size(&self, var_name: &str) -> Option<usize> {
		if let Some(len) = self.variable_sizes.get(var_name) {
			*len
		} else if let Some(outer_scope) = self.outer_scope {
			if let Some(alias_name) = self.variable_aliases.iter().find_map(|translation| {
				if translation.get_def_name() == var_name {
					match translation {
						ArgumentTranslation::SingleToSingle(_, _)
						| ArgumentTranslation::SingleToMultiCell(_, _) => panic!(
							"Something went wrong finding the size of variable \"{var_name}\""
						),
						ArgumentTranslation::MultiToMulti(_, call_name) => Some(call_name.clone()),
					}
				} else {
					None
				}
			}) {
				outer_scope.get_variable_size(&alias_name)
			} else {
				outer_scope.get_variable_size(var_name)
			}
		} else {
			panic!("Size of variable '{var_name}' could not be found");
		}
	}

	// add a pointer to the variable in the scope, the scope cannot directly own the variable being reassigned
	// mem is including the offset, as it is a value returned from a prior allocate call
	fn reassign_variable_mem(&mut self, var: VariableSpec, mem: usize) {
		let None = self.variable_memory_cells.get(&var) else {
			panic!("Cannot reassign {var} in same scope as it is defined!");
		};

		self.variable_memory_cells
			.insert(var, mem - self.allocation_offset());
	}

	// reverts the above operation, again needs the original variable to not be stored directly in this scope
	fn revert_reassignment(&mut self, var: &VariableSpec) {
		self.variable_memory_cells.remove(var);
	}
}
