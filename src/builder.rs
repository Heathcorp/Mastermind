// turns low-level bf instructions into plain bf
// take in a timeline of cell allocations and move-to-cell operations, etc
// output plain bf according to that spec

// this algorithm is responsible for actually allocating physical tape cells as opposed to the parser
// can introduce optimisations here with some kind of allocation timeline sorting algorithm (hard leetcode style problem)

use std::{collections::HashMap, fmt::Display};

use crate::compiler::Instruction;

pub fn build(instructions: Vec<Instruction>) -> String {
	let mut alloc_tape = Vec::new();
	let mut alloc_map = HashMap::new();

	let mut loop_stack = Vec::new();
	let mut head_pos = 0;
	let mut ops = Vec::new();

	for instruction in instructions {
		match instruction {
			// the ids (indices really) given by the compiler are guaranteed to be unique (at the time of writing)
			// however they will absolutely not be very efficient
			Instruction::AllocateCell(id) => {
				let cell = alloc_tape.allocate();
				alloc_map.insert(id, cell);
			}
			Instruction::FreeCell(id) => {
				let Some(cell) = alloc_map.remove(&id) else {
					panic!("Attempted to free cell id {id} which could not be found");
				};

				alloc_tape.free(cell);
			}
			Instruction::OpenLoop(id) => {
				let Some(cell) = alloc_map.get(&id) else {
					panic!("Attempted to open loop at cell id {id} which could not be found");
				};

				ops.move_to_cell(&mut head_pos, *cell);
				ops.push(Opcode::OpenLoop);
				loop_stack.push(*cell);
			}
			Instruction::CloseLoop => {
				let Some(cell) = loop_stack.pop() else {
					panic!("Attempted to close un-opened loop");
				};

				ops.move_to_cell(&mut head_pos, cell);
				ops.push(Opcode::CloseLoop);
			}
			Instruction::AddToCell(id, imm) => {
				let Some(cell) = alloc_map.get(&id) else {
					panic!("Attempted to add to cell id {id} which could not be found");
				};

				ops.move_to_cell(&mut head_pos, *cell);

				let imm = imm as i8;
				if imm > 0 {
					for i in 0..imm {
						ops.push(Opcode::Add);
					}
				} else if imm < 0 {
					for i in 0..-imm {
						ops.push(Opcode::Subtract);
					}
				}
			}
			Instruction::OutputCell(id) => {
				let Some(cell) = alloc_map.get(&id) else {
					panic!("Attempted to output cell id {id} which could not be found");
				};

				ops.move_to_cell(&mut head_pos, *cell);
				ops.push(Opcode::Output);
			}
		}
	}

	let mut output = String::new();
	ops.into_iter()
		.for_each(|opcode| output.push_str(&opcode.to_string()));

	output
}

trait AllocationArray {
	fn allocate(&mut self) -> usize;
	fn free(&mut self, cell: usize);
}

impl AllocationArray for Vec<bool> {
	fn allocate(&mut self) -> usize {
		for i in 0..self.len() {
			if !self[i] {
				self[i] = true;
				return i;
			}
		}
		self.push(true);
		return self.len() - 1;
	}

	fn free(&mut self, cell: usize) {
		let (true, true) = (cell < self.len(), self[cell]) else {
			panic!("No allocated cell {cell} found in allocation array: {self:#?}");
		};

		self[cell] = false;

		// trim any false values at the end of the array
		for i in (0..self.len()).rev() {
			if self[i] {
				self.truncate(i + 1);
				break;
			}
		}
	}
}

pub enum Opcode {
	Add,
	Subtract,
	Right,
	Left,
	OpenLoop,
	CloseLoop,
	Output,
	Input,
}

impl Display for Opcode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Opcode::Add => "+",
			Opcode::Subtract => "-",
			Opcode::Right => ">",
			Opcode::Left => "<",
			Opcode::OpenLoop => "[",
			Opcode::CloseLoop => "]",
			Opcode::Output => ".",
			Opcode::Input => ",",
		})
	}
}

trait BrainfuckProgram {
	fn move_to_cell(&mut self, head_pos: &mut usize, cell: usize);
}

impl BrainfuckProgram for Vec<Opcode> {
	fn move_to_cell(&mut self, head_pos: &mut usize, cell: usize) {
		if *head_pos < cell {
			for _ in *head_pos..cell {
				self.push(Opcode::Right);
			}
		} else if cell < *head_pos {
			// theoretically equivalent to cell..head_pos?
			for _ in ((cell + 1)..=(*head_pos)).rev() {
				self.push(Opcode::Left);
			}
		}
		*head_pos = cell;
	}
}
