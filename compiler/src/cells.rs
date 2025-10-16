use std::fmt::Display;

use crate::{parser::LocationSpecifier, tokeniser::Token};

/// when making Brainfuck variants, for a cell location type, you must implement this trait
/// for now this is implemented by TapeCell (1D location specifier), and TapeCell2D (2D)
pub trait TapeCellVariant
where
	Self: PartialEq + Copy + Clone + Eq + TapeOrigin + TapeCellLocation,
{
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct TapeCell(pub i32);

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct TapeCell2D(pub i32, pub i32);

impl TapeCellVariant for TapeCell {}
impl TapeCellVariant for TapeCell2D {}

impl Display for TapeCell {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.0))?;
		Ok(())
	}
}

impl Display for TapeCell2D {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("({},{})", self.0, self.1))?;
		Ok(())
	}
}

pub trait TapeOrigin {
	fn origin_cell() -> Self;
}

impl TapeOrigin for TapeCell {
	fn origin_cell() -> TapeCell {
		TapeCell(0)
	}
}
impl TapeOrigin for TapeCell2D {
	fn origin_cell() -> TapeCell2D {
		TapeCell2D(0, 0)
	}
}

pub trait TapeCellLocation
where
	Self: Sized + Display,
{
	/// parse any memory location specifiers
	/// let g @(4,2) = 68;
	/// or
	/// let p @3 = 68;
	fn parse_location_specifier(
		tokens: &[Token],
	) -> Result<(LocationSpecifier<Self>, usize), String>;

	/// safely cast a 2D or 1D location specifier into a 1D non-negative cell offset,
	///  for use with struct fields
	fn to_positive_cell_offset(&self) -> Result<usize, String>;
}
