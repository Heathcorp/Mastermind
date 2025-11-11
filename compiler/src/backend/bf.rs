use super::common::{
	BrainfuckBuilder, BrainfuckBuilderData, BrainfuckProgram, CellAllocator, CellAllocatorData,
	OpcodeVariant, TapeCellVariant,
};
use crate::{
	macros::macros::{r_assert, r_panic},
	parser::tokeniser::Token,
};

pub type TapeCell = i32;
impl TapeCellVariant for TapeCell {
	fn origin_cell() -> TapeCell {
		0
	}
	fn with_offset(&self, offset: i32) -> Self {
		self + offset
	}
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Opcode {
	Add,
	Subtract,
	Right,
	Left,
	OpenLoop,
	CloseLoop,
	Output,
	Input,
	Clear,
}

impl OpcodeVariant for Opcode {
	fn try_from_char(c: char) -> Option<Opcode> {
		match c {
			'+' => Some(Opcode::Add),
			'-' => Some(Opcode::Subtract),
			'>' => Some(Opcode::Right),
			'<' => Some(Opcode::Left),
			'[' => Some(Opcode::OpenLoop),
			']' => Some(Opcode::CloseLoop),
			'.' => Some(Opcode::Output),
			',' => Some(Opcode::Input),
			_ => None,
		}
	}
}

impl CellAllocator<TapeCell> for CellAllocatorData<TapeCell> {
	/// Check if the desired number of cells can be allocated to the right of a given location
	fn check_allocatable(&mut self, location: &TapeCell, size: usize) -> bool {
		for k in 0..size {
			if self.cells.contains(&(location + k as i32)) {
				return false;
			}
		}
		return true;
	}

	/// Allocate size number of cells and return the location, optionally specify a location
	fn allocate(&mut self, location: Option<TapeCell>, size: usize) -> Result<TapeCell, String> {
		if let Some(l) = location {
			if !self.check_allocatable(&l, size) {
				r_panic!("Location specifier @{l} conflicts with another allocation");
			}
		}

		// find free space
		let mut region_start = location.unwrap_or(0);
		for i in region_start.. {
			if self.cells.contains(&i) {
				region_start = i + 1;
			} else if i - region_start == (size as i32 - 1) {
				break;
			}
		}

		for i in region_start..(region_start + size as i32) {
			r_assert!(
				self.cells.insert(i),
				"Unreachable error detected in cell allocation: allocate({location:?}, {size:?})"
			);
		}

		Ok(region_start)
	}

	/// Allocate a cell as close as possible to the given cell,
	/// used for optimisations which need extra cells for efficiency
	fn allocate_temp_cell(&mut self, location: TapeCell) -> TapeCell {
		// alternate left then right, getting further and further out
		let mut left_iter = (0..=location).rev();
		let mut right_iter = (location + 1)..;
		loop {
			if let Some(i) = left_iter.next() {
				// unallocated cell, allocate it and return
				if self.cells.insert(i) {
					return i;
				}
			}

			if let Some(i) = right_iter.next() {
				if self.cells.insert(i) {
					return i;
				}
			}
		}
	}

	fn free(&mut self, cell: TapeCell, size: usize) -> Result<(), String> {
		for i in cell..(cell + size as i32) {
			r_assert!(
				self.cells.remove(&i),
				"Cannot free cell @{i} as it is not allocated.",
			);
		}

		Ok(())
	}
}

impl BrainfuckProgram for Vec<Opcode> {
	fn to_string(self) -> String {
		let mut s = String::new();
		self.into_iter().for_each(|o| {
			s.push_str(match o {
				Opcode::Add => "+",
				Opcode::Subtract => "-",
				Opcode::Right => ">",
				Opcode::Left => "<",
				Opcode::OpenLoop => "[",
				Opcode::CloseLoop => "]",
				Opcode::Output => ".",
				Opcode::Input => ",",
				Opcode::Clear => "[-]",
			})
		});
		s
	}

	fn from_str(s: &str) -> Vec<Opcode> {
		let mut ops = Vec::new();
		let mut i = 0;
		while i < s.len() {
			let substr = &s[i..];
			if substr.starts_with("[-]") {
				ops.push(Opcode::Clear);
				i += 3;
			} else {
				match substr.chars().next().unwrap() {
					'+' => ops.push(Opcode::Add),
					'-' => ops.push(Opcode::Subtract),
					'>' => ops.push(Opcode::Right),
					'<' => ops.push(Opcode::Left),
					'[' => ops.push(Opcode::OpenLoop),
					']' => ops.push(Opcode::CloseLoop),
					'.' => ops.push(Opcode::Output),
					',' => ops.push(Opcode::Input),
					_ => (), // could put a little special opcode in for other characters
				}
				i += 1;
			}
		}

		ops
	}
}

impl BrainfuckProgram for BrainfuckBuilderData<TapeCell, Opcode> {
	fn to_string(self) -> String {
		self.opcodes.to_string()
	}

	fn from_str(s: &str) -> BrainfuckBuilderData<TapeCell, Opcode> {
		BrainfuckBuilderData {
			opcodes: Vec::from_str(s),
			head_pos: 0,
			// head_pos: TapeCell(0),
		}
	}
}

impl BrainfuckBuilder<TapeCell, Opcode> for BrainfuckBuilderData<TapeCell, Opcode> {
	fn new() -> BrainfuckBuilderData<TapeCell, Opcode> {
		BrainfuckBuilderData {
			opcodes: Vec::new(),
			head_pos: 0,
		}
	}
	fn len(&self) -> usize {
		self.opcodes.len()
	}
	fn push(&mut self, op: Opcode) {
		self.opcodes.push(op);
	}
	fn extend<T>(&mut self, ops: T)
	where
		T: IntoIterator<Item = Opcode>,
	{
		self.opcodes.extend(ops);
	}
	fn move_to_cell(&mut self, cell: TapeCell) {
		let x = cell;
		let x_pos = self.head_pos;
		//Move x level
		if x_pos < x {
			for _ in x_pos..x {
				self.opcodes.push(Opcode::Right);
			}
		} else if x < x_pos {
			// theoretically equivalent to cell..head_pos?
			for _ in ((x + 1)..=x_pos).rev() {
				self.opcodes.push(Opcode::Left);
			}
		}

		self.head_pos = cell;
	}

	fn add_to_current_cell(&mut self, imm: i8) {
		if imm > 0 {
			for _ in 0..imm {
				self.opcodes.push(Opcode::Add);
			}
		} else if imm < 0 {
			// needs to be i32 because -(-128) = -128 in i8-land
			for _ in 0..-(imm as i32) {
				self.opcodes.push(Opcode::Subtract);
			}
		}
	}

	fn clear_current_cell(&mut self) {
		self.opcodes.push(Opcode::OpenLoop);
		self.opcodes.push(Opcode::Subtract);
		self.opcodes.push(Opcode::CloseLoop);
	}
	fn output_current_cell(&mut self) {
		self.opcodes.push(Opcode::Output);
	}
	fn input_to_current_cell(&mut self) {
		self.opcodes.push(Opcode::Input);
	}
	fn open_loop(&mut self) {
		self.opcodes.push(Opcode::OpenLoop);
	}
	fn close_loop(&mut self) {
		self.opcodes.push(Opcode::CloseLoop);
	}
}
