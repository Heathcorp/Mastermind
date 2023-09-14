use std::collections::HashMap;

#[derive(Debug)]
pub struct BrainfuckBuilder {
	program: Vec<char>,
	// originally this was a hashmap but I did it this way so that variables can share names
	// TODO: redo this to keep track of scopes so we can have multiple variables with the same name
	variables_map: Vec<(String, i32)>,
	// the position that tape cell zero is on the allocation array
	allocation_array_zero_offset: i32,
	allocation_array: Vec<bool>,
	tape_offset_pos: i32,

	loop_depth: usize,
}

// not sure what this would be equivalent to in a normal compiler,
// but basically this "class" is the only thing that directly writes brainfuck code,
// it keeps track of tape memory allocation and abstracts tape head positioning from the actual compiler
impl BrainfuckBuilder {
	pub fn new() -> BrainfuckBuilder {
		BrainfuckBuilder {
			program: Vec::new(),
			variables_map: Vec::new(),
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

	pub fn get_var_pos(&self, var_name: &str) -> i32 {
		// iterate in reverse through the variables so that we can have multiple variables named the same
		println!("{var_name} {:#?}", self.variables_map);
		self.variables_map
			.iter()
			.rev()
			.find(|(vn, pos)| var_name == *vn)
			.unwrap()
			.1
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
		self.variables_map.push((String::from(var_name), cell));
	}

	pub fn free_var(&mut self, var_name: &str) {
		// could probably be simplified
		let (index, (vn, cell)) = self
			.variables_map
			.iter()
			.enumerate()
			.rev()
			.find(|(i, (vn, cell))| var_name == *vn)
			.unwrap();

		self.free_cell(*cell);
		// probably could be optimised, this can have O(n) if shifting a lot of elements,
		// theoretically you shouldn't be freeing old variables often enough for this to matter,
		// also this is a compiler not an interpreter
		self.variables_map.remove(index);
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

	pub fn combine_builder(&mut self, other: &BrainfuckBuilder) {}
}
