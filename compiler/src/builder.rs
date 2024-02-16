// turns low-level bf instructions into plain bf
// take in a timeline of cell allocations and move-to-cell operations, etc
// output plain bf according to that spec

// this algorithm is responsible for actually allocating physical tape cells as opposed to the parser
// can introduce optimisations here with some kind of allocation timeline sorting algorithm (hard leetcode style problem)

use std::{collections::HashMap, num::Wrapping};

use crate::{
	compiler::{Instruction, MemoryId},
	macros::macros::{r_assert, r_panic},
	MastermindConfig,
};

pub struct Builder<'a> {
	pub config: &'a MastermindConfig,
}

type LoopDepth = usize;
pub type TapeCell = usize;
type TapeValue = u8;

impl Builder<'_> {
	pub fn build(&self, instructions: Vec<Instruction>) -> Result<Vec<Opcode>, String> {
		let mut alloc_tape = Vec::new();
		let mut alloc_map: HashMap<MemoryId, (TapeCell, usize, LoopDepth, Vec<Option<TapeValue>>)> =
			HashMap::new();

		let mut loop_stack: Vec<TapeCell> = Vec::new();
		let mut current_loop_depth: LoopDepth = 0;
		let mut skipped_loop_depth: Option<LoopDepth> = None;
		let mut head_pos = 0;
		let mut ops: Vec<Opcode> = Vec::new();

		for instruction in instructions {
			if let Some(depth) = skipped_loop_depth {
				// current loop is being skipped because of unreachable loop optimisations
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
				Instruction::Allocate(memory, location_specifier) => {
					let cell = alloc_tape.allocate(location_specifier, memory.len())?;
					let old = alloc_map.insert(
						memory.id(),
						(
							cell,
							memory.len(),
							current_loop_depth,
							vec![Some(0); memory.len()],
						),
					);

					let None = old else {
						r_panic!("Attempted to reallocate memory {memory:#?}");
					};
				}
				Instruction::AssertCellValue(cell_obj, imm) => {
					let Some((_cell_base, size, alloc_loop_depth, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!(
							"Attempted to assert value of cell {cell_obj:#?} \
which could not be found"
						);
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let known_value = &mut known_values[mem_idx];

					// allow the user to assert that we don't know the value of the cell by clobbering when we do inline brainfuck
					if *alloc_loop_depth == current_loop_depth || imm.is_none() {
						*known_value = imm;
					} else {
						r_panic!(
							"Cannot assert cell {cell_obj:#?} value \
outside of loop it was allocated"
						);
					}
				}
				Instruction::Free(id) => {
					// TODO: do I need to check alloc loop depth here? Or are cells never freed in an inner scope?
					// think about this in regards to reusing cell space when a cell isn't being used
					let Some((cell, size, _alloc_loop_depth, known_values)) = alloc_map.remove(&id)
					else {
						r_panic!("Attempted to free memory id {id} which could not be found");
					};

					let None = known_values
						.into_iter()
						.find_map(|known_value| (known_value.unwrap_or(1) != 0).then_some(()))
					else {
						r_panic!(
							"Attempted to free memory id {id} which has unknown or non-zero values"
						);
					};

					alloc_tape.free(cell, size)?;
				}
				Instruction::OpenLoop(cell_obj) => {
					let Some((cell_base, size, alloc_loop_depth, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!(
							"Attempted to open loop at cell {cell_obj:#?} which could not be found"
						);
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;
					let known_value = &mut known_values[mem_idx];

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
						ops.move_to_cell(&mut head_pos, cell);
						ops.push(Opcode::OpenLoop);
						loop_stack.push(cell);
						current_loop_depth += 1;
					}
				}
				Instruction::CloseLoop(cell_obj) => {
					let Some((cell_base, size, alloc_loop_depth, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!(
							"Attempted to close loop at cell {cell_obj:#?} which could not be found"
						);
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;
					let known_value = &mut known_values[mem_idx];

					let Some(stack_cell) = loop_stack.pop() else {
						r_panic!("Attempted to close un-opened loop");
					};
					r_assert!(cell == stack_cell, "Attempted to close a loop unbalanced");

					current_loop_depth -= 1;

					ops.move_to_cell(&mut head_pos, cell);
					ops.push(Opcode::CloseLoop);

					// if a loop finishes on a cell then it is guaranteed to be 0 based on brainfuck itself
					// I did encounter issues with nested loops here, interesting
					if current_loop_depth == *alloc_loop_depth {
						*known_value = Some(0);
					}
				}
				Instruction::AddToCell(cell_obj, imm) => {
					let Some((cell_base, size, alloc_loop_depth, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!("Attempted to add to cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(&mut head_pos, cell);

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
				Instruction::InputToCell(cell_obj) => {
					let Some((cell_base, size, _, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!(
							"Attempted to input to cell {cell_obj:#?} which could not be found"
						);
					};

					// TODO: refactor this duplicate code (get_cell_safe or something like that)
					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(&mut head_pos, cell);
					ops.push(Opcode::Input);
					// no way to know at compile time what the input to the program will be
					*known_value = None;
				}
				// Instruction::AssertCellValue(id, value) => {}
				Instruction::ClearCell(cell_obj) => {
					let Some((cell_base, size, alloc_loop_depth, known_values)) =
						alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!("Attempted to clear cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(&mut head_pos, cell);

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
						ops.push(Opcode::Clear);
					}

					if *alloc_loop_depth == current_loop_depth {
						*known_value = Some(0);
					} else {
						// TODO: fix this for if statements
						*known_value = None;
					}
				}
				Instruction::OutputCell(cell_obj) => {
					let Some((cell_base, size, _, _)) = alloc_map.get(&cell_obj.memory_id) else {
						r_panic!("Attempted to output cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = *cell_base + mem_idx;

					ops.move_to_cell(&mut head_pos, cell);
					ops.push(Opcode::Output);
				}
				Instruction::InsertBrainfuckAtCell(operations, location_specifier) => {
					ops.move_to_cell(
						&mut head_pos,
						location_specifier.unwrap_or({
							// default position to run brainfuck in?
							// not sure what is best here, probably the last cell in the allocation tape
							let mut first_unallocated = None;
							for i in 0..alloc_tape.len() {
								match (first_unallocated, &alloc_tape[i]) {
									(None, false) => {
										first_unallocated = Some(i);
									}
									(Some(_), true) => {
										first_unallocated = None;
									}
									_ => (),
								}
							}

							first_unallocated.unwrap_or(alloc_tape.len())
						}),
					);
					ops.extend(operations);
				}
			}
		}

		Ok(ops)
	}
}

trait AllocationArray {
	fn allocate(&mut self, location: Option<TapeCell>, size: usize) -> Result<TapeCell, String>;
	fn free(&mut self, cell: TapeCell, size: usize) -> Result<(), String>;
}

impl AllocationArray for Vec<bool> {
	fn allocate(&mut self, location: Option<TapeCell>, size: usize) -> Result<TapeCell, String> {
		// should the region start at the current tape head?
		let mut region_start = location.unwrap_or(0);
		// extend the tape if the location specifier is too big
		if region_start >= self.len() {
			self.resize(region_start + 1, false);
		}

		for i in region_start..self.len() {
			if self[i] {
				// if a specifier was set, it is invalid so throw an error
				// TODO: not sure if this should throw here or not
				if let Some(l) = location {
					r_panic!("Location specifier @{l} conflicts with another allocation");
				};
				// reset search to start at next cell
				region_start = i + 1;
			} else if i - region_start == (size - 1) {
				break;
			}
		}

		for i in region_start..(region_start + size) {
			if i < self.len() {
				self[i] = true;
			} else {
				self.push(true);
			}
		}

		Ok(region_start)
	}

	fn free(&mut self, cell: TapeCell, size: usize) -> Result<(), String> {
		r_assert!(
			cell + size <= self.len(),
			"Cannot free cells {cell}..{} as allocation array is not \
large enough (this should never occur): {self:#?}",
			cell + size
		);

		for i in cell..(cell + size) {
			r_assert!(self[i], "Cannot free cell {i} as it is not allocated.");
			self[i] = false;
		}

		// trim any false values at the end of the array
		for i in (0..self.len()).rev() {
			if self[i] {
				self.truncate(i + 1);
				break;
			}
		}
		Ok(())
	}
}

#[derive(Clone, Copy, Debug)]
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

pub trait BrainfuckProgram {
	fn move_to_cell(&mut self, head_pos: &mut usize, cell: usize);
	fn from_str(s: &str) -> Self;
	fn to_string(self) -> String;
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
}
