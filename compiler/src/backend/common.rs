use super::constants_optimiser::calculate_optimal_addition;
use crate::{
	frontend::types::{CellLocation, Instruction, MemoryId},
	macros::macros::{r_assert, r_panic},
	misc::{MastermindConfig, MastermindContext},
	parser::types::TapeCellLocation,
};

use std::{
	collections::{HashMap, HashSet},
	num::Wrapping,
};

type LoopDepth = usize;
type TapeValue = u8;

impl<'a> MastermindContext {
	pub fn ir_to_bf<TC: TapeCellVariant, OC: OpcodeVariant>(
		&self,
		instructions: Vec<Instruction<TC, OC>>,
		return_to_cell: Option<TC>,
	) -> Result<Vec<OC>, String>
	where
		BrainfuckBuilderData<TC, OC>: BrainfuckBuilder<TC, OC>,
		CellAllocatorData<TC>: CellAllocator<TC>,
	{
		let mut allocator = CellAllocatorData::new(self.config.clone());

		struct AllocationMapEntry<TC> {
			cell_base: TC,
			size: usize,
			alloc_loop_depth: LoopDepth,
			known_values: Vec<Option<TapeValue>>,
		}
		let mut alloc_map: HashMap<MemoryId, AllocationMapEntry<TC>> = HashMap::new();

		let mut loop_stack: Vec<TC> = Vec::new();
		let mut current_loop_depth: LoopDepth = 0;
		let mut skipped_loop_depth: Option<LoopDepth> = None;
		let mut ops = BrainfuckBuilderData::new();

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
					let cell = allocator.allocate(location_specifier, memory.len())?;
					let None = alloc_map.insert(
						memory.id(),
						AllocationMapEntry {
							cell_base: cell,
							size: memory.len(),
							alloc_loop_depth: current_loop_depth,
							known_values: vec![Some(0); memory.len()],
						},
					) else {
						r_panic!("Attempted to reallocate memory {memory:#?}");
					};
				}
				Instruction::AssertCellValue(cell_obj, imm) => {
					let Some(AllocationMapEntry {
						cell_base: _,
						size,
						alloc_loop_depth,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
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
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth: _,
						known_values,
					}) = alloc_map.remove(&id)
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

					allocator.free(cell_base, size)?;
				}
				Instruction::OpenLoop(cell_obj) => {
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
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
					let cell = cell_base.with_offset(mem_idx as i32);
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
						ops.open_loop();
						loop_stack.push(cell);
						current_loop_depth += 1;
					}
				}
				Instruction::CloseLoop(cell_obj) => {
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
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
					let cell = cell_base.with_offset(mem_idx as i32);
					let known_value = &mut known_values[mem_idx];

					let Some(stack_cell) = loop_stack.pop() else {
						r_panic!("Attempted to close un-opened loop");
					};
					r_assert!(cell == stack_cell, "Attempted to close a loop unbalanced");

					current_loop_depth -= 1;

					ops.move_to_cell(cell);
					ops.close_loop();

					// if a loop finishes on a cell then it is guaranteed to be 0 based on brainfuck itself
					// I did encounter issues with nested loops here, interesting
					if current_loop_depth == *alloc_loop_depth {
						*known_value = Some(0);
					}
				}
				Instruction::AddToCell(cell_obj, imm) => {
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!("Attempted to add to cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = cell_base.with_offset(mem_idx as i32);
					let known_value = &mut known_values[mem_idx];

					// TODO: fix bug, if only one multiplication then we can have a value already in the cell, but never otherwise

					// not sure if these optimisations should be in the builder step or in the compiler
					if self.config.optimise_constants {
						// ops.move_to_cell(&mut head_pos, cell);
						// here we use an algorithm that finds the best combo of products and constants to make the number to minimise bf code
						// first we get the closest allocated cell so we can calculate the distance cost of multiplying
						// TODO: instead find the nearest zero cell, doesn't matter if allocated or not
						let temp_cell = allocator.allocate_temp_cell(cell);

						let optimised_ops =
							calculate_optimal_addition(imm as i8, ops.head_pos, cell, temp_cell);

						ops.extend(optimised_ops.opcodes);
						ops.head_pos = optimised_ops.head_pos;

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
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth: _,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
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
					let cell = cell_base.with_offset(mem_idx as i32);
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(cell);
					ops.input_to_current_cell();
					// no way to know at compile time what the input to the program will be
					*known_value = None;
				}
				// Instruction::AssertCellValue(id, value) => {}
				Instruction::ClearCell(cell_obj) => {
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth,
						known_values,
					}) = alloc_map.get_mut(&cell_obj.memory_id)
					else {
						r_panic!("Attempted to clear cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = cell_base.with_offset(mem_idx as i32);
					let known_value = &mut known_values[mem_idx];

					ops.move_to_cell(cell);

					let mut clear = true;

					if let Some(known_value) = known_value {
						if self.config.optimise_cell_clearing
							&& *alloc_loop_depth == current_loop_depth
							// not sure if this should be 4 or 3, essentially it depends on if we prefer clears or changes [-] vs ++---
							&& (*known_value as i8).abs() < 4
						{
							// 	let imm = *known_value as i8;
							// 	if imm > 0 {
							// 		for _ in 0..imm {
							// 			ops.push(Opcode2D::Subtract);
							// 		}
							// 	} else if imm < 0 {
							// 		for _ in 0..-imm {
							// 			ops.push(Opcode2D::Add);
							// 		}
							// 	}
							ops.add_to_current_cell(-(*known_value as i8));
							clear = false;
						}
					}

					if clear {
						ops.clear_current_cell();
					}

					if *alloc_loop_depth == current_loop_depth {
						*known_value = Some(0);
					} else {
						// TODO: fix this for if statements
						*known_value = None;
					}
				}
				Instruction::OutputCell(cell_obj) => {
					let Some(AllocationMapEntry {
						cell_base,
						size,
						alloc_loop_depth: _,
						known_values: _,
					}) = alloc_map.get(&cell_obj.memory_id)
					else {
						r_panic!("Attempted to output cell {cell_obj:#?} which could not be found");
					};

					let mem_idx = cell_obj.index.unwrap_or(0);
					r_assert!(
						mem_idx < *size,
						"Attempted to access memory outside of allocation"
					);
					let cell = cell_base.with_offset(mem_idx as i32);

					ops.move_to_cell(cell);
					ops.output_current_cell();
				}
				Instruction::InsertBrainfuckAtCell(operations, location_specifier) => {
					// move to the correct cell, based on the location specifier
					match location_specifier {
						CellLocation::FixedCell(cell) => ops.move_to_cell(cell.into()),
						CellLocation::MemoryCell(cell_obj) => {
							let Some(AllocationMapEntry {
								cell_base,
								size,
								alloc_loop_depth: _,
								known_values: _,
							}) = alloc_map.get(&cell_obj.memory_id)
							else {
								r_panic!("Attempted to use location of cell {cell_obj:#?} which could not be found");
							};
							let mem_idx = cell_obj.index.unwrap_or(0);
							r_assert!(
								mem_idx < *size,
								"Attempted to access memory outside of allocation"
							);
							let cell = cell_base.with_offset(mem_idx as i32);
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
			ops.move_to_cell(origin_cell.into());
		}

		Ok(ops.opcodes)
	}
}

/// This trait must be implemented for a cell location type for a Brainfuck variant
/// for now this is implemented by TapeCell (i32 1D location specifier), and TapeCell2D (2D)
pub trait TapeCellVariant
where
	Self: PartialEq + Copy + Clone + Eq + TapeCellLocation,
{
	fn origin_cell() -> Self;
	fn with_offset(&self, offset: i32) -> Self;
}

/// This trait must be implemented for a Brainfuck variant
pub trait OpcodeVariant
where
	Self: Sized + Clone + Copy,
{
	fn try_from_char(c: char) -> Option<Self>;
}

pub struct CellAllocatorData<TC> {
	pub cells: HashSet<TC>,
	pub config: MastermindConfig,
}
impl<T> CellAllocatorData<T> {
	fn new(config: MastermindConfig) -> CellAllocatorData<T> {
		CellAllocatorData {
			cells: HashSet::new(),
			config,
		}
	}
}

pub trait CellAllocator<TC> {
	fn check_allocatable(&mut self, location: &TC, size: usize) -> bool;
	fn allocate(&mut self, location: Option<TC>, size: usize) -> Result<TC, String>;
	fn allocate_temp_cell(&mut self, location: TC) -> TC;
	fn free(&mut self, cell: TC, size: usize) -> Result<(), String>;
}

pub struct BrainfuckBuilderData<TC, OC> {
	pub opcodes: Vec<OC>,
	pub head_pos: TC,
}

pub trait BrainfuckBuilder<TC, OC> {
	fn new() -> Self;
	fn len(&self) -> usize;
	fn push(&mut self, op: OC);
	fn extend<T>(&mut self, ops: T)
	where
		T: IntoIterator<Item = OC>;
	fn move_to_cell(&mut self, cell: TC);
	fn add_to_current_cell(&mut self, imm: i8);
	fn clear_current_cell(&mut self);
	fn output_current_cell(&mut self);
	fn input_to_current_cell(&mut self);
	fn open_loop(&mut self);
	fn close_loop(&mut self);
}

pub trait BrainfuckProgram {
	fn to_string(self) -> String;
	fn from_str(s: &str) -> Self;
}
