use super::{
	expressions::Expression,
	tokeniser::{next_token, Token},
	types::{
		Clause, LocationSpecifier, Reference, TapeCellLocation, VariableTarget,
		VariableTargetReferenceChain, VariableTypeReference,
	},
};
use crate::{
	backend::{
		bf::TapeCell,
		bf2d::TapeCell2D,
		common::{OpcodeVariant, TapeCellVariant},
	},
	macros::macros::{r_assert, r_panic},
	parser::types::VariableTypeDefinition,
};

pub fn parse_program<TC: TapeCellVariant, OC: OpcodeVariant>(
	raw: &str,
) -> Result<Vec<Clause<TC, OC>>, String> {
	let program_chars: Vec<char> = raw.chars().collect();
	let mut chars_slice = &program_chars[..];
	let mut clauses = vec![];
	loop {
		let clause = parse_clause(&mut chars_slice)?;
		if let Clause::None = clause {
			break;
		}
		clauses.push(clause);
	}

	Ok(clauses)
}

fn parse_clause<TC: TapeCellVariant, OC: OpcodeVariant>(
	chars: &mut &[char],
) -> Result<Clause<TC, OC>, String> {
	let mut s = *chars;
	Ok(match next_token(&mut s)? {
		Token::None => Clause::None,
		Token::If => parse_if_else_clause(chars)?,
		Token::While => parse_while_clause(chars)?,
		Token::Fn => parse_function_definition_clause(chars)?,
		Token::Struct => {
			let Token::Name(_) = next_token(&mut s)? else {
				// TODO: add source snippet
				r_panic!("Expected identifier after `struct` keyword.");
			};
			match next_token(&mut s)? {
				Token::LeftBrace => parse_struct_definition_clause(chars)?,
				_ => parse_let_clause(chars)?,
			}
		}
		Token::Cell => parse_let_clause(chars)?,
		Token::Name(_) => match next_token(&mut s)? {
			Token::LeftParenthesis => todo!(),
			_ => todo!(),
		},
		token => r_panic!("Invalid starting token: {token:?}"),
	})
}

fn parse_block<TC: TapeCellVariant, OC: OpcodeVariant>(
	chars: &mut &[char],
) -> Result<Vec<Clause<TC, OC>>, String> {
	let Token::LeftBrace = next_token(chars)? else {
		r_panic!("Expected `{{` in code block.");
	};

	let mut clauses = vec![];
	loop {
		{
			let mut s = *chars;
			if let Token::RightBrace = next_token(&mut s)? {
				break;
			}
		}
		let clause = parse_clause(chars)?;
		if let Clause::None = clause {
			break;
		}
		clauses.push(clause);
	}

	Ok(clauses)
}

////////////////////////////
////////////////////////////
////////////////////////////

impl TapeCellLocation for TapeCell {
	fn parse_location_specifier(
		chars: &mut &[char],
	) -> Result<LocationSpecifier<TapeCell>, String> {
		let mut s = *chars;
		let Token::At = next_token(&mut s)? else {
			return Ok(LocationSpecifier::None);
		};
		*chars = s;

		match next_token(&mut s)? {
			Token::Minus | Token::Number(_) => Ok(LocationSpecifier::Cell(parse_integer(chars)?)),
			// variable location specifier:
			Token::Name(_) => Ok(LocationSpecifier::Variable(parse_var_target(chars)?)),
			// TODO: add source snippet
			_ => r_panic!("Invalid location specifier.",),
		}
	}

	fn to_positive_cell_offset(&self) -> Result<usize, String> {
		r_assert!(*self >= 0, "Expected non-negative cell offset.");
		Ok(*self as usize)
	}
}

impl TapeCellLocation for TapeCell2D {
	fn parse_location_specifier(
		chars: &mut &[char],
	) -> Result<LocationSpecifier<TapeCell2D>, String> {
		let mut s = *chars;
		let Token::At = next_token(&mut s)? else {
			return Ok(LocationSpecifier::None);
		};
		*chars = s;

		match next_token(&mut s)? {
			Token::LeftParenthesis => {
				// parse a 2-tuple
				let tuple = parse_integer_tuple::<2>(chars)?;
				Ok(LocationSpecifier::Cell(TapeCell2D(tuple[0], tuple[1])))
			}
			Token::Minus | Token::Number(_) => Ok(LocationSpecifier::Cell(TapeCell2D(
				parse_integer(chars)?,
				0,
			))),
			// variable location specifier:
			Token::Name(_) => Ok(LocationSpecifier::Variable(parse_var_target(chars)?)),
			// TODO: add source snippet
			_ => r_panic!("Invalid location specifier."),
		}
	}

	fn to_positive_cell_offset(&self) -> Result<usize, String> {
		r_assert!(
			self.1 == 0 && self.0 >= 0,
			"Expected non-negative 1st dimensional cell offset (i.e. (x,y) where y=0)."
		);
		Ok(self.0 as usize)
	}
}

fn parse_var_type_definition<TC: TapeCellLocation>(
	chars: &mut &[char],
) -> Result<VariableTypeDefinition<TC>, String> {
	let mut var_type = match next_token(chars)? {
		Token::Cell => VariableTypeReference::Cell,
		Token::Struct => {
			let Token::Name(struct_name) = next_token(chars)? else {
				// TODO: add source snippet
				r_panic!("Expected struct type name in variable definition.");
			};

			VariableTypeReference::Struct(struct_name)
		}
		_ => {
			// TODO: add source snippet
			r_panic!("Unexpected token in variable type definition.");
		}
	};

	// parse array specifiers
	{
		let mut s = *chars;
		while let Token::LeftSquareBracket = next_token(&mut s)? {
			var_type = VariableTypeReference::Array(Box::new(var_type), parse_subscript(chars)?);
			s = chars;
		}
	}

	let Token::Name(name) = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected name in variable definition.");
	};

	Ok(VariableTypeDefinition {
		var_type,
		name,
		location_specifier: TC::parse_location_specifier(chars)?,
	})
}

/// parse the subscript of an array variable, e.g. [4] [6] [0]
/// must be compile-time constant
fn parse_subscript(chars: &mut &[char]) -> Result<usize, String> {
	let Token::LeftSquareBracket = next_token(chars)? else {
		// TODO: add program snippet
		r_panic!("Expected `[` in array subscript.");
	};
	let Token::Number(number) = next_token(chars)? else {
		// TODO: add program snippet
		r_panic!("Expected natural number in array subscript.");
	};
	let Token::RightSquareBracket = next_token(chars)? else {
		// TODO: add program snippet
		r_panic!("Expected `]` in array subscript.");
	};
	// TODO: handle errors here
	Ok(number)
}

pub fn parse_var_target(chars: &mut &[char]) -> Result<VariableTarget, String> {
	let is_spread = {
		let mut s = *chars;
		if let Token::Asterisk = next_token(&mut s)? {
			*chars = s;
			true
		} else {
			false
		}
	};

	let Token::Name(base_var_name) = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected identifier in variable target identifier.");
	};

	let mut ref_chain = vec![];
	let mut s = *chars;
	loop {
		match next_token(&mut s)? {
			Token::LeftSquareBracket => {
				let index = parse_subscript(chars)?;
				ref_chain.push(Reference::Index(index));
			}
			Token::Dot => {
				let Token::Name(subfield_name) = next_token(&mut s)? else {
					// TODO: add source snippet
					r_panic!("Expected subfield name in variable target identifier.");
				};
				ref_chain.push(Reference::NamedField(subfield_name));
			}
			_ => break,
		}
		*chars = s;
	}

	Ok(VariableTarget {
		name: base_var_name,
		subfields: if ref_chain.len() > 0 {
			Some(VariableTargetReferenceChain(ref_chain))
		} else {
			None
		},
		is_spread,
	})
}

fn parse_integer(chars: &mut &[char]) -> Result<i32, String> {
	let mut token = next_token(chars)?;
	let mut is_negative = false;
	if let Token::Minus = token {
		is_negative = true;
		token = next_token(chars)?;
	}
	let Token::Number(magnitude) = token else {
		// TODO: add source snippet
		r_panic!("Expected integer.")
	};
	// TODO: handle errors here
	Ok(match is_negative {
		// TODO: truncation error handling
		false => magnitude as i32,
		true => -(magnitude as i32),
	})
}

fn parse_integer_tuple<const LENGTH: usize>(chars: &mut &[char]) -> Result<[i32; LENGTH], String> {
	let Token::LeftParenthesis = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected opening parenthesis in tuple.")
	};

	let mut tuple = [0; LENGTH];
	for (j, element) in tuple.iter_mut().enumerate() {
		*element = parse_integer(chars)?;

		if j < LENGTH - 1 {
			let Token::Comma = next_token(chars)? else {
				// TODO: add source snippet
				r_panic!("Expected comma in tuple.");
			};
		}
	}
	let Token::RightParenthesis = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected closing parenthesis in tuple.");
	};

	Ok(tuple)
}

////////////////////////////
////////////////////////////
////////////////////////////

fn parse_if_else_clause<TC: TapeCellVariant, OC: OpcodeVariant>(
	chars: &mut &[char],
) -> Result<Clause<TC, OC>, String> {
	let Token::If = next_token(chars)? else {
		// TODO: add program snippet
		r_panic!("Expected \"if\" in if-else clause.");
	};

	let is_not = {
		let mut s = *chars;
		if let Token::Not = next_token(&mut s)? {
			*chars = s;
			true
		} else {
			false
		}
	};
	let condition = Expression::parse(chars)?;
	{
		let mut s = *chars;
		let Token::LeftBrace = next_token(&mut s)? else {
			r_panic!("Expected code block in if-else clause.");
		};
	}
	let block_one = parse_block(chars)?;

	let block_two = {
		let mut s = *chars;
		if let Token::Else = next_token(&mut s)? {
			*chars = s;
			Some(parse_block(chars)?)
		} else {
			None
		}
	};

	Ok(match (is_not, block_one, block_two) {
		(false, if_block, None) => Clause::If {
			condition,
			if_block,
		},
		(true, if_not_block, None) => Clause::IfNot {
			condition,
			if_not_block,
		},
		(false, if_block, Some(else_block)) => Clause::IfElse {
			condition,
			if_block,
			else_block,
		},
		(true, if_not_block, Some(else_block)) => Clause::IfNotElse {
			condition,
			if_not_block,
			else_block,
		},
	})
}

fn parse_while_clause<TC: TapeCellVariant, OC: OpcodeVariant>(
	chars: &mut &[char],
) -> Result<Clause<TC, OC>, String> {
	let Token::While = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected `while` in while clause.");
	};

	let condition = Expression::parse(chars)?;
	// TODO: make while loops support expressions
	let Expression::VariableReference(condition_variable) = condition else {
		r_panic!("While clause expected variable target condition.");
	};

	{
		let mut s = *chars;
		let Token::LeftBrace = next_token(&mut s)? else {
			r_panic!("Expected code block in while clause.");
		};
	}
	let loop_block = parse_block(chars)?;

	Ok(Clause::While {
		var: condition_variable,
		block: loop_block,
	})
}

fn parse_function_definition_clause<TC: TapeCellVariant, OC: OpcodeVariant>(
	chars: &mut &[char],
) -> Result<Clause<TC, OC>, String> {
	let Token::Fn = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected `fn` in function definition clause.");
	};

	let Token::Name(function_name) = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected name in function definition clause.");
	};

	let Token::LeftParenthesis = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected argument list in function definition clause.");
	};
	let mut arguments = vec![];
	loop {
		{
			let mut s = *chars;
			if let Token::RightParenthesis = next_token(&mut s)? {
				*chars = s;
				break;
			}
		}
		arguments.push(parse_var_type_definition(chars)?);

		match next_token(chars)? {
			Token::RightParenthesis => break,
			Token::Comma => (),
			// TODO: add source snippet
			_ => r_panic!("Unexpected token in function argument list."),
		}
	}

	Ok(Clause::DefineFunction {
		name: function_name,
		arguments,
		block: parse_block(chars)?,
	})
}

/// Parse tokens representing a struct definition into a clause
fn parse_struct_definition_clause<TC: TapeCellLocation, O>(
	chars: &mut &[char],
) -> Result<Clause<TC, O>, String> {
	let Token::Struct = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected `struct` in struct definition.");
	};

	let Token::Name(name) = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected name in struct definition.");
	};

	let Token::LeftBrace = next_token(chars)? else {
		// TODO: add source snippet
		r_panic!("Expected `{{` in struct clause.");
	};

	let mut fields = vec![];
	loop {
		let field = parse_var_type_definition::<TC>(chars)?;
		fields.push(field.try_into()?);
		let Token::Semicolon = next_token(chars)? else {
			// TODO: add source snippet
			r_panic!("Expected semicolon after struct definition field.");
		};
		if let Token::RightBrace = next_token(chars)? {
			break;
		}
	}

	Ok(Clause::DefineStruct { name, fields })
}

/// parse variable declarations and definitions.
/// e.g. `cell x = 0;` or `struct DummyStruct y;`
fn parse_let_clause<TC: TapeCellLocation, O>(chars: &mut &[char]) -> Result<Clause<TC, O>, String> {
	let var = parse_var_type_definition(chars)?;

	let mut s = *chars;
	if let Token::EqualsSign = next_token(&mut s)? {
		*chars = s;
		let expr = Expression::parse(chars)?;
		let Token::Semicolon = next_token(chars)? else {
			r_panic!("Expected semicolon after variable definition.");
		};
		return Ok(Clause::DefineVariable { var, value: expr });
	}
	let Token::Semicolon = next_token(chars)? else {
		r_panic!("Expected semicolon after variable declaration.");
	};
	Ok(Clause::DeclareVariable { var })
}
