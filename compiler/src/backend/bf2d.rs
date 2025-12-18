use super::common::{
	BrainfuckBuilder, BrainfuckBuilderData, BrainfuckProgram, CellAllocator, CellAllocatorData,
	OpcodeVariant, TapeCellVariant,
};
use crate::macros::macros::{r_assert, r_panic};

use std::hash::Hash;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct TapeCell2D(pub i32, pub i32);
impl TapeCellVariant for TapeCell2D {
	fn origin_cell() -> TapeCell2D {
		TapeCell2D(0, 0)
	}
	fn with_offset(&self, offset: i32) -> Self {
		TapeCell2D(self.0 + offset, self.1)
	}
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Opcode2D {
	Add,
	Subtract,
	Right,
	Left,
	OpenLoop,
	CloseLoop,
	Output,
	Input,
	Clear,
	Up,
	Down,
}

impl OpcodeVariant for Opcode2D {
	fn try_from_char(c: char) -> Option<Opcode2D> {
		match c {
			'+' => Some(Opcode2D::Add),
			'-' => Some(Opcode2D::Subtract),
			'>' => Some(Opcode2D::Right),
			'<' => Some(Opcode2D::Left),
			'^' => Some(Opcode2D::Up),
			'v' => Some(Opcode2D::Down),
			'[' => Some(Opcode2D::OpenLoop),
			']' => Some(Opcode2D::CloseLoop),
			'.' => Some(Opcode2D::Output),
			',' => Some(Opcode2D::Input),
			_ => None,
		}
	}
}

impl BrainfuckProgram for Vec<Opcode2D> {
	fn to_string(self) -> String {
		let mut s = String::new();
		self.into_iter().for_each(|o| {
			s.push_str(match o {
				Opcode2D::Add => "+",
				Opcode2D::Subtract => "-",
				Opcode2D::Right => ">",
				Opcode2D::Left => "<",
				Opcode2D::OpenLoop => "[",
				Opcode2D::CloseLoop => "]",
				Opcode2D::Output => ".",
				Opcode2D::Input => ",",
				Opcode2D::Clear => "[-]",
				Opcode2D::Up => "^",
				Opcode2D::Down => "v",
			})
		});
		s
	}

	fn from_str(s: &str) -> Vec<Opcode2D> {
		let mut ops = Vec::new();
		let mut i = 0;
		while i < s.len() {
			let substr = &s[i..];
			if substr.starts_with("[-]") {
				ops.push(Opcode2D::Clear);
				i += 3;
			} else {
				match substr.chars().next().unwrap() {
					'+' => ops.push(Opcode2D::Add),
					'-' => ops.push(Opcode2D::Subtract),
					'>' => ops.push(Opcode2D::Right),
					'<' => ops.push(Opcode2D::Left),
					'[' => ops.push(Opcode2D::OpenLoop),
					']' => ops.push(Opcode2D::CloseLoop),
					'.' => ops.push(Opcode2D::Output),
					',' => ops.push(Opcode2D::Input),
					'^' => ops.push(Opcode2D::Up),
					'v' => ops.push(Opcode2D::Down),
					_ => (), // could put a little special opcode in for other characters
				}
				i += 1;
			}
		}

		ops
	}
}

impl std::fmt::Display for TapeCell2D {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("({}, {})", self.0, self.1))?;
		Ok(())
	}
}

// TODO: refactor
impl CellAllocator<TapeCell2D> for CellAllocatorData<TapeCell2D> {
	/// Check if the desired number of cells can be allocated to the right of a given location
	fn check_allocatable(&mut self, location: &TapeCell2D, size: usize) -> bool {
		for k in 0..size {
			if self
				.cells
				.contains(&TapeCell2D(location.0 + k as i32, location.1))
			{
				return false;
			}
		}
		return true;
	}

	/// Will either check a specific location can be allocated at the chosen size or if no location is
	/// provided it will find a memory location where this size can be allocated
	/// Uses a variety of memory allocation methods based on settings
	fn allocate(
		&mut self,
		location: Option<TapeCell2D>,
		size: usize,
	) -> Result<TapeCell2D, String> {
		let mut region_start = location.unwrap_or(TapeCell2D(0, 0));
		//Check specified memory allocation above to ensure that this works nicely with all algorithms
		if let Some(l) = location {
			if !self.check_allocatable(&l, size) {
				r_panic!("Location specifier @{l} conflicts with another allocation");
			}
		} else {
			// should the region start at the current tape head?
			if self.config.memory_allocation_method == 0 {
				for i in region_start.0.. {
					if self.cells.contains(&TapeCell2D(i, region_start.1)) {
						region_start = TapeCell2D(i + 1, region_start.1);
					} else if i - region_start.0 == (size as i32 - 1) {
						break;
					}
				}
			} else if self.config.memory_allocation_method == 1 {
				//Zig Zag
				let mut found = false;
				let mut loops = 0;
				let mut i;
				let mut j;
				while !found {
					i = region_start.0 + loops;
					j = region_start.1;
					for _ in 0..=loops {
						if self.check_allocatable(&TapeCell2D(i, j), size) {
							found = true;
							region_start = TapeCell2D(i, j);
							break;
						}
						i = i - 1;
						j = j + 1;
					}
					loops += 1;
				}
			} else if self.config.memory_allocation_method == 2 {
				//Spiral
				let mut found = false;
				let mut loops = 1;
				let directions = ['N', 'E', 'S', 'W'];
				let mut i = region_start.0;
				let mut j = region_start.1;
				while !found {
					for dir in directions {
						match dir {
							'N' => {
								for _ in 0..loops {
									j += 1;
									if self.check_allocatable(&TapeCell2D(i, j), size) {
										found = true;
										region_start = TapeCell2D(i, j);
										break;
									}
								}
							}
							'E' => {
								for _ in 0..loops {
									i += 1;
									if self.check_allocatable(&TapeCell2D(i, j), size) {
										found = true;
										region_start = TapeCell2D(i, j);
										break;
									}
								}
							}
							'S' => {
								for _ in 0..loops {
									j -= 1;
									if self.check_allocatable(&TapeCell2D(i, j), size) {
										found = true;
										region_start = TapeCell2D(i, j);
										break;
									}
								}
							}
							'W' => {
								for _ in 0..loops {
									i -= 1;
									if self.check_allocatable(&TapeCell2D(i, j), size) {
										found = true;
										region_start = TapeCell2D(i, j);
										break;
									}
								}
							}
							_ => {}
						}
						if found {
							break;
						}
					}
					if found {
						break;
					}
					i -= 1;
					j -= 1;
					loops += 2;
				}
			} else if self.config.memory_allocation_method == 3 {
				//Tiles
				let mut found = false;
				let mut loops = 0;
				while !found {
					for i in -loops..=loops {
						for j in -loops..=loops {
							if self.check_allocatable(
								&TapeCell2D(region_start.0 + i, region_start.1 + j),
								size,
							) {
								found = true;
								region_start = TapeCell2D(region_start.0 + i, region_start.1 + j);
								break;
							}
						}
						if found {
							break;
						}
					}
					loops += 1;
				}
			} else {
				r_panic!(
					"Memory allocation method {} not implemented.",
					self.config.memory_allocation_method
				);
			}
		}

		// make all cells in the specified region allocated
		for i in region_start.0..(region_start.0 + size as i32) {
			if !self.cells.contains(&TapeCell2D(i, region_start.1)) {
				self.cells.insert(TapeCell2D(i, region_start.1));
			}
		}

		Ok(region_start)
	}

	/// Allocate a cell as close as possible to the given cell,
	/// used for optimisations which need extra cells for efficiency
	fn allocate_temp_cell(&mut self, location: TapeCell2D) -> TapeCell2D {
		// alternate left then right, getting further and further out
		let mut left_iter = (0..=location.0).rev();
		let mut right_iter = (location.0 + 1)..;
		loop {
			if let Some(i) = left_iter.next() {
				// unallocated cell, allocate it and return
				if self.cells.insert(TapeCell2D(i, location.1)) {
					return TapeCell2D(i, location.1);
				}
			}

			if let Some(i) = right_iter.next() {
				if self.cells.insert(TapeCell2D(i, location.1)) {
					return TapeCell2D(i, location.1);
				}
			}
		}
	}

	fn free(&mut self, cell: TapeCell2D, size: usize) -> Result<(), String> {
		for i in cell.0..(cell.0 + size as i32) {
			let c = TapeCell2D(i, cell.1);
			r_assert!(
				self.cells.remove(&c),
				"Cannot free cell @{c} as it is not allocated."
			);
		}

		Ok(())
	}
}

impl BrainfuckProgram for BrainfuckBuilderData<TapeCell2D, Opcode2D> {
	fn to_string(self) -> String {
		self.opcodes.to_string()
	}

	fn from_str(s: &str) -> BrainfuckBuilderData<TapeCell2D, Opcode2D> {
		BrainfuckBuilderData {
			opcodes: Vec::from_str(s),
			head_pos: TapeCell2D(0, 0),
		}
	}
}

impl BrainfuckBuilder<TapeCell2D, Opcode2D> for BrainfuckBuilderData<TapeCell2D, Opcode2D> {
	fn new() -> BrainfuckBuilderData<TapeCell2D, Opcode2D> {
		BrainfuckBuilderData {
			opcodes: Vec::new(),
			head_pos: TapeCell2D(0, 0),
		}
	}
	fn len(&self) -> usize {
		self.opcodes.len()
	}
	fn push(&mut self, op: Opcode2D) {
		self.opcodes.push(op);
	}
	fn extend<T>(&mut self, ops: T)
	where
		T: IntoIterator<Item = Opcode2D>,
	{
		self.opcodes.extend(ops);
	}
	fn move_to_cell(&mut self, cell: TapeCell2D) {
		let x = cell.0;
		let y = cell.1;
		let x_pos = self.head_pos.0;
		let y_pos = self.head_pos.1;
		//Move x level
		if x_pos < x {
			for _ in x_pos..x {
				self.opcodes.push(Opcode2D::Right);
			}
		} else if x < x_pos {
			// theoretically equivalent to cell..head_pos?
			for _ in ((x + 1)..=x_pos).rev() {
				self.opcodes.push(Opcode2D::Left);
			}
		}
		//Move y level
		if y_pos < y {
			for _ in y_pos..y {
				self.opcodes.push(Opcode2D::Up);
			}
		} else if y < y_pos {
			// theoretically equivalent to cell..head_pos?
			for _ in ((y + 1)..=y_pos).rev() {
				self.opcodes.push(Opcode2D::Down);
			}
		}
		self.head_pos = cell;
	}

	fn add_to_current_cell(&mut self, imm: i8) {
		if imm > 0 {
			for _ in 0..imm {
				self.opcodes.push(Opcode2D::Add);
			}
		} else if imm < 0 {
			// needs to be i32 because -(-128) = -128 in i8-land
			for _ in 0..-(imm as i32) {
				self.opcodes.push(Opcode2D::Subtract);
			}
		}
	}

	fn clear_current_cell(&mut self) {
		self.opcodes.push(Opcode2D::OpenLoop);
		self.opcodes.push(Opcode2D::Subtract);
		self.opcodes.push(Opcode2D::CloseLoop);
	}
	fn output_current_cell(&mut self) {
		self.opcodes.push(Opcode2D::Output);
	}
	fn input_to_current_cell(&mut self) {
		self.opcodes.push(Opcode2D::Input);
	}
	fn open_loop(&mut self) {
		self.opcodes.push(Opcode2D::OpenLoop);
	}
	fn close_loop(&mut self) {
		self.opcodes.push(Opcode2D::CloseLoop);
	}
}
