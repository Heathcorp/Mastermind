use crate::{
	macros::macros::*,
	parser::types::{Clause, Reference, VariableTargetReferenceChain},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Instruction<TC, OC> {
	Allocate(Memory, Option<TC>),
	Free(MemoryId), // the number indicates which cell in the allocation stack should be freed (cell 0, is the top of the stack, 1 is the second element, etc)
	OpenLoop(CellReference), // same with other numbers here, they indicate the cell in the allocation stack to use in the instruction
	CloseLoop(CellReference), // pass in the cell id, this originally wasn't there but may be useful later on
	AddToCell(CellReference, u8),
	InputToCell(CellReference),
	ClearCell(CellReference), // not sure if this should be here, seems common enough that it should be
	AssertCellValue(CellReference, Option<u8>), // allows the user to hand-tune optimisations further
	OutputCell(CellReference),
	InsertBrainfuckAtCell(Vec<OC>, CellLocation<TC>),
}

#[derive(Debug, Clone)]
/// Either a fixed constant cell or a reference to some existing memory
pub enum CellLocation<TC> {
	Unspecified,
	FixedCell(TC),
	MemoryCell(CellReference),
}

#[derive(Debug, Clone)]
pub enum Memory {
	Cell {
		id: MemoryId,
	},
	Cells {
		id: MemoryId,
		len: usize,
	},
	/// A memory cell that references a previously allocated cell in an outer scope, used for function arguments
	MappedCell {
		id: MemoryId,
		index: Option<usize>,
	},
	/// Memory mapped cells, referencing previously allocated cells in an outer scope
	MappedCells {
		id: MemoryId,
		start_index: usize,
		len: usize,
	},
	// infinite cell something (TODO?)
}
pub type MemoryId = usize;

#[derive(Debug, Clone, Copy)]
pub struct CellReference {
	pub memory_id: MemoryId,
	pub index: Option<usize>,
}

impl Memory {
	pub fn id(&self) -> MemoryId {
		match self {
			Memory::Cell { id }
			| Memory::Cells { id, len: _ }
			| Memory::MappedCell { id, index: _ }
			| Memory::MappedCells {
				id,
				start_index: _,
				len: _,
			} => *id,
		}
	}
	pub fn len(&self) -> usize {
		match self {
			Memory::Cell { id: _ } | Memory::MappedCell { id: _, index: _ } => 1,
			Memory::Cells { id: _, len }
			| Memory::MappedCells {
				id: _,
				start_index: _,
				len,
			} => *len,
		}
	}
}

#[derive(Clone, Debug)] // probably shouldn't be cloning here but whatever
pub struct Function<TC, OC> {
	pub arguments: Vec<(String, ValueType)>,
	pub block: Vec<Clause<TC, OC>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// an absolute definition of a type, as opposed to `VariableTypeReference` which is more of a reference
pub enum ValueType {
	Cell,
	Array(usize, Box<ValueType>),
	DictStruct(Vec<(String, ValueType, Option<usize>)>),
	// TupleStruct(Vec<ValueType>),
}

#[derive(Clone, Debug)]
/// equivalent to ValueType::DictStruct enum variant,
/// Rust doesn't support enum variants as types yet so need this workaround for struct definitions in scope object
pub struct DictStructType(pub Vec<(String, ValueType, Option<usize>)>);
impl ValueType {
	pub fn from_struct(struct_def: DictStructType) -> Self {
		ValueType::DictStruct(struct_def.0)
	}

	// TODO: make size() and get_and_validate_subfield_cell_map() more efficient,
	//  currently these two recurse back and forth and are a bit of a monster combo

	/// return the type size in cells
	pub fn size(&self) -> Result<usize, String> {
		Ok(match self {
			ValueType::Cell => 1,
			ValueType::Array(len, value_type) => *len * value_type.size()?,
			ValueType::DictStruct(fields) => Self::get_and_validate_subfield_cell_map(fields)?.1,
		})
	}

	/// deterministically place all struct subfields on a non-negative cell, return the positions of each and the total length
	/// return Err() if location specified subfields overlap
	pub fn get_and_validate_subfield_cell_map(
		fields: &Vec<(String, ValueType, Option<usize>)>,
	) -> Result<(HashMap<&String, (usize, &ValueType)>, usize), String> {
		// (set of cells, max cell)
		let mut cell_map = HashMap::new();

		// map of field names and their starting cells
		let mut subfield_map = HashMap::new();
		let mut max_cell = 0usize;
		let mut unfixed_fields = vec![];
		// handle the cells with specified locations
		for (field_name, field_type, field_location) in fields {
			match field_location {
				Some(location) => {
					subfield_map.insert(field_name, (*location, field_type));
					for cell_index in *location..(*location + field_type.size()?) {
						// this assumes the field locations have been validated
						if let Some(other_name) = cell_map.insert(cell_index, field_name) {
							r_panic!(
									"Subfields \"{other_name}\" and \"{field_name}\" overlap in struct."
								);
						};
						max_cell = max_cell.max(cell_index);
					}
				}
				None => {
					unfixed_fields.push((field_name, field_type));
				}
			}
		}

		for (field_name, field_type) in unfixed_fields {
			let field_size = field_type.size()?;
			// repeatedly try to insert the fields into leftover memory locations
			let mut start_index = 0usize;
			for cur_index in 0.. {
				if cell_map.contains_key(&cur_index) {
					start_index = cur_index + 1;
				} else if (cur_index - start_index + 1) >= field_size {
					// found a run with the right amount of cells free
					break;
				}
			}
			subfield_map.insert(field_name, (start_index, field_type));
			for cell_index in start_index..(start_index + field_size) {
				// inefficient but whatever, this insert is not necessary
				cell_map.insert(cell_index, field_name);
				max_cell = max_cell.max(cell_index);
			}
		}

		let size = max_cell + 1;

		Ok((subfield_map, size))
	}

	/// get a subfield's type as well as memory cell index
	pub fn get_subfield(
		&self,
		subfield_chain: &VariableTargetReferenceChain,
	) -> Result<(&ValueType, usize), String> {
		let mut cur_field = self;
		let mut cur_index = 0;
		for subfield_ref in subfield_chain.0.iter() {
			match (cur_field, subfield_ref) {
				(ValueType::Array(len, element_type), Reference::Index(index)) => {
					r_assert!(
						index < len,
						"Index \"{subfield_ref}\" must be less than array length ({len})."
					);
					cur_index += element_type.size()? * index;
					cur_field = element_type;
				}
				(ValueType::DictStruct(fields), Reference::NamedField(subfield_name)) => {
					let (subfield_map, _size) = Self::get_and_validate_subfield_cell_map(fields)?;
					let Some((subfield_cell_offset, subfield_type)) =
						subfield_map.get(subfield_name)
					else {
						r_panic!("Could not find subfield \"{subfield_ref}\" in struct type")
					};
					cur_index += subfield_cell_offset;
					cur_field = subfield_type;
				}

				(ValueType::DictStruct(_), Reference::Index(_)) => {
					r_panic!("Cannot read index subfield \"{subfield_ref}\" of struct type.")
				}
				(ValueType::Array(_, _), Reference::NamedField(_)) => {
					r_panic!("Cannot read named subfield \"{subfield_ref}\" of array type.")
				}
				(ValueType::Cell, subfield_ref) => {
					r_panic!("Attempted to get subfield \"{subfield_ref}\" of cell type.")
				}
			}
		}
		Ok((cur_field, cur_index))
	}
}

impl std::fmt::Display for ValueType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ValueType::Cell => {
				f.write_str("cell")?;
			}
			ValueType::Array(length, element_type) => {
				f.write_fmt(format_args!("{element_type}[{length}]"))?;
			}
			ValueType::DictStruct(fields) => {
				f.write_str("{")?;
				let fields_len = fields.len();
				for (i, (field_name, field_type, offset)) in fields.iter().enumerate() {
					f.write_fmt(format_args!("{field_type} {field_name}"))?;
					if let Some(offset) = offset {
						f.write_fmt(format_args!(" @{offset}"))?;
					}
					f.write_str(";")?;
					if i < (fields_len - 1) {
						f.write_str(" ")?;
					}
				}
			}
		}
		Ok(())
	}
}
