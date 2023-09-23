use std::collections::HashMap;

#[derive(Debug)]
pub struct BrainfuckBuilder {
	pub program: Vec<char>,
	// would very much like to have a tree/linked structure here but rust is anal about these things, even if my implementation is correct???
	variable_scopes: Vec<VariableScope>,
	// the position that tape cell zero is on the allocation array
	allocation_array_zero_offset: i32,
	allocation_array: Vec<bool>,
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
impl BrainfuckBuilder {
	pub fn new() -> BrainfuckBuilder {
		BrainfuckBuilder {
			program: Vec::new(),
			variable_scopes: Vec::from([VariableScope::new()]),
			allocation_array_zero_offset: 0,
			allocation_array: Vec::new(),
			tape_offset_pos: 0,
			loop_depth: 0,
			loop_balance_stack: Vec::new(),
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
			let i: usize = (pos + self.allocation_array_zero_offset)
				.try_into()
				.unwrap();
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
		let i: usize = (cell_pos + self.allocation_array_zero_offset)
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

	pub fn combine_builder(&mut self, other: &BrainfuckBuilder) {}
}
