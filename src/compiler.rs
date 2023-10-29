use std::collections::{HashMap, HashSet};

use crate::parser::{Block, Command, VariableType};

#[derive(Debug)]
pub struct MastermindCompiler {
	pub program: Vec<char>,
	// would very much like to have a tree/linked structure here but rust is anal about these things, even if my implementation is correct???
	variable_scopes: Vec<VariableScope>,
	// the position that tape cell zero is on the allocation array, is this even needed? TODO
	allocation_array_zero_offset: i32,
	allocation_array: Vec<bool>,
	stack_start_cell: Option<i32>,

	tape_head_pos: i32,

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
			allocation_array_zero_offset: 0,
			allocation_array: Vec::new(),
			stack_start_cell: None,
			tape_head_pos: 0,
			loop_depth: 0,
			loop_balance_stack: Vec::new(),
		}
	}

	#[allow(dead_code)]
	pub fn to_string(&self) -> String {
		self.program.iter().collect()
	}

	fn _move_to_cell(&mut self, cell: i32) {
		let forward = cell > self.tape_head_pos;
		let character = match forward {
			true => '>',
			false => '<',
		};

		if let Some(balance) = self.loop_balance_stack.last_mut() {
			*balance += cell - self.tape_head_pos;
		}

		for _ in 0..self.tape_head_pos.abs_diff(cell) {
			self.program.push(character);
			self.tape_head_pos += match forward {
				true => 1,
				false => -1,
			};
		}
	}

	fn get_var_scope<'a>(&'a mut self, var_name: &'a str) -> (&'a mut VariableScope, &'a str) {
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
	fn check_var_scope(&mut self, var_name: &str) -> bool {
		let current_scope = self.variable_scopes.last().unwrap();
		current_scope.variable_map.contains_key(var_name)
	}

	fn get_current_scope(&mut self) -> &mut VariableScope {
		self.variable_scopes.last_mut().unwrap()
	}

	fn get_var(&mut self, var_name: &str) -> CompilerVariable {
		// iterate in reverse through the variables to check the latest scopes first
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		var_scope.variable_map.get(alias_name).unwrap().clone()
	}

	fn _add_to_current_cell(&mut self, imm: i8) {
		if imm == 0 {
			return;
		};

		let i: usize = (self.tape_head_pos + self.allocation_array_zero_offset)
			.try_into()
			.unwrap();

		if !self.allocation_array[i] {
			panic!("Attempted to add to an unallocated tape cell");
		}

		let character = match imm > 0 {
			true => '+',
			false => '-',
		};
		for _ in 0..imm.abs() {
			self.program.push(character);
		}
	}

	fn _add_to_cell(&mut self, cell: i32, imm: i8) {
		self._move_to_cell(cell);
		self._add_to_current_cell(imm);
	}

	fn _add_to_var(&mut self, var: &CompilerVariable, imm: i8) {
		match var {
			CompilerVariable::Boolean { var_name, cell }
			| CompilerVariable::Integer8 { var_name, cell } => {
				self._add_to_cell(*cell, imm);
			}
			CompilerVariable::Integer16 {
				var_name: _,
				cell1: _,
				cell2: _,
			} => {
				panic!("16bit integer addition unimplemented");
			}
			CompilerVariable::Integer24 {
				var_name: _,
				cell1: _,
				cell2: _,
				cell3: _,
			} => {
				panic!("24bit integer addition unimplemented");
			}
		}
	}

	// while(a) a--
	// if a is known at compile time:
	// a = a - a
	// whichever is more efficient
	fn _clear_cell(&mut self, cell: i32) {
		self._move_to_cell(cell);

		// TODO: somehow keep track of variables/cells that we know the value of at compile time
		// checking if abs(a) is 3 or less as clearing a cell with [-] is only 3 characters
		// if let Some(known_value) = known_val {
		// 	if known_value.abs() <= 3 {
		// 		self._add_to_current_cell(-cell.known_value.unwrap());
		// 		return;
		// 	}
		// }

		self.open_loop();
		self._add_to_current_cell(-1);
		self.close_loop();
	}

	// TODO: make this with union types so you can input a vector or a cell?
	// a, b, c = d; d = 0
	fn _drain_cell_into_multiple(&mut self, src: i32, targets: Vec<i32>) {
		self._move_to_cell(src);
		self.open_loop();

		targets.into_iter().for_each(|dest_cell| {
			self._move_to_cell(dest_cell);
			self._add_to_current_cell(1);
		});

		self._move_to_cell(src);
		self._add_to_current_cell(-1);
		self.close_loop();
	}

	// a = b; b = 0
	fn _drain_cell(&mut self, src: i32, dest: i32) {
		self._drain_cell_into_multiple(src, vec![dest]);
	}

	// a, b, c = d; e used
	fn _copy_cell_into_multiple_with_existing_cell(
		&mut self,
		src: i32,
		targets: Vec<i32>,
		temp: i32,
	) {
		let mut new_targets = targets.clone();
		new_targets.push(temp);
		self._drain_cell_into_multiple(src, new_targets);

		self._drain_cell(temp, src);
	}

	// a = b; c used
	fn _copy_cell_with_existing_cell(&mut self, src: i32, dest: i32, temp: i32) {
		self._copy_cell_into_multiple_with_existing_cell(src, vec![dest], temp);
	}

	// a, b, c = d
	fn _copy_cell_into_multiple(&mut self, src_cell: i32, mut targets: Vec<i32>) {
		let temp = self.allocate_cell(None);
		self._copy_cell_into_multiple_with_existing_cell(src_cell, targets, temp);
		self.free_cell(temp);
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
					cell: self.allocate_cell(None),
				},
			),
			VariableType::Integer8 => (
				var_name.clone(),
				CompilerVariable::Integer8 {
					var_name: var_name.clone(),
					cell: self.allocate_cell(None),
				},
			),
			VariableType::Integer16 => (
				var_name.clone(),
				CompilerVariable::Integer16 {
					var_name: var_name.clone(),
					cell1: self.allocate_cell(None),
					cell2: self.allocate_cell(None),
				},
			),
			VariableType::Integer24 => (
				var_name.clone(),
				CompilerVariable::Integer24 {
					var_name: var_name.clone(),
					cell1: self.allocate_cell(None),
					cell2: self.allocate_cell(None),
					cell3: self.allocate_cell(None),
				},
			),
		};
		self.get_current_scope().variable_map.insert(pair.0, pair.1);
	}

	// move a variable without moving any contents, just change the underlying cell that a variable points at, EXPERIMENTAL
	// TODO: rethink this
	// new_cell needs to already be allocated
	fn change_var_cell(&mut self, var_name: &str, new_cell: i32) -> i32 {
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		let old_var_details = var_scope.variable_map.remove(alias_name).unwrap();
		let (old_cell, new_var_details) = match old_var_details {
			CompilerVariable::Boolean {
				var_name: _,
				cell: old_cell,
			} => (
				old_cell,
				CompilerVariable::Boolean {
					var_name: String::from(alias_name),
					cell: new_cell,
				},
			),

			CompilerVariable::Integer8 {
				var_name: _,
				cell: old_cell,
			} => (
				old_cell,
				CompilerVariable::Integer8 {
					var_name: String::from(alias_name),
					cell: new_cell,
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
			CompilerVariable::Boolean { var_name: _, cell }
			| CompilerVariable::Integer8 { var_name: _, cell } => {
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
	fn allocate_cell(&mut self, initial_val: Option<i8>) -> i32 {
		// let mut pos = self.tape_head_pos;
		let mut pos = 0;
		// TODO: refactor this loop, shouldn't need breaks
		loop {
			let i: usize = (pos + self.allocation_array_zero_offset)
				.try_into()
				.unwrap();
			if let Some(stack_start_cell) = &self.stack_start_cell {
				if i >= (stack_start_cell + self.allocation_array_zero_offset)
					.try_into()
					.unwrap()
				{
					panic!("Cannot allocate cells past the current stack start cell");
				}
			}
			if i >= self.allocation_array.len() {
				self.allocation_array.resize(i + 1, false);
				*self.allocation_array.last_mut().unwrap() = true;
				break;
			} else if self.allocation_array[i] == false {
				self.allocation_array[i] = true;
				break;
			}
			pos += 1;
		}

		if let Some(imm) = initial_val {
			self._move_to_cell(pos);
			self._add_to_current_cell(imm);
		}

		pos
	}

	fn free_cell(&mut self, cell: i32) {
		// if let Some(known_value) = cell.known_value {
		// 	if known_value != 0 {
		// 		panic!("Cannot free cell with known value of {known_value}");
		// 	}
		// } else {
		// 	panic!("Cannot free cell with unknown value");
		// };

		let i: usize = (cell + self.allocation_array_zero_offset)
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
				Command::AssertVariableValue { var_name, imm } => {
					// the compiler couldn't guarantee the value of a variable so the programmer can give a known value
					let mut var = self.get_var(&var_name);
					match &mut var {
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							// TODO
							// cell.known_value = Some(imm.try_into().unwrap());
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
							panic!("Variable value assertion unimplemented for 16bit and 24bit variables");
						}
					}
				}
				Command::AddImmediate { var_name, imm } => {
					let var = self.get_var(&var_name);
					self._add_to_var(&var, imm);
				}
				Command::CopyVariable {
					target_name,
					source_name,
				} => {
					// because bf is fucked, the quickest way to copy consumes the original variable
					// so we have to copy it twice, then copy one of them back to the original variable

					// if the variable is in its top level scope then it's okay to move the variable instead
					let source_var = self.get_var(&source_name);
					let target_var = self.get_var(&target_name);

					match (source_var, target_var) {
						(
							CompilerVariable::Integer8 {
								var_name: _,
								cell: source_cell,
							}
							| CompilerVariable::Boolean {
								var_name: _,
								cell: source_cell,
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
							}
							| CompilerVariable::Boolean {
								var_name: _,
								cell: target_cell,
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
							},
							CompilerVariable::Integer8 {
								var_name: _,
								cell: target_cell,
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
					let var = self.get_var(&var_name);
					match var {
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							// TODO: make optimisation for booleans within their top scope as they don't need brackets
							self._clear_cell(cell);
						}
						CompilerVariable::Integer16 {
							var_name: _,
							cell1,
							cell2,
						} => {
							self._clear_cell(cell1);
							self._clear_cell(cell2);
						}
						CompilerVariable::Integer24 {
							var_name: _,
							cell1,
							cell2,
							cell3,
						} => {
							self._clear_cell(cell1);
							self._clear_cell(cell2);
							self._clear_cell(cell3);
						}
					}
				}
				Command::PushStack { var_name } => {
					// this whole construction is a bit messy
					// TODO: redo and make a function
					if self.stack_start_cell.is_none() {
						self.stack_start_cell = Some(
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
						);
					}

					let stack_cell = self.stack_start_cell.clone().unwrap();

					let var = self.get_var(&var_name);

					if let CompilerVariable::Boolean {
						var_name: _,
						cell: var_cell,
					}
					| CompilerVariable::Integer8 {
						var_name: _,
						cell: var_cell,
					} = var
					{
						self._move_to_cell(var_cell);
						self.add_symbol('['); // open a loop without the balance checker

						// move to the stack base cell (null byte)
						self._move_to_cell(stack_cell);
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
						self._move_to_cell(var_cell);
						self._add_to_current_cell(-1);
						self.add_symbol(']');

						// now set the stack top boolean to true, I wonder if this can be optimised?
						self._move_to_cell(stack_cell);
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
						self._move_to_cell(stack_cell);
					} else {
						panic!("Stack push operation not implemented for 16bit or 24bit variables");
					}
					// because we cloned it
					self.stack_start_cell = Some(stack_cell);
				}
				Command::PopStack { var_name } => {
					let var = self.get_var(&var_name);

					let cell =
						match var {
							CompilerVariable::Boolean { var_name: _, cell }
							| CompilerVariable::Integer8 { var_name: _, cell } => cell,
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

					let stack_cell = self.stack_start_cell.clone().unwrap();

					self._move_to_cell(stack_cell);
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
					self._move_to_cell(cell);
					self._add_to_current_cell(1);
					self._move_to_cell(stack_cell);
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
					self._move_to_cell(stack_cell);

					self.stack_start_cell = Some(stack_cell);
				}
				Command::StackLoop { var_name, block } => {
					// // somehow loop through a stack then free it
					// self.move_to_pos(&self.stack_start_cell.unwrap());

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

					self.stack_start_cell = None;
				}
				Command::ConsumeLoop {
					var_name,
					block: loop_block,
				} => {
					let var = self.get_var(&var_name);

					match var {
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							// to start the loop move to the variable you want to consume
							self._move_to_cell(cell);
							self.open_loop();
							// do what you want to do in the loop
							self.open_scope(None);
							self.compile(loop_block);
							self.close_scope();
							// decrement the variable
							self._move_to_cell(cell);
							self._add_to_current_cell(-1);

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
					let var = self.get_var(&var_name);

					match var {
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							// to start the loop move to the variable you want to consume
							self._move_to_cell(cell);
							self.open_loop();
							// do what you want to do in the loop
							self.open_scope(None);
							self.compile(loop_block);
							self.close_scope();

							self._move_to_cell(cell);
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
					let var = self.get_var(&var_name);

					match (var, consume, else_block) {
						(
							CompilerVariable::Boolean { var_name: _, cell }
							| CompilerVariable::Integer8 { var_name: _, cell },
							true,
							None,
						) => {
							// simple if block which clears the variable
							self._move_to_cell(cell);
							self.open_loop();
							self._clear_cell(cell);

							self.compile(if_block);

							self._move_to_cell(cell);
							self.close_loop()
						}
						(
							CompilerVariable::Boolean { var_name: _, cell }
							| CompilerVariable::Integer8 { var_name: _, cell },
							false,
							None,
						) => {
							// if block which does some funky moves to keep the variable around
							// not sure if it would be better just to do a plain copy
							self._move_to_cell(cell);
							self.open_loop();

							let temp_cell = self.allocate_cell(None);
							self._drain_cell(cell, temp_cell);
							self.change_var_cell(&var_name, temp_cell); // TODO: with the borrow stuff this is probably going to break because it won't be able to access the variable because it has been removed when we borrow
											// might need to rethink this instead, maybe create a new variable name just for this if statement
											// other problem is when we insert the borrowed variable back into the variable map it will not know which scope to put it in

							self.compile(if_block);

							self._move_to_cell(cell);
							self.close_loop();

							// copy back, again this is probably flawed design
							self._drain_cell(temp_cell, cell);
							self.change_var_cell(&var_name, cell);
							self.free_cell(temp_cell);
						}
						(
							CompilerVariable::Boolean { var_name: _, cell }
							| CompilerVariable::Integer8 { var_name: _, cell },
							true,
							Some(else_block),
						) => {
							// if block with else block, clears the condition variable
							let else_cell = self.allocate_cell(Some(1));

							self._move_to_cell(cell);
							self.open_loop();
							self._clear_cell(cell);
							// clear else condition
							self._clear_cell(else_cell);

							self.compile(if_block);

							self._move_to_cell(cell);
							self.close_loop();

							// else
							self._move_to_cell(else_cell);
							self.open_loop();
							self._clear_cell(else_cell);

							self.compile(else_block);

							self._move_to_cell(else_cell);
							self.close_loop();
							self.free_cell(else_cell);
						}
						(
							CompilerVariable::Boolean { var_name: _, cell }
							| CompilerVariable::Integer8 { var_name: _, cell },
							false,
							Some(else_block),
						) => {
							// if block with else block, retains the condition variable
							let mut else_cell = self.allocate_cell(Some(1));

							self._move_to_cell(cell);
							self.open_loop();

							let temp_cell = self.allocate_cell(None);
							self._drain_cell(cell, temp_cell);
							self.change_var_cell(&var_name, temp_cell); // TODO: issues as seen above
											// clear else condition
							self._clear_cell(else_cell);

							self.compile(if_block);

							self._move_to_cell(cell);
							self.close_loop();

							// copy back, again this is probably flawed design
							self._drain_cell(temp_cell, cell);
							self.change_var_cell(&var_name, cell);
							self.free_cell(temp_cell);

							// else
							self._move_to_cell(else_cell);
							self.open_loop();
							self._clear_cell(else_cell);

							self.compile(else_block);

							self._move_to_cell(else_cell);
							self.close_loop();
							self.free_cell(else_cell);
						}
						(
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
							},
							_,
							_,
						) => {
							panic!("If-else statement not implemented for 16bit or 24bit integers")
						}
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
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							self._move_to_cell(cell);
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
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
							self._move_to_cell(cell);
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
					let var = self.get_var(&var_name);
					let mut byte_refs = Vec::new();
					match var {
						CompilerVariable::Boolean { var_name: _, cell }
						| CompilerVariable::Integer8 { var_name: _, cell } => {
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

					self._move_to_cell(byte_refs[i]);
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
	},
	Integer8 {
		var_name: String,
		cell: i32,
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
