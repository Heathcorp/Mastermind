// turns low-level bf instructions into plain bf
// take in a timeline of cell allocations and move-to-cell operations, etc
// output plain bf according to that spec

// this algorithm is responsible for actually allocating physical tape cells as opposed to the parser
// can introduce optimisations here with some kind of allocation timeline sorting algorithm (hard leetcode style problem)

use std::{
	collections::{HashMap, HashSet},
	fmt::Display,
	num::Wrapping,
};

use crate::{
	constants_optimiser::calculate_optimal_addition,
	frontend::{CellLocation, Instruction, MemoryId},
	macros::macros::{r_assert, r_panic},
	misc::MastermindContext,
};

type LoopDepth = usize;
type TapeValue = u8;

#[derive(PartialEq, Clone, Hash, Eq, Copy, Debug)]
pub struct TapeCell2D(pub i32, pub i32);

impl Display for TapeCell2D {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("({},{})", self.0, self.1))?;
		Ok(())
	}
}

impl MastermindContext {
	pub fn ir_to_bf(
		&self,
		instructions: Vec<Instruction<TapeCell2D>>,
		return_to_cell: Option<TapeCell2D>,
	) -> Result<Vec<Opcode2D>, String> {
		let mut allocator = CellAllocator::new();
		let mut alloc_map: HashMap<
			MemoryId,
			(TapeCell2D, usize, LoopDepth, Vec<Option<TapeValue>>),
		> = HashMap::new();

		let mut loop_stack: Vec<TapeCell2D> = Vec::new();
		let mut current_loop_depth: LoopDepth = 0;
		let mut skipped_loop_depth: Option<LoopDepth> = None;
		let mut ops = BFBuilder::new();

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
				// however they will absolutely not be very efficient if used directly as cell locations
				Instruction::Allocate(memory, location_specifier) => {
					let cell = allocator.allocate(
						location_specifier,
						memory.len(),
						self.config.memory_allocation_method,
					)?;
					let None = alloc_map.insert(
						memory.id(),
						(
							cell,
							memory.len(),
							current_loop_depth,
							vec![Some(0); memory.len()],
						),
					) else {
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

					allocator.free(cell, size)?;
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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
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

					// skip the loop if the optimisations are turned on and we know the value is 0
					if open {
						ops.move_to_cell(cell);
						ops.push(Opcode2D::OpenLoop);
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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
					let known_value = &mut known_values[mem_idx];

					let Some(stack_cell) = loop_stack.pop() else {
						r_panic!("Attempted to close un-opened loop");
					};
					r_assert!(cell == stack_cell, "Attempted to close a loop unbalanced");

					current_loop_depth -= 1;

					ops.move_to_cell(cell);
					ops.push(Opcode2D::CloseLoop);

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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
					let known_value = &mut known_values[mem_idx];

					// TODO: fix bug, if only one multiplication then we can have a value already in the cell, but never otherwise

					// not sure if these optimisations should be in the builder step or in the compiler
					if self.config.optimise_constants {
						// ops.move_to_cell(&mut head_pos, cell);
						// here we use an algorithm that finds the best combo of products and constants to make the number to minimise bf code
						// first we get the closest allocated cell so we can calculate the distance cost of multiplying
						// TODO: instead find the nearest zero cell, doesn't matter if allocated or not
						let temp_cell = allocator.allocate_temp_cell(cell);

						let optimised_ops: BFBuilder =
							calculate_optimal_addition(imm as i8, ops.head_pos, cell, temp_cell);

						ops.head_pos = optimised_ops.head_pos;
						ops.extend(optimised_ops.opcodes);

						allocator.free(temp_cell, 1)?;
					} else {
						ops.move_to_cell(cell);
						ops.add_to_current_cell(imm as i8);
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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(cell);
					ops.push(Opcode2D::Input);
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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(cell);

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
									ops.push(Opcode2D::Subtract);
								}
							} else if imm < 0 {
								for _ in 0..-imm {
									ops.push(Opcode2D::Add);
								}
							}
							clear = false;
						}
					}

					if clear {
						ops.push(Opcode2D::Clear);
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
					let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);

					ops.move_to_cell(cell);
					ops.push(Opcode2D::Output);
				}
				Instruction::InsertBrainfuckAtCell(operations, location_specifier) => {
					// move to the correct cell, based on the location specifier
					match location_specifier {
						CellLocation::FixedCell(cell) => ops.move_to_cell(cell),
						CellLocation::MemoryCell(cell_obj) => {
							let Some((cell_base, size, _alloc_loop_depth, _known_values)) =
								alloc_map.get(&cell_obj.memory_id)
							else {
								r_panic!("Attempted to use location of cell {cell_obj:#?} which could not be found");
							};
							let mem_idx = cell_obj.index.unwrap_or(0);
							r_assert!(
								mem_idx < *size,
								"Attempted to access memory outside of allocation"
							);
							let cell = TapeCell2D(cell_base.0 + mem_idx as i32, cell_base.1);
							ops.move_to_cell(cell);
						}
						CellLocation::Unspecified => (),
					}

					// paste the in-line BF operations
					ops.extend(operations);
				}
			}
		}

		// this is used in embedded brainfuck contexts to preserve head position
		if let Some(origin_cell) = return_to_cell {
			ops.move_to_cell(origin_cell);
		}

		Ok(ops.opcodes)
	}
}

struct CellAllocator {
	alloc_map: HashSet<TapeCell2D>,
}

// allocator will not automatically allocate negative-index cells
// but users can
impl CellAllocator {
	fn new() -> CellAllocator {
		CellAllocator {
			alloc_map: HashSet::new(),
		}
	}

	/// Checks if the memory size can be allocated to the right of a given location e.g. arrays
	fn check_allocatable(&mut self, location: &TapeCell2D, size: usize) -> bool {
		for k in 0..size {
			if self
				.alloc_map
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
		method: u8,
	) -> Result<TapeCell2D, String> {
		let mut region_start = location.unwrap_or(TapeCell2D(0, 0));
		//Check specified memory allocation above to ensure that this works nicely with all algorithms
		if let Some(l) = location {
			if !self.check_allocatable(&l, size) {
				r_panic!(
					"Location specifier @{0},{1} conflicts with another allocation",
					l.0,
					l.1
				);
			}
		} else {
			// should the region start at the current tape head?
			if method == 0 {
				for i in region_start.0.. {
					if self.alloc_map.contains(&TapeCell2D(i, region_start.1)) {
						region_start = TapeCell2D(i + 1, region_start.1);
					} else if i - region_start.0 == (size as i32 - 1) {
						break;
					}
				}
			} else if method == 1 {
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
			} else if method == 2 {
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
			} else if method == 3 {
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
				panic!("Memory Allocation Method not implemented");
			}
		}

		// make all cells in the specified region allocated
		for i in region_start.0..(region_start.0 + size as i32) {
			if !self.alloc_map.contains(&TapeCell2D(i, region_start.1)) {
				self.alloc_map.insert(TapeCell2D(i, region_start.1));
			}
		}

		Ok(region_start)
	}

	// allocate but start looking close to the given cell, used for optimising constants as you need an extra cell to multiply
	// again not sure if this stuff should be in the builder step or the compiler step ? This seems the simplest for now
	// but I'm wary that complex systems often evolve from simple ones, and any optimisations introduce complexity
	fn allocate_temp_cell(&mut self, location: TapeCell2D) -> TapeCell2D {
		// this will allocate the given cell if unallocated so beware
		if self.alloc_map.insert(location) {
			return location;
		}

		// alternate left then right, getting further and further out
		// there is surely a nice one liner rusty iterator way of doing it but somehow this is clearer until I learn that
		let mut left_iter = (0..location.0).rev();
		let mut right_iter = (location.0 + 1)..;
		loop {
			if let Some(i) = left_iter.next() {
				// unallocated cell, allocate it and return
				if self.alloc_map.insert(TapeCell2D(i, location.1)) {
					return TapeCell2D(i, location.1);
				}
			}

			if let Some(i) = right_iter.next() {
				if self.alloc_map.insert(TapeCell2D(i, location.1)) {
					return TapeCell2D(i, location.1);
				}
			}
		}
	}

	fn free(&mut self, cell: TapeCell2D, size: usize) -> Result<(), String> {
		for i in cell.0..(cell.0 + size as i32) {
			r_assert!(
				self.alloc_map.contains(&TapeCell2D(i, cell.1)),
				"Cannot free cell @{0},{1} as it is not allocated.",
				i,
				cell.1
			);
			self.alloc_map.remove(&TapeCell2D(i, cell.1));
		}

		Ok(())
	}
}

// #[derive(Clone, Copy, Debug)]
// pub enum Opcode {
// 	Add,
// 	Subtract,
// 	Right,
// 	Left,
// 	OpenLoop,
// 	CloseLoop,
// 	Output,
// 	Input,
// 	Clear,
// 	Up,
// 	Down,
// }

#[derive(Clone, Copy, Debug)]
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

pub struct BFBuilder {
	opcodes: Vec<Opcode2D>,
	pub head_pos: TapeCell2D,
}

pub trait BrainfuckOpcodes {
	fn to_string(self) -> String;
	fn from_str(s: &str) -> Self;
}

impl BrainfuckOpcodes for Vec<Opcode2D> {
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

impl BrainfuckOpcodes for BFBuilder {
	fn to_string(self) -> String {
		self.opcodes.to_string()
	}

	fn from_str(s: &str) -> Self {
		BFBuilder {
			opcodes: Vec::from_str(s),
			head_pos: TapeCell2D(0, 0),
		}
	}
}

impl BFBuilder {
	pub fn new() -> BFBuilder {
		BFBuilder {
			opcodes: Vec::new(),
			head_pos: TapeCell2D(0, 0),
		}
	}
	pub fn len(&self) -> usize {
		self.opcodes.len()
	}
	pub fn push(&mut self, op: Opcode2D) {
		self.opcodes.push(op);
	}
	pub fn extend<T>(&mut self, ops: T)
	where
		T: IntoIterator<Item = Opcode2D>,
	{
		self.opcodes.extend(ops);
	}
	pub fn move_to_cell(&mut self, cell: TapeCell2D) {
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

	pub fn add_to_current_cell(&mut self, imm: i8) {
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
}
