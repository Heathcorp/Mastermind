use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub struct TapeCell(pub i32);
#[derive(PartialEq, Clone, Hash, Eq, Copy, Debug)]
pub struct TapeCell2D(pub i32, pub i32);

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
