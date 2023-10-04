use std::collections::HashMap;

use crate::parser::{Block, Command};

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
	variable_map: HashMap<String, i32>,
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

	pub fn move_to_pos(&mut self, target_pos: i32) {
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

	pub fn get_var_pos(&mut self, var_name: &str) -> i32 {
		// iterate in reverse through the variables to check the latest scopes first
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		var_scope.variable_map.get(alias_name).unwrap().clone()
	}

	pub fn move_to_var(&mut self, var_name: &str) {
		let target_pos = self.get_var_pos(var_name);
		self.move_to_pos(target_pos);
	}

	pub fn add_to_current_cell(&mut self, imm: i8) {
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

	// find free variable spaces and add to hashmap, you need to free this variable
	pub fn allocate_var(&mut self, var_name: &str) {
		let cell = self.allocate_cell();
		self.get_current_scope()
			.variable_map
			.insert(String::from(var_name), cell);
	}

	// move a variable without moving any contents, just change the underlying cell that a variable points at, EXPERIMENTAL
	// new_cell needs to already be allocated
	pub fn change_var_cell(&mut self, var_name: &str, new_cell: i32) -> i32 {
		let (var_scope, alias_name) = self.get_var_scope(var_name);
		let old_cell = var_scope
			.variable_map
			.insert(String::from(alias_name), new_cell)
			.unwrap();

		// basically a pop operation, return the old cell so that it can be restored later
		return old_cell;
	}

	pub fn free_var(&mut self, var_name: &str) {
		// could probably be simplified
		let (scope, var_alias) = self.get_var_scope(var_name);

		let cell = *scope.variable_map.get(var_alias).unwrap();
		scope.variable_map.remove(var_alias);

		self.free_cell(cell);
	}

	// find free cell and return the offset position (pointer basically)
	// if you do not free this it will stay and clog up future allocations
	pub fn allocate_cell(&mut self) -> i32 {
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

	pub fn free_cell(&mut self, cell_pos: i32) {
		let i: usize = cell_pos // + self.allocation_array_zero_offset)
			.try_into()
			.unwrap();
		self.allocation_array[i] = false;
	}

	pub fn open_loop(&mut self) {
		self.program.push('[');
		self.loop_balance_stack.push(0);
		self.loop_depth += 1;
	}

	pub fn close_loop(&mut self) {
		self.program.push(']');
		self.loop_depth -= 1;

		if self.loop_balance_stack.pop().unwrap() != 0 {
			panic!("Loop unbalanced!");
		}
	}

	pub fn open_scope(&mut self, translations: Option<&HashMap<String, String>>) {
		self.variable_scopes.push(VariableScope::new());

		if let Some(translations) = translations {
			let scope_aliases = &mut self.variable_scopes.last_mut().unwrap().variable_aliases;
			for (k, v) in translations.iter() {
				scope_aliases.insert(k.clone(), v.clone());
			}
		}
	}

	pub fn close_scope(&mut self) {
		// if you do not free all variables then they will be deleted but not deallocated (not cool)
		// it will not error out though, not sure if that's a good thing
		self.variable_scopes.pop();
	}

	pub fn add_symbol(&mut self, symbol: char) {
		self.program.push(symbol);
	}

	pub fn compile(&mut self, block: Block) {
		// let start_len = self.program.len();
		// the real meat and potatoes

		// start a new variable name scope

		for var in block.variables.iter() {
			if !var.argument {
				self.allocate_var(var.name.as_str());
			}
		}

		for cmd in block.commands.clone() {
			match cmd {
				Command::AddImmediate { var_name, imm } => {
					self.move_to_var(&var_name);
					self.add_to_current_cell(imm);
				}
				Command::CopyVariable {
					target_name,
					source_name,
				} => {
					// because bf is fucked, the quickest way to copy consumes the original variable
					// so we have to copy it twice, then copy one of them back to the original variable

					// if the variable is in its top level scope then it's okay to move the variable
					let temp_cell = self.allocate_cell();

					self.move_to_var(&source_name);
					self.open_loop();
					// decrement the source variable
					self.add_to_current_cell(-1);
					// increment the target variables
					self.move_to_var(&target_name);
					self.add_to_current_cell(1);
					self.move_to_pos(temp_cell);
					self.add_to_current_cell(1);

					self.move_to_var(&source_name);
					self.close_loop();

					match self.check_var_scope(&source_name) {
						true => {
							// variable is defined in this scope so copying and moving is not a problem
							// simply move the source variable to point to the temporary cell
							let old_var_cell = self.change_var_cell(&source_name, temp_cell);
							// free up the old variable's spot for something else
							// this won't be very helpful currently as variables are allocated before commands, TODO: change this
							self.free_cell(old_var_cell);
						}
						false => {
							// variable is not within this scope so we need to satisfy the loop balance
							// copy back the temp cell
							self.move_to_pos(temp_cell);
							self.open_loop();
							self.add_to_current_cell(-1);

							self.move_to_var(&source_name);
							self.add_to_current_cell(1);

							self.move_to_pos(temp_cell);
							self.close_loop();

							// free the temp memory
							self.free_cell(temp_cell);
						}
					}
				}
				Command::ClearVariable {
					var_name,
					is_boolean,
				} => {
					// TODO: make optimisation for booleans within their top scope as they don't need brackets
					self.move_to_var(&var_name);
					// pretty poor and won't work correctly at the moment if more than one clear happens or if the boolean starts as false
					// TODO: refactor a lot of things before trying to make this work
					let boolean_optimisation = self.check_var_scope(&var_name) && is_boolean; // TODO: wire up variable type
					if !boolean_optimisation {
						self.open_loop();
					}
					self.add_to_current_cell(-1);
					if !boolean_optimisation {
						self.close_loop();
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

					self.move_to_var(&var_name);
					self.add_symbol('['); // open a loop without the balance checker

					// move to the stack base cell (null byte)
					self.move_to_pos(self.stack_start_pos.unwrap());
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
					self.move_to_var(&var_name);
					self.add_to_current_cell(-1);
					self.add_symbol(']');

					// now set the stack top boolean to true, I wonder if this can be optimised?
					self.move_to_pos(self.stack_start_pos.unwrap());
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
				}
				Command::PopStack { var_name } => {
					self.move_to_pos(self.stack_start_pos.unwrap());
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
					self.move_to_var(&var_name);
					self.add_to_current_cell(1);
					self.move_to_pos(self.stack_start_pos.unwrap());
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
				}
				Command::ConsumeLoop {
					var_name,
					block: loop_block,
				} => {
					// to start the loop move to the variable you want to consume
					self.move_to_var(&var_name);
					self.open_loop();
					// do what you want to do in the loop
					self.open_scope(None);
					self.compile(loop_block);
					self.close_scope();
					// decrement the variable
					self.move_to_var(&var_name);
					self.add_to_current_cell(-1);

					self.close_loop();
				}
				Command::WhileLoop {
					var_name,
					block: loop_block,
				} => {
					// to start the loop move to the variable you want to consume
					self.move_to_var(&var_name);
					self.open_loop();

					// do what you want to do in the loop
					self.open_scope(None);
					self.compile(loop_block);
					self.close_scope();

					self.move_to_var(&var_name);
					self.close_loop();
				}
				Command::IfElse {
					var_name,
					consume,
					if_block,
					else_block,
				} => {
					let temp_move_cell = match consume {
						false => Some(self.allocate_cell()),
						true => None,
					};
					let else_condition_cell = match else_block {
						Some(_) => {
							let cell = self.allocate_cell();
							self.move_to_pos(cell);
							self.add_to_current_cell(1);
							Some(cell)
						}
						None => None,
					};

					// if block
					self.move_to_var(&var_name);
					self.open_loop();

					match temp_move_cell {
						Some(cell) => {
							// move if condition to temp cell
							self.open_loop();
							self.add_to_current_cell(-1);
							self.move_to_pos(cell);
							self.add_to_current_cell(1);
							self.move_to_var(&var_name);
							self.close_loop();
						}
						None => {
							// consume the if variable instead
							// TODO: check if it is a boolean and just decrement instead
							self.open_loop();
							self.add_to_current_cell(-1);
							self.close_loop();
						}
					}

					// reassign the variable to the temporary cell so that if statement inner can use the variable
					let old_var_cell = match temp_move_cell {
						Some(cell) => Some(self.change_var_cell(&var_name, cell)),
						None => None,
					};

					match else_condition_cell {
						Some(cell) => {
							// remove the else condition so that it does not run
							self.move_to_pos(cell);
							self.add_to_current_cell(-1);
						}
						None => (),
					};

					self.compile(if_block);

					match old_var_cell {
						Some(cell) => {
							// move to the original variable position to close the loop
							self.move_to_pos(cell);
							self.close_loop();

							// now copy back the variable from the temp cell
							self.move_to_var(&var_name);
							self.open_loop();
							self.add_to_current_cell(-1);
							self.move_to_pos(cell);
							self.add_to_current_cell(1);
							self.move_to_var(&var_name);
							self.close_loop();

							// reassign the variable again (undo from before)
							self.change_var_cell(&var_name, cell);
							self.free_cell(temp_move_cell.unwrap());
						}
						None => {
							// just move to the variable
							self.move_to_var(&var_name);
							self.close_loop();
						}
					}

					match else_condition_cell {
						Some(cell) => {
							// else block
							self.move_to_pos(cell);
							self.open_loop();
							// clear else condition variable
							self.add_to_current_cell(-1);

							self.compile(else_block.unwrap());

							self.move_to_pos(cell);
							self.close_loop();
							self.free_cell(cell);
						}
						None => (),
					};
				}
				Command::ScopedBlock {
					var_translations,
					block,
				} => {
					// tricky stuff, this is only used for functions atm
					// basically we need to recursively compile the contained block,
					// I think this will have to be done in the self

					// prime the self with the variable translations
					self.open_scope(Some(&var_translations));

					self.compile(block);

					// remove the variable translations from the self
					self.close_scope();
				}
				Command::DebugTape => {
					self.add_symbol('#');
				}
				Command::DebugGoto(var_name) => {
					self.move_to_var(&var_name);
				}
				Command::DebugPrintInt(var_name) => {
					self.move_to_var(&var_name);
					self.add_symbol('@');
				}
				Command::OutputByte { var_name } => {
					self.move_to_var(&var_name);
					self.add_symbol('.');
				}
			}
		}

		for var in block.variables.iter() {
			if !var.argument {
				self.free_var(var.name.as_str());
			}
		}

		// self.close_scope();
		///////
		// let s: String = self.program[start_len..self.program.len()]
		// 	.iter()
		// 	.collect();
		// println!("{block:#?} ::: {s}");
	}
}
