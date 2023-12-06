// turns low-level bf instructions into plain bf
// take in a timeline of cell allocations and move-to-cell operations, etc
// output plain bf according to that spec

// this algorithm is responsible for actually allocating physical tape cells as opposed to the parser
// can introduce optimisations here with some kind of allocation timeline sorting algorithm (hard leetcode style problem)

use std::{collections::HashMap, fmt::Display, num::Wrapping};

use crate::{compiler::Instruction, MastermindConfig};

pub struct Builder<'a> {
	pub config: &'a MastermindConfig,
}

pub type CellId = usize;
type LoopDepth = usize;
type TapeCell = usize;
type TapeValue = u8;

impl Builder<'_> {
	pub fn build(&self, instructions: Vec<Instruction>) -> String {
		let mut alloc_tape = Vec::new();
		let mut alloc_map: HashMap<CellId, (TapeCell, LoopDepth, Option<TapeValue>)> =
			HashMap::new();

		let mut loop_stack: Vec<TapeCell> = Vec::new();
		let mut current_loop_depth: LoopDepth = 0;
		let mut skipped_loop_depth: Option<LoopDepth> = None;
		let mut head_pos = 0;
		let mut ops: Vec<Opcode> = Vec::new();

		for instruction in instructions {
			// TODO: skipped loop
			if let Some(depth) = skipped_loop_depth {
				match instruction {
					Instruction::OpenLoop(_) => {
						current_loop_depth += 1;
					}
					Instruction::CloseLoop(_) => {
						current_loop_depth -= 1;
						if current_loop_depth == depth {
							skipped_loop_depth = None;
						}
					}
					_ => (),
				}
				continue;
			}
			match instruction {
				// the ids (indices really) given by the compiler are guaranteed to be unique (at the time of writing)
				// however they will absolutely not be very efficient
				Instruction::AllocateCell(id) => {
					let cell = alloc_tape.allocate();
					let old = alloc_map.insert(id, (cell, current_loop_depth, Some(0)));

					let None = old else {
						panic!("Attempted to reallocate cell id {id}");
					};
				}
				Instruction::FreeCell(id) => {
					// TODO: do I need to check alloc loop depth here? Or are cells never freed in an inner scope?
					// think about this in regards to reusing cell space when a cell isn't being used
					let Some((cell, _alloc_loop_depth, known_value)) = alloc_map.remove(&id) else {
						panic!("Attempted to free cell id {id} which could not be found");
					};

					let Some(0) = known_value else {
						panic!(
							"Attempted to free cell id {id} which has an unknown or non-zero value"
						);
					};

					alloc_tape.free(cell);
				}
				Instruction::OpenLoop(id) => {
					let Some((cell, alloc_loop_depth, known_value)) = alloc_map.get_mut(&id) else {
						panic!("Attempted to open loop at cell id {id} which could not be found");
					};

					let mut open = true;

					if let Some(known_value) = known_value {
						if *alloc_loop_depth == current_loop_depth
							&& *known_value == 0 && self.config.optimise_unreachable_loops
						{
							open = false;
							skipped_loop_depth = Some(current_loop_depth);
							current_loop_depth += 1;
						}
					}

					if open {
						ops.move_to_cell(&mut head_pos, *cell);
						ops.push(Opcode::OpenLoop);
						loop_stack.push(*cell);
						current_loop_depth += 1;
					}
				}
				Instruction::CloseLoop(id) => {
					let Some((cell, _, known_value)) = alloc_map.get_mut(&id) else {
						panic!("Attempted to close loop at cell id {id} which could not be found");
					};
					let Some(stack_cell) = loop_stack.pop() else {
						panic!("Attempted to close un-opened loop");
					};
					assert!(*cell == stack_cell, "Attempted to close a loop unbalanced");

					current_loop_depth -= 1;

					ops.move_to_cell(&mut head_pos, *cell);
					ops.push(Opcode::CloseLoop);

					// if a loop finishes on a cell then it is guaranteed to be 0 based on brainfuck itself
					// TODO: is this an issue if in a nested loop?
					*known_value = Some(0);
				}
				Instruction::AddToCell(id, imm) => {
					let Some((cell, alloc_loop_depth, known_value)) = alloc_map.get_mut(&id) else {
						panic!("Attempted to add to cell id {id} which could not be found");
					};

					ops.move_to_cell(&mut head_pos, *cell);

					let i_imm = imm as i8;
					if i_imm > 0 {
						for _ in 0..i_imm {
							ops.push(Opcode::Add);
						}
					} else if i_imm < 0 {
						for _ in 0..-i_imm {
							ops.push(Opcode::Subtract);
						}
					}

					if imm != 0 {
						if *alloc_loop_depth != current_loop_depth {
							*known_value = None;
						} else if let Some(known_value) = known_value {
							*known_value = (Wrapping(*known_value) + Wrapping(imm)).0;
						}
					}
				}
				// Instruction::AssertCellValue(id, value) => {}
				Instruction::ClearCell(id) => {
					let Some((cell, alloc_loop_depth, known_value)) = alloc_map.get_mut(&id) else {
						panic!("Attempted to clear cell id {id} which could not be found");
					};

					ops.move_to_cell(&mut head_pos, *cell);

					let mut clear = true;

					if let Some(known_value) = known_value {
						if self.config.optimise_cell_clearing
							&& *alloc_loop_depth == current_loop_depth
							&& (*known_value as i8).abs() < 4
						// not sure if this should be 4 or 3, essentially it depends on if we prefer clears or changes [-] vs ++---
						{
							let imm = *known_value as i8;
							if imm > 0 {
								for _ in 0..imm {
									ops.push(Opcode::Subtract);
								}
							} else if imm < 0 {
								for _ in 0..-imm {
									ops.push(Opcode::Add);
								}
							}
							clear = false;
						}
					}

					if clear {
						ops.push(Opcode::OpenLoop);
						ops.push(Opcode::Subtract);
						ops.push(Opcode::CloseLoop);
					}

					*known_value = Some(0);
				}
				Instruction::OutputCell(id) => {
					let Some((cell, _, _)) = alloc_map.get(&id) else {
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
}

trait AllocationArray {
	fn allocate(&mut self) -> TapeCell;
	fn free(&mut self, cell: TapeCell);
}

impl AllocationArray for Vec<bool> {
	fn allocate(&mut self) -> TapeCell {
		for i in 0..self.len() {
			if !self[i] {
				self[i] = true;
				return i;
			}
		}
		self.push(true);
		return self.len() - 1;
	}

	fn free(&mut self, cell: TapeCell) {
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
