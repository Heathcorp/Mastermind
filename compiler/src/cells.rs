use std::fmt::Display;

use crate::parser::TapeCellLocation;

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
		f.write_fmt(format_args!("({}, {})", self.0, self.1))?;
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
