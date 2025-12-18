use super::expressions::Expression;
use crate::macros::macros::r_panic;

/// Clause type type variables:
/// - TC: TapeCell can be changed to implement 2D brainfuck, or other modifications
/// - OC: Opcode represents the valid Brainfuck Opcodes that we're generating (also used for 2D or other BF variants)
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Clause<TC, OC> {
	None,
	DeclareVariable {
		var: VariableTypeDefinition<TC>,
	},
	DefineVariable {
		var: VariableTypeDefinition<TC>,
		value: Expression,
	},
	DefineStruct {
		name: String,
		fields: Vec<StructFieldTypeDefinition>,
	},
	AddAssign {
		var: VariableTarget,
		value: Expression,
		self_referencing: bool,
	},
	Assign {
		var: VariableTarget,
		value: Expression,
		self_referencing: bool,
	},
	AssertVariableValue {
		var: VariableTarget,
		// Some(constant) indicates we know the value, None indicates we don't know the value
		// typically will either be used for assert unknown or assert 0
		value: Option<Expression>,
	},
	DrainLoop {
		source: Expression,
		targets: Vec<VariableTarget>,
		block: Option<Vec<Clause<TC, OC>>>,
		// TODO: reassess this syntax
		is_copying: bool,
	},
	While {
		var: VariableTarget,
		block: Vec<Clause<TC, OC>>,
	},
	Output {
		value: Expression,
	},
	Input {
		var: VariableTarget,
	},
	DefineFunction {
		name: String,
		// TODO: fix the type here, as function definitions don't actually need location specifiers and therefore don't need a tape cell type
		arguments: Vec<VariableTypeDefinition<TC>>,
		block: Vec<Clause<TC, OC>>,
	},
	CallFunction {
		function_name: String,
		arguments: Vec<Expression>,
	},
	If {
		condition: Expression,
		if_block: Vec<Clause<TC, OC>>,
	},
	IfNot {
		condition: Expression,
		if_not_block: Vec<Clause<TC, OC>>,
	},
	IfElse {
		condition: Expression,
		if_block: Vec<Clause<TC, OC>>,
		else_block: Vec<Clause<TC, OC>>,
	},
	IfNotElse {
		condition: Expression,
		if_not_block: Vec<Clause<TC, OC>>,
		else_block: Vec<Clause<TC, OC>>,
	},
	Block(Vec<Clause<TC, OC>>),
	Brainfuck {
		location_specifier: LocationSpecifier<TC>,
		clobbered_variables: Vec<VariableTarget>,
		operations: Vec<ExtendedOpcode<TC, OC>>,
	},
}

pub trait TapeCellLocation
where
	Self: Sized + std::fmt::Display,
{
	/// optionally parse a memory location specifier
	/// let g @(4,2) = 68;
	/// or
	/// let p @3 = 68;
	fn parse_location_specifier(chars: &mut &[char]) -> Result<LocationSpecifier<Self>, String>;

	/// safely cast a 2D or 1D location specifier into a 1D non-negative cell offset,
	///  for use with struct fields
	fn to_positive_cell_offset(&self) -> Result<usize, String>;
}

// extended brainfuck opcodes to include mastermind code blocks
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ExtendedOpcode<TC, OC> {
	Opcode(OC),
	Block(Vec<Clause<TC, OC>>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// the type of a variable according to the user, not validated yet as the parser does not keep track of types
pub enum VariableTypeReference {
	Cell,
	Struct(String),
	Array(Box<VariableTypeReference>, usize),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LocationSpecifier<TC> {
	None,
	Cell(TC),
	Variable(VariableTarget),
}
impl<T> LocationSpecifier<T> {
	fn is_none(&self) -> bool {
		matches!(self, LocationSpecifier::None)
	}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableTypeDefinition<TC> {
	pub name: String,
	pub var_type: VariableTypeReference,
	pub location_specifier: LocationSpecifier<TC>,
	// Infinite {name: String, pattern: ???},
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldTypeDefinition {
	pub name: String,
	pub field_type: VariableTypeReference,
	pub location_offset_specifier: Option<usize>,
}
// let non_neg_location_specifier = match &var_def.location_specifier {
// 	LocationSpecifier::None => None,
// 	LocationSpecifier::Cell(l) => {
// 		// assert the y coordinate is 0
// 		// r_assert!(
// 		// 	l.1 == 0,
// 		// 	"Struct field location specifiers do not support 2D grid cells: {var_def}"
// 		// );
// 		r_assert!(
// 			l.0 >= 0,
// 			"Struct field location specifiers must be non-negative: {var_def}"
// 		);
// 		Some(l.0 as usize)
// 	}
// 	LocationSpecifier::Variable(_) => {
// 		r_panic!( "Location specifiers in struct definitions must be relative, not variables: {var_def}")
// 	}
// };
impl<TC> TryInto<StructFieldTypeDefinition> for VariableTypeDefinition<TC>
where
	TC: TapeCellLocation,
{
	type Error = String;

	fn try_into(self) -> Result<StructFieldTypeDefinition, String> {
		let location_offset_specifier = match &self.location_specifier {
			LocationSpecifier::None => None,
			LocationSpecifier::Cell(cell) => Some(match cell.to_positive_cell_offset() {
				Ok(offset) => offset,
				Err(err) => r_panic!("Cannot create struct field \"{self}\". {err}"),
			}),
			LocationSpecifier::Variable(_) => r_panic!(
				"Location specifiers in struct definitions \
must be relative, not variable."
			),
		};
		Ok(StructFieldTypeDefinition {
			name: self.name,
			field_type: self.var_type,
			location_offset_specifier,
		})
	}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Reference {
	NamedField(String),
	Index(usize),
}

/// Represents a list of subfield references after the `.` or `[x]` operators, e.g. `obj.h[6]` would have `['h', '[6]']`
// a bit verbose, not quite sure about this
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableTargetReferenceChain(pub Vec<Reference>);
/// Represents a target variable in an expression, this has no type informatino
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableTarget {
	pub name: String,
	pub subfields: Option<VariableTargetReferenceChain>,
	pub is_spread: bool,
}
impl VariableTarget {
	/// convert a definition to a target for use with definition clauses (as opposed to declarations)
	pub fn from_definition<T>(var_def: &VariableTypeDefinition<T>) -> Self {
		VariableTarget {
			name: var_def.name.clone(),
			subfields: None,
			is_spread: false,
		}
	}
}

impl std::fmt::Display for VariableTypeReference {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self {
			VariableTypeReference::Cell => f.write_str("cell"),
			VariableTypeReference::Struct(struct_name) => {
				f.write_fmt(format_args!("struct {struct_name}"))
			}
			VariableTypeReference::Array(element_type, len) => {
				f.write_fmt(format_args!("{element_type}[{len}]"))
			}
		}
	}
}

impl<T: std::fmt::Display> std::fmt::Display for VariableTypeDefinition<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{} {}", self.var_type, self.name))?;
		match &self.location_specifier {
			LocationSpecifier::Cell(_) | LocationSpecifier::Variable(_) => {
				f.write_str(&format!(" {}", self.location_specifier))?
			}
			LocationSpecifier::None => (),
		}

		Ok(())
	}
}

impl<T: std::fmt::Display> std::fmt::Display for LocationSpecifier<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("@")?;
		match self {
			LocationSpecifier::Cell(cell) => f.write_str(&format!("{cell}"))?,
			LocationSpecifier::Variable(var) => f.write_str(&format!("{var}"))?,
			LocationSpecifier::None => (),
		}

		Ok(())
	}
}

impl std::fmt::Display for Reference {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Reference::NamedField(subfield_name) => f.write_str(&format!(".{subfield_name}"))?,
			Reference::Index(index) => f.write_str(&format!("[{index}]"))?,
		}

		Ok(())
	}
}

impl std::fmt::Display for VariableTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_spread {
			f.write_str("*")?;
		}
		f.write_str(&self.name)?;
		if let Some(subfield_refs) = &self.subfields {
			for ref_step in subfield_refs.0.iter() {
				f.write_str(&format!("{ref_step}"))?;
			}
		}

		Ok(())
	}
}
