use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct BrainfuckBuilder {
	program: Vec<char>,
	// would very much like to have a tree/linked structure here but rust is anal about these things, even if my implementation is correct???
	variable_scopes: Vec<VariableScope>,
	// the position that tape cell zero is on the allocation array
	allocation_array_zero_offset: i32,
	allocation_array: Vec<bool>,
	tape_offset_pos: i32,

	loop_depth: usize,
}

#[derive(Debug)]
pub struct VariableScope {
	variable_aliases: HashMap<String, String>,
	variable_map: HashMap<String, i32>,
}

// not sure what this would be equivalent to in a normal compiler,
// but basically this "class" is the only thing that directly writes brainfuck code,
// it keeps track of tape memory allocation and abstracts tape head positioning from the actual compiler
impl BrainfuckBuilder {
	pub fn new() -> BrainfuckBuilder {
		BrainfuckBuilder {
			program: Vec::new(),
			variable_scopes: Vec::new(),
			allocation_array_zero_offset: 0,
			allocation_array: Vec::new(),
			tape_offset_pos: 0,
			loop_depth: 0,
		}
	}

	pub fn to_string(&self) -> String {
		self.program.iter().collect()
	}

	pub fn move_to_pos(&mut self, target_pos: i32) {
		let forward = target_pos > self.tape_offset_pos;
		let character = match forward {
			true => '>',
			false => '<',
		};
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
		panic!();
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

	pub fn add_to_current_cell(&mut self, imm: i32) {
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
		self.variable_scopes
			.last_mut()
			.unwrap()
			.variable_map
			.insert(String::from(var_name), cell);
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
		let mut pos = self.tape_offset_pos;
		loop {
			let i: usize = (pos + self.allocation_array_zero_offset)
				.try_into()
				.unwrap();
			if i >= self.allocation_array.len() {
				self.allocation_array.push(true);
				return pos;
			} else if self.allocation_array[i] == false {
				self.allocation_array[i] = true;
				return pos;
			}
			pos += 1;
		}
	}

	pub fn free_cell(&mut self, cell_pos: i32) {
		let i: usize = (cell_pos + self.allocation_array_zero_offset)
			.try_into()
			.unwrap();
		self.allocation_array[i] = false;
	}

	pub fn open_loop(&mut self) {
		self.program.push('[');
		self.loop_depth += 1;
	}

	pub fn close_loop(&mut self) {
		self.program.push(']');
		self.loop_depth -= 1;
	}

	pub fn open_scope(&mut self) {
		self.variable_scopes.push(VariableScope {
			variable_aliases: HashMap::new(),
			variable_map: HashMap::new(),
		})
	}

	pub fn close_scope(&mut self) {
		// if you do not free all variables then they will be deleted but not deallocated (not cool)
		// it will not error out though, not sure if that's a good thing
		self.variable_scopes.pop();
	}

	pub fn combine_builder(&mut self, other: &BrainfuckBuilder) {}
}
