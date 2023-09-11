use std::collections::HashMap;

#[derive(Debug)]
pub struct BrainfuckBuilder {
	program: Vec<char>,
	variables_map: HashMap<String, i32>,
	tape_offset_pos: i32,
}

impl BrainfuckBuilder {
	pub fn new<'a>() -> BrainfuckBuilder {
		BrainfuckBuilder {
			program: Vec::new(),
			variables_map: HashMap::new(),
			tape_offset_pos: 0,
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

	pub fn move_to_var(&mut self, var_name: &str) {
		let target_pos = *self.variables_map.get(var_name).unwrap();
		self.move_to_pos(target_pos);
	}

	pub fn add_to_cell(&mut self, imm: i32) {
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

	pub fn allocate_var(&mut self, var_name: &str) {
		// find free variable spaces and add to hashmap
		let mut max_val = -1;
		for var_pos in self.variables_map.values() {
			if *var_pos > max_val {
				max_val = *var_pos;
			}
		}
		self.variables_map
			.insert(String::from(var_name), max_val + 1);
	}

	pub fn open_loop(&mut self) {
		self.program.push('[');
	}

	pub fn close_loop(&mut self) {
		self.program.push(']');
	}
}
