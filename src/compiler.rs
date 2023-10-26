use std::collections::{HashMap, HashSet};

use crate::parser::{Block, Command, VariableType};

#[derive(Debug)]
pub struct MastermindCompiler {
	pub program: Vec<char>,
	// would very much like to have a tree/linked structure here but rust is anal about these things, even if my implementation is correct???
	variable_scopes: Vec<VariableScope>,
	// the position that tape cell zero is on the allocation array, is this even needed? TODO
	// allocation_array_zero_offset: i32,
	allocation_array: Vec<bool>,
	stack_start_pos: Option<i32>,

	tape_offset_pos: i32,

	loop_depth: usize,
	// loop balance is 0 if the cell that started the loop is the same as the when you end the loop
	// basically if the number of < and > is equal inside a loop
	// each frame of the stack represents a loop, the last one is the current loop's balance factor
	loop_balance_stack: Vec<i32>,
}

#[derive(Debug)]
pub struct VariableScope {
	variable_aliases: HashMap<String, String>,
	variable_map: HashMap<String, CompilerVariable>,
}

impl VariableScope {
	pub fn new() -> VariableScope {
		VariableScope {
			variable_aliases: HashMap::new(),
			variable_map: HashMap::new(),
		}
	}
}

// not sure what this would be equivalent to in a normal compiler,
// but basically this "class" is the only thing that directly writes brainfuck code,
// it keeps track of tape memory allocation and abstracts tape head positioning from the actual compiler
impl MastermindCompiler {
	pub fn new() -> MastermindCompiler {
		MastermindCompiler {
			program: Vec::new(),
			variable_scopes: Vec::from([VariableScope::new()]),
			// allocation_array_zero_offset: 0,
			allocation_array: Vec::new(),
			stack_start_pos: None,
			tape_offset_pos: 0,
			loop_depth: 0,
			loop_balance_stack: Vec::new(),
		}
	}

	#[allow(dead_code)]
	pub fn to_string(&self) -> String {
		self.program.iter().collect()
	}

	pub fn move_to_cell(&mut self, target_pos: i32) {
		let forward = target_pos > self.tape_offset_pos;
		let character = match forward {
			true => '>',
			false => '<',
		};

		if let Some(balance) = self.loop_balance_stack.last_mut() {
			*balance += target_pos - self.tape_offset_pos;
		}

		for _ in 0..self.tape_offset_pos.abs_diff(target_pos) {
			self.program.push(character);
			self.tape_offset_pos += match forward {
				true => 1,
				false => -1,
			};
		}
	}

	pub fn get_var_scope<'a>(&'a mut self, var_name: &'a str) -> (&'a mut VariableScope, &'a str) {
		// tried very hard to do this recursively but rust is ridiculous
		let mut var_alias = var_name;
		let scope_iter = self.variable_scopes.iter_mut().rev();
		for scope in scope_iter {
			if scope.variable_map.contains_key(var_alias) {
				return (scope, var_alias);
			}
			if scope.variable_aliases.contains_key(var_alias) {
				var_alias = scope.variable_aliases.get(var_alias).unwrap();
			}
		}
		panic!("Could not find variable \"{}\"", var_name);
	}

	// returns true if variable is defined in this current scope
	// useful for const optimisations, basically check whether we can move the variable without affecting things
	pub fn check_var_scope(&mut self, var_name: &str) -> bool {
		let current_scope = self.variable_scopes.last().unwrap();
		current_scope.variable_map.contains_key(var_name)
	}

	pub fn get_current_scope(&mut self) -> &mut VariableScope {
		self.variable_scopes.last_mut().unwrap()
	}

	pub fn get_var(&mut self, var_name: &str) -> CompilerVariable {
		// iterate in reverse through the variables to check the latest scopes first
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		(*var_scope.variable_map.get(alias_name).unwrap()).clone()
	}

	fn add_to_current_cell(&mut self, imm: i8) {
		if imm == 0 {
			return;
		};

		let character = match imm > 0 {
			true => '+',
			false => '-',
		};
		for _ in 0..imm.abs() {
			self.program.push(character);
		}
	}

	fn add_to_var(&mut self, var: CompilerVariable, imm: i8) {
		match var {
			CompilerVariable::Boolean {
				var_name: _,
				cell,
				known_cell_value: _,
			}
			| CompilerVariable::Integer8 {
				var_name: _,
				cell,
				known_cell_value: _,
			} => {
				self.move_to_cell(cell);
				self.add_to_current_cell(imm);
			}
			CompilerVariable::Integer16 {
				var_name,
				cell1,
				cell2,
			} => {
				panic!("16bit integer addition unimplemented");
			}
			CompilerVariable::Integer24 {
				var_name,
				cell1,
				cell2,
				cell3,
			} => {
				panic!("24bit integer addition unimplemented");
			}
		}
	}

	// while(a) a--
	// if a is known at compile time:
	// a = a - a
	// whichever is more efficient
	fn _clear_cell(&mut self, cell: i32, known_value: Option<i8>) {
		self.move_to_cell(cell);

		// checking if abs(a) is 3 or less as clearing a cell with [-] is only 3 characters
		if known_value.is_some() && (known_value.unwrap().abs() <= 3) {
			self.add_to_current_cell(-known_value.unwrap());
		} else {
			self.open_loop();
			self.add_to_current_cell(-1);
			self.close_loop();
		}
	}

	// a, b, c = d; d = 0
	fn _drain_cell_into_multiple(&mut self, src_cell: i32, dest_cells: Vec<i32>) {
		self.move_to_cell(src_cell);
		self.open_loop();

		dest_cells.into_iter().for_each(|dest_cell| {
			self.move_to_cell(dest_cell);
			self.add_to_current_cell(1);
		});

		self.move_to_cell(src_cell);
		self.add_to_current_cell(-1);
		self.close_loop();
	}

	// a = b; b = 0
	fn _drain_cell(&mut self, src_cell: i32, dest_cell: i32) {
		self._drain_cell_into_multiple(src_cell, vec![dest_cell]);
	}

	// a, b, c = d; e used
	fn _copy_cell_into_multiple_with_existing_cell(
		&mut self,
		src_cell: i32,
		dest_cells: Vec<i32>,
		temp_cell: i32,
	) {
		let mut new_dest_cells = dest_cells.clone();
		new_dest_cells.push(temp_cell);
		self._drain_cell_into_multiple(src_cell, new_dest_cells);

		self._drain_cell(temp_cell, src_cell);
	}

	// a = b; c used
	fn _copy_cell_with_existing_cell(&mut self, src_cell: i32, dest_cell: i32, temp_cell: i32) {
		self._copy_cell_into_multiple_with_existing_cell(src_cell, vec![dest_cell], temp_cell);
	}

	// a, b, c = d
	fn _copy_cell_into_multiple(&mut self, src_cell: i32, dest_cells: Vec<i32>) {
		let temp_cell = self.allocate_cell();
		self._copy_cell_into_multiple_with_existing_cell(src_cell, dest_cells, temp_cell);
		self.free_cell(temp_cell);
	}

	// a = b
	fn _copy_cell(&mut self, src_cell: i32, dest_cell: i32) {
		self._copy_cell_into_multiple(src_cell, vec![dest_cell]);
	}

	// find free variable spaces and add to hashmap, you need to free this variable
	pub fn allocate_var(&mut self, var_name: &String, var_type: VariableType) {
		let pair: (String, CompilerVariable) = match var_type {
			VariableType::Boolean => (
				var_name.clone(),
				CompilerVariable::Boolean {
					var_name: var_name.clone(),
					cell: self.allocate_cell(),
					known_cell_value: None, // theoretically when we allocate we know the cell is 0, TODO
				},
			),
			VariableType::Integer8 => (
				var_name.clone(),
				CompilerVariable::Integer8 {
					var_name: var_name.clone(),
					cell: self.allocate_cell(),
					known_cell_value: None,
				},
			),
			VariableType::Integer16 => (
				var_name.clone(),
				CompilerVariable::Integer16 {
					var_name: var_name.clone(),
					cell1: self.allocate_cell(),
					cell2: self.allocate_cell(),
				},
			),
			VariableType::Integer24 => (
				var_name.clone(),
				CompilerVariable::Integer24 {
					var_name: var_name.clone(),
					cell1: self.allocate_cell(),
					cell2: self.allocate_cell(),
					cell3: self.allocate_cell(),
				},
			),
		};
		self.get_current_scope().variable_map.insert(pair.0, pair.1);
	}

	// move a variable without moving any contents, just change the underlying cell that a variable points at, EXPERIMENTAL
	// new_cell needs to already be allocated
	fn change_var_cell(&mut self, var_name: &str, new_cell: i32) -> i32 {
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		let old_var_details = var_scope.variable_map.remove(alias_name).unwrap();
		let (old_cell, new_var_details) = match old_var_details {
			CompilerVariable::Boolean {
				var_name: _,
				cell: old_cell,
				known_cell_value,
			} => (
				old_cell,
				CompilerVariable::Boolean {
					var_name: String::from(alias_name),
					cell: new_cell,
					known_cell_value,
				},
			),

			CompilerVariable::Integer8 {
				var_name: _,
				cell: old_cell,
				known_cell_value,
			} => (
				old_cell,
				CompilerVariable::Integer8 {
					var_name: String::from(alias_name),
					cell: new_cell,
					known_cell_value,
				},
			),

			CompilerVariable::Integer16 {
				var_name: _,
				cell1: _,
				cell2: _,
			}
			| CompilerVariable::Integer24 {
				var_name: _,
				cell1: _,
				cell2: _,
				cell3: _,
			} => {
				panic!("16bit and 24bit integer cell change unimplemented");
			}
		};

		var_scope
			.variable_map
			.insert(String::from(alias_name), new_var_details);

		// basically a pop operation, return the old cell so that it can be restored later
		return old_cell;
	}

	fn free_var(&mut self, var_name: &str) {
		// could probably be simplified
		let (scope, var_alias) = self.get_var_scope(var_name);

		let var_details = scope.variable_map.remove(var_alias).unwrap();

		match var_details {
			CompilerVariable::Boolean {
				var_name,
				cell,
				known_cell_value,
			}
			| CompilerVariable::Integer8 {
				var_name,
				cell,
				known_cell_value,
			} => {
				// TODO: known cell value stuff
				// if known_cell_value.is_none() || (known_cell_value.unwrap() != 0) {
				// 	panic!("Attempted to free variable \"{var_name}\" that has an unknown value or is non-zero");
				// }
				self.free_cell(cell);
			}
			CompilerVariable::Integer16 {
				var_name,
				cell1,
				cell2,
			} => {
				self.free_cell(cell1);
				self.free_cell(cell2);
			}
			CompilerVariable::Integer24 {
				var_name,
				cell1,
				cell2,
				cell3,
			} => {
				self.free_cell(cell1);
				self.free_cell(cell2);
				self.free_cell(cell3);
			}
		}
	}

	// find free cell and return the offset position (pointer basically)
	// if you do not free this it will stay and clog up future allocations
	fn allocate_cell(&mut self) -> i32 {
		// let mut pos = self.tape_offset_pos;
		let mut pos = 0;
		loop {
			let i: usize = pos // + self.allocation_array_zero_offset)
				.try_into()
				.unwrap();
			if let Some(stack_start_pos) = self.stack_start_pos {
				let stack_start_cell: usize = stack_start_pos // + self.allocation_array_zero_offset)
					.try_into()
					.unwrap();
				if i >= stack_start_cell {
					panic!("Cannot allocate cells past the current stack start cell");
				}
			}
			if i >= self.allocation_array.len() {
				self.allocation_array.resize(i + 1, false);
				*self.allocation_array.last_mut().unwrap() = true;
				return pos;
			} else if self.allocation_array[i] == false {
				self.allocation_array[i] = true;
				return pos;
			}
			pos += 1;
		}
	}

	fn free_cell(&mut self, cell_pos: i32) {
		let i: usize = cell_pos // + self.allocation_array_zero_offset)
			.try_into()
			.unwrap();
		self.allocation_array[i] = false;
	}

	fn open_loop(&mut self) {
		self.program.push('[');
		self.loop_balance_stack.push(0);
		self.loop_depth += 1;
	}

	fn close_loop(&mut self) {
		self.program.push(']');
		self.loop_depth -= 1;

		if self.loop_balance_stack.pop().unwrap() != 0 {
			panic!("Loop unbalanced!");
		}
	}

	fn open_scope(&mut self, translations: Option<&HashMap<String, String>>) {
		self.variable_scopes.push(VariableScope::new());

		if let Some(translations) = translations {
			let scope_aliases = &mut self.variable_scopes.last_mut().unwrap().variable_aliases;
			for (k, v) in translations.iter() {
				scope_aliases.insert(k.clone(), v.clone());
			}
		}
	}

	fn close_scope(&mut self) {
		// if you do not free all variables then they will be deleted but not deallocated (not cool)
		// it will not error out though, not sure if that's a good thing
		self.variable_scopes.pop();
	}

	fn add_symbol(&mut self, symbol: char) {
		self.program.push(symbol);
	}

	pub fn compile(&mut self, block: Block) {
		// let start_len = self.program.len();
		// the real meat and potatoes

		let mut scope_vars = HashSet::new();

		for cmd in block.commands.clone() {
			match cmd {
				Command::DeclareVariable { name, var_type } => {
					self.allocate_var(&name, var_type);
					scope_vars.insert(name);
				}
				Command::FreeVariable { var_name } => {
					self.free_var(&var_name);
					scope_vars.remove(&var_name);
				}
				Command::AddImmediate { var_name, imm } => {
					let var_details = self.get_var(&var_name);
					self.add_to_var(var_details, imm);
				}
				Command::CopyVariable {
					target_name,
					source_name,
				} => {
					// because bf is fucked, the quickest way to copy consumes the original variable
					// so we have to copy it twice, then copy one of them back to the original variable

					// if the variable is in its top level scope then it's okay to move the variable instead
					let source_ref = self.get_var(&source_name);
					let target_ref = self.get_var(&target_name);

					match (source_ref, target_ref) {
						(
							CompilerVariable::Integer8 {
								var_name: _,
								cell: source_cell,
								known_cell_value: _,
							}
							| CompilerVariable::Boolean {
								var_name: _,
								cell: source_cell,
								known_cell_value: _,
							}
							| CompilerVariable::Integer16 {
								var_name: _,
								cell1: source_cell,
								cell2: _,
							}
							| CompilerVariable::Integer24 {
								var_name: _,
								cell1: source_cell,
								cell2: _,
								cell3: _,
							},
							CompilerVariable::Integer8 {
								var_name: _,
								cell: target_cell,
								known_cell_value: _,
							}
							| CompilerVariable::Boolean {
								var_name: _,
								cell: target_cell,
								known_cell_value: _,
							},
						) => {
							// simple copy as any more significant binary digits would overflow instantly
							self._copy_cell(source_cell, target_cell);
						}
						_ => {
							panic!("Copy operation unimplemented for 16bit, 24bit");
						}
					}
				}
				Command::DrainVariable {
					target_name,
					source_name,
				} => {
					let source_ref = self.get_var(&source_name);
					let target_ref = self.get_var(&target_name);
					match (source_ref, target_ref) {
						(
							CompilerVariable::Integer8 {
								var_name: _,
								cell: source_cell,
								known_cell_value: _,
							},
							CompilerVariable::Integer8 {
								var_name: _,
								cell: target_cell,
								known_cell_value: _,
							},
						) => {
							self._drain_cell(source_cell, target_cell);
						}
						_ => {
							panic!("Drain operation unimplemented into 16bit, 24bit");
						}
					}
				}
				Command::ClearVariable { var_name } => {
					let var_ref = self.get_var(&var_name);
					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value,
						} => {
							// TODO: make optimisation for booleans within their top scope as they don't need brackets
							self._clear_cell(cell, known_cell_value);
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1,
							cell2,
						} => {
							self._clear_cell(cell1, None);
							self._clear_cell(cell2, None);
						}
						CompilerVariable::Integer24 {
							var_name: _,
							cell1,
							cell2,
							cell3,
						} => {
							self._clear_cell(cell1, None);
							self._clear_cell(cell2, None);
							self._clear_cell(cell3, None);
						}
					}
				}
				Command::PushStack { var_name } => {
					// this whole construction is a bit messy
					// TODO: redo
					self.stack_start_pos = Some(
						self.stack_start_pos.unwrap_or(
							{
								match self
									.allocation_array
									.iter()
									.rev()
									.enumerate()
									.find(|(_i, allocated)| !**allocated)
								{
									Some((last_allocated, _allocated)) => {
										if last_allocated == (self.allocation_array.len() - 1) {
											self.allocation_array.push(true);
										} else {
											self.allocation_array[last_allocated + 1] = true;
										}
										last_allocated + 1
									}
									None => {
										self.allocation_array.push(true);
										self.allocation_array.len() - 1
									}
								}
							}
							.try_into()
							.unwrap(),
						),
					);

					let var_ref = self.get_var(&var_name);

					if let CompilerVariable::Boolean {
						var_name: _,
						cell: var_cell,
						known_cell_value: _,
					}
					| CompilerVariable::Integer8 {
						var_name: _,
						cell: var_cell,
						known_cell_value: _,
					} = var_ref
					{
						self.move_to_cell(var_cell);
						self.add_symbol('['); // open a loop without the balance checker

						// move to the stack base cell (null byte)
						self.move_to_cell(self.stack_start_pos.unwrap());
						// pure brainfuck from here:
						// base cell of stack is null to find it easily, so move to the right one
						self.add_symbol('>');
						self.add_symbol('>');
						// move to the right until we get to the null cell
						self.add_symbol('[');
						// each stack element is two cells, cell two is a boolean indicating if cell one has data
						self.add_symbol('>');
						self.add_symbol('>');
						self.add_symbol(']');

						// move from the boolean to the value cell
						self.add_symbol('<');
						// increment
						self.add_symbol('+');
						// now find the start of the stack again
						self.add_symbol('<');
						self.add_symbol('[');
						self.add_symbol('<');
						self.add_symbol('<');
						self.add_symbol(']');

						// close loop
						self.move_to_cell(var_cell);
						self.add_to_current_cell(-1);
						self.add_symbol(']');

						// now set the stack top boolean to true, I wonder if this can be optimised?
						self.move_to_cell(self.stack_start_pos.unwrap());
						self.add_symbol('>');
						self.add_symbol('>');
						self.add_symbol('[');
						self.add_symbol('>');
						self.add_symbol('>');
						self.add_symbol(']');
						self.add_symbol('+');
						self.add_symbol('<');
						self.add_symbol('<');
						self.add_symbol('[');
						self.add_symbol('<');
						self.add_symbol('<');
						self.add_symbol(']');
						// should not move anywhere
						self.move_to_cell(self.stack_start_pos.unwrap());
					} else {
						panic!("Stack push operation not implemented for 16bit or 24bit variables");
					}
				}
				Command::PopStack { var_name } => {
					let var_ref = self.get_var(&var_name);

					let cell =
						match var_ref {
							CompilerVariable::Boolean {
								var_name: _,
								cell,
								known_cell_value: _,
							}
							| CompilerVariable::Integer8 {
								var_name: _,
								cell,
								known_cell_value: _,
							} => cell,
							CompilerVariable::Integer16 {
								var_name: _,
								cell1: _,
								cell2: _,
							}
							| CompilerVariable::Integer24 {
								var_name: _,
								cell1: _,
								cell2: _,
								cell3: _,
							} => {
								panic!("Stack pop operation not implemented for 16bit or 24bit variables");
							}
						};

					self.move_to_cell(self.stack_start_pos.unwrap());
					self.add_symbol('>');
					self.add_symbol('>');
					self.add_symbol('[');
					self.add_symbol('>');
					self.add_symbol('>');
					self.add_symbol(']');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol('<');

					self.add_symbol('[');
					self.add_symbol('>');
					self.add_symbol('[');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol(']');
					self.move_to_cell(cell);
					self.add_to_current_cell(1);
					self.move_to_cell(self.stack_start_pos.unwrap());
					self.add_symbol('>');
					self.add_symbol('>');
					self.add_symbol('[');
					self.add_symbol('>');
					self.add_symbol('>');
					self.add_symbol(']');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol('-');
					self.add_symbol(']');
					// now clean up the boolean
					self.add_symbol('>');
					self.add_symbol('-');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol('[');
					self.add_symbol('<');
					self.add_symbol('<');
					self.add_symbol(']');
					// this should not move anywhere but I left this here for clarity
					self.move_to_cell(self.stack_start_pos.unwrap());
				}
				Command::StackLoop { var_name, block } => {
					// // somehow loop through a stack then free it
					// self.move_to_pos(self.stack_start_pos.unwrap());

					// self.add_symbol('>');
					// self.add_symbol('>');
					// self.add_symbol('-');
					// self.add_symbol('<');
					// self.add_symbol('<');
					// self.add_symbol('[');
					// self.add_symbol('<');
					// self.add_symbol('<');
					// self.add_symbol(']');

					panic!("StackLoop unimplemented");

					self.stack_start_pos = None;
				}
				Command::ConsumeLoop {
					var_name,
					block: loop_block,
				} => {
					let var_ref = self.get_var(&var_name);

					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => {
							// to start the loop move to the variable you want to consume
							self.move_to_cell(cell);
							self.open_loop();
							// do what you want to do in the loop
							self.open_scope(None);
							self.compile(loop_block);
							self.close_scope();
							// decrement the variable
							self.move_to_cell(cell);
							self.add_to_current_cell(-1);

							self.close_loop();
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1: _,
							cell2: _,
						}
						| CompilerVariable::Integer24 {
							var_name: _,
							cell1: _,
							cell2: _,
							cell3: _,
						} => {
							panic!("Consume loop unimplemented for 16bit and 24bit integers");
						}
					}
				}
				Command::WhileLoop {
					var_name,
					block: loop_block,
				} => {
					let var_ref = self.get_var(&var_name);

					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => {
							// to start the loop move to the variable you want to consume
							self.move_to_cell(cell);
							self.open_loop();
							// do what you want to do in the loop
							self.open_scope(None);
							self.compile(loop_block);
							self.close_scope();

							self.move_to_cell(cell);
							self.close_loop();
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1: _,
							cell2: _,
						}
						| CompilerVariable::Integer24 {
							var_name: _,
							cell1: _,
							cell2: _,
							cell3: _,
						} => {
							panic!("While loop unimplemented for 16bit and 24bit integers");
						}
					}
				}
				Command::IfElse {
					var_name,
					consume,
					if_block,
					else_block,
				} => {
					let var_ref = self.get_var(&var_name);
					let var_cell = match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => cell,
						CompilerVariable::Integer16 {
							var_name: _,
							cell1: _,
							cell2: _,
						}
						| CompilerVariable::Integer24 {
							var_name: _,
							cell1: _,
							cell2: _,
							cell3: _,
						} => {
							panic!("If-else statements unimplemented for 16bit and 24bit integers");
						}
					};

					let temp_move_cell = match consume {
						false => Some(self.allocate_cell()),
						true => None,
					};
					let else_condition_cell = match else_block {
						Some(_) => {
							let else_condition_cell = self.allocate_cell();
							self.move_to_cell(else_condition_cell);
							self.add_to_current_cell(1);
							Some(else_condition_cell)
						}
						None => None,
					};

					// if block
					self.move_to_cell(var_cell);
					self.open_loop();

					if let Some(temp_move_cell) = temp_move_cell {
						// move if condition to temp cell
						self._drain_cell(var_cell, temp_move_cell);
					} else {
						// consume the if variable instead
						self._clear_cell(var_cell, None); // TODO: known values
					}

					// reassign the variable to the temporary cell so that if statement inner can use the variable
					let old_var_cell = match temp_move_cell {
						Some(temp_move_cell) => {
							Some(self.change_var_cell(&var_name, temp_move_cell))
						}
						None => None,
					};

					if let Some(else_condition_cell) = else_condition_cell {
						// remove the else condition so that it does not run
						self.move_to_cell(else_condition_cell);
						self.add_to_current_cell(-1);
					}

					self.compile(if_block);

					// move to the original variable position to close the loop
					self.move_to_cell(var_cell);
					self.close_loop();

					if let Some(old_var_cell) = old_var_cell {
						// now copy back the variable from the temp cell
						self._drain_cell(temp_move_cell.unwrap(), old_var_cell);

						// reassign the variable again (undo from before)
						self.change_var_cell(&var_name, old_var_cell);
						self.free_cell(temp_move_cell.unwrap());
					}

					if let Some(else_condition_cell) = else_condition_cell {
						// else block
						self.move_to_cell(else_condition_cell);
						self.open_loop();
						// clear else condition variable
						self.add_to_current_cell(-1);

						self.compile(else_block.unwrap());

						self.move_to_cell(else_condition_cell);
						self.close_loop();
						self.free_cell(else_condition_cell);
					}
				}
				Command::ScopedBlock {
					var_translations,
					block,
				} => {
					// tricky stuff, this is only used for functions atm
					// basically we need to recursively compile the contained block,
					// I think this will have to be done in the self

					// prime the compiler with the variable translations
					self.open_scope(Some(&var_translations));

					self.compile(block);

					// remove the variable translations from the self
					self.close_scope();
				}
				Command::DebugTape => {
					self.add_symbol('#');
				}
				Command::DebugGoto(var_name) => {
					let var_ref = self.get_var(&var_name);

					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => {
							self.move_to_cell(cell);
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1: _,
							cell2: _,
						}
						| CompilerVariable::Integer24 {
							var_name: _,
							cell1: _,
							cell2: _,
							cell3: _,
						} => {
							panic!("Debug-goto unimplemented for 16bit and 24bit integers");
						}
					}
				}
				Command::DebugPrintInt(var_name) => {
					let var_ref = self.get_var(&var_name);
					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => {
							self.move_to_cell(cell);
							self.add_symbol('@');
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1: _,
							cell2: _,
						}
						| CompilerVariable::Integer24 {
							var_name: _,
							cell1: _,
							cell2: _,
							cell3: _,
						} => {
							panic!("Debug print unimplemented for 16bit and 24bit integers");
						}
					}
				}
				Command::OutputByte {
					var_name,
					byte_index,
				} => {
					let var_ref = self.get_var(&var_name);
					let mut byte_refs = Vec::new();
					match var_ref {
						CompilerVariable::Boolean {
							var_name: _,
							cell,
							known_cell_value: _,
						}
						| CompilerVariable::Integer8 {
							var_name: _,
							cell,
							known_cell_value: _,
						} => {
							byte_refs.push(cell);
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1,
							cell2,
						} => {
							byte_refs.push(cell1);
							byte_refs.push(cell2);
						}
						CompilerVariable::Integer24 {
							var_name: _,
							cell1,
							cell2,
							cell3,
						} => {
							byte_refs.push(cell1);
							byte_refs.push(cell2);
							byte_refs.push(cell3);
						}
					}

					let i = byte_index.unwrap_or(0);
					if i >= byte_refs.len() {
						panic!("Variable \"{var_name}\" has {} bytes, attempted to output byte index {i}", byte_refs.len());
					}

					self.move_to_cell(byte_refs[i]);
					self.add_symbol('.');
				}
				Command::NoOp => (),
			}
		}

		for var in scope_vars.into_iter() {
			self.free_var(&var);
		}
	}
}

#[derive(Debug, Clone)]
enum CompilerVariable {
	Boolean {
		var_name: String,
		cell: i32,
		known_cell_value: Option<i8>,
	}, // could do some initial value stuff here for optimisations
	Integer8 {
		var_name: String,
		cell: i32,
		known_cell_value: Option<i8>,
	},
	Integer16 {
		var_name: String,
		cell1: i32,
		cell2: i32,
	},
	Integer24 {
		var_name: String,
		cell1: i32,
		cell2: i32,
		cell3: i32,
	},
}
