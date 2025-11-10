// project dependencies:
use crate::{
	backend::{bf::TapeCell, bf2d::TapeCell2D, common::OpcodeVariant},
	macros::macros::{r_assert, r_panic},
	tokeniser::Token,
};

// stdlib dependencies
use std::{fmt::Display, mem::discriminant, num::Wrapping};

/// recursive function to create a tree representation of the program
pub fn parse_clause_from_tokens<TC: TapeCellLocation, OC: OpcodeVariant>(
	tokens: &[Token],
	blocks: Vec<Vec<Clause<TC, OC>>>,
) -> Result<Option<Clause<TC, OC>>, String> {
	Ok(match (&tokens[0], &tokens.get(1), &tokens.get(2)) {
		(Token::Cell, _, _)
		| (Token::Struct, Some(Token::Name(_)), Some(Token::Name(_) | Token::OpenSquareBracket)) => {
			Some(parse_let_clause(tokens)?)
		}
		(Token::Struct, Some(Token::Name(_)), Some(Token::OpenBrace)) => {
			Some(parse_struct_clause(tokens)?)
		}
		(Token::Plus, Some(Token::Plus), _) | (Token::Minus, Some(Token::Minus), _) => {
			Some(parse_increment_clause(tokens)?)
		}
		(Token::Name(_), Some(Token::EqualsSign | Token::Dot | Token::OpenSquareBracket), _) => {
			Some(parse_set_clause(clause_tokens)?)
		}
		(Token::Drain, _, _) => Some(parse_drain_copy_clause(
			tokens,
			true,
			blocks
				.get(0)
				.ok_or(format!("Expected code block in drain clause: {tokens:#?}"))?,
		)?),
		(Token::Copy, _, _) => {
			clauses.push(parse_drain_copy_clause(clause_tokens, false)?);
		}
		(Token::While, _, _) => {
			clauses.push(parse_while_clause(clause_tokens)?);
		}
		(Token::Output, _, _) => {
			clauses.push(parse_output_clause(clause_tokens)?);
		}
		(Token::Input, _, _) => {
			clauses.push(parse_input_clause(clause_tokens)?);
		}
		(Token::Name(_), Some(Token::OpenParenthesis), _) => {
			clauses.push(parse_function_call_clause(clause_tokens)?);
		}
		(Token::Fn, _, _) => {
			clauses.push(parse_function_definition_clause(clause_tokens)?);
		}
		(Token::Name(_), Token::Plus | Token::Minus, Token::EqualsSign) => {
			clauses.extend(parse_add_clause(clause_tokens)?);
		}
		(Token::If, _, _) => {
			clauses.push(parse_if_else_clause(clause_tokens)?);
		}
		(Token::OpenBrace, _, _) => {
			let braced_tokens = get_braced_tokens(clause_tokens, BRACES)?;
			let inner_clauses = parse(braced_tokens)?;
			clauses.push(Clause::Block(inner_clauses));
		}
		(Token::Assert, _, _) => Some(parse_assert_clause(tokens)?),
		// empty clause
		(Token::Semicolon, _, _) => None,
		// the None token usually represents whitespace, it should be filtered out before reaching this function
		// Wrote out all of these possibilities so that the compiler will tell me when I haven't implemented a token
		(
			Token::Else
			| Token::Not
			| Token::ClosingBrace
			| Token::OpenSquareBracket
			| Token::ClosingSquareBracket
			| Token::OpenParenthesis
			| Token::ClosingParenthesis
			| Token::Comma
			| Token::Plus
			| Token::Minus
			| Token::Into
			| Token::Digits(_)
			| Token::Name(_)
			| Token::String(_)
			| Token::Character(_)
			| Token::True
			| Token::False
			| Token::EqualsSign
			| Token::Asterisk
			| Token::Clobbers
			| Token::Equals
			| Token::Unknown
			| Token::Dot
			| Token::At
			| Token::Struct,
			_,
			_,
		) => r_panic!("Invalid clause: {tokens:#?}"),
	})
}

// currently just syntax sugar, should make it actually do post/pre increments
fn parse_increment_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let (var, _) = parse_var_target(&clause[2..])?;
	//An increment clause can never be self referencing since it just VAR++
	Ok(match (&clause[0], &clause[1]) {
		(Token::Plus, Token::Plus) => Clause::AddToVariable {
			var,
			value: Expression::NaturalNumber(1),
			self_referencing: false,
		},
		(Token::Minus, Token::Minus) => Clause::AddToVariable {
			var,
			value: Expression::NaturalNumber((-1i8 as u8) as usize),
			self_referencing: false,
		},
		_ => {
			r_panic!("Invalid pattern in increment clause: {clause:#?}");
		}
	})
	// assumed that the final token is a semicolon
}

fn parse_set_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let mut i = 0usize;
	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	Ok(match &clause[i] {
		Token::EqualsSign => {
			i += 1;
			let expr = Expression::parse(&clause[i..(clause.len() - 1)])?;
			let self_referencing = expr.check_self_referencing(&var);
			Clause::SetVariable {
				var,
				value: expr,
				self_referencing,
			}
		}
		Token::Plus | Token::Minus => {
			let is_add = if let Token::Plus = &clause[i] {
				true
			} else {
				false
			};
			i += 1;
			let Token::EqualsSign = &clause[i] else {
				r_panic!("Expected equals sign in add-assign operator: {clause:#?}");
			};
			i += 1;

			let mut expr = Expression::parse(&clause[i..(clause.len() - 1)])?;
			if !is_add {
				expr = expr.flipped_sign()?;
			}

			let self_referencing = expr.check_self_referencing(&var);
			Clause::AddToVariable {
				var,
				value: expr,
				self_referencing,
			}
		}
		_ => r_panic!("Expected assignment operator in set clause: {clause:#?}"),
	})
}

fn parse_if_else_clause<TC: TapeCellLocation, OC: OpcodeVariant>(
	clause: &[Token],
) -> Result<Clause<TC, OC>, String> {
	// skip first token, assumed to start with if
	let mut i = 1usize;
	let mut not = false;
	if let Token::Not = &clause[i] {
		not = true;
		i += 1;
	}

	let condition_start_token = i;

	i += 1;
	while let Some(token) = clause.get(i) {
		if let Token::OpenBrace = token {
			break;
		}
		i += 1;
	}
	r_assert!(
		i < clause.len(),
		"Expected condition and block in if statement: {clause:#?}"
	);

	let condition = Expression::parse(&clause[condition_start_token..i])?;

	let block_one = {
		let block_tokens = get_braced_tokens(&clause[i..], BRACES)?;
		i += 2 + block_tokens.len();
		parse(block_tokens)?
	};

	let block_two = if let Some(Token::Else) = &clause.get(i) {
		i += 1;
		let block_tokens = get_braced_tokens(&clause[i..], BRACES)?;
		// i += 2 + block_tokens.len();
		Some(parse(block_tokens)?)
	} else {
		None
	};

	Ok(match (not, block_one, block_two) {
		(false, block_one, block_two) => Clause::IfElse {
			condition,
			if_block: Some(block_one),
			else_block: block_two,
		},
		(true, block_one, block_two) => Clause::IfElse {
			condition,
			if_block: block_two,
			else_block: Some(block_one),
		},
	})
}

fn parse_output_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let mut i = 1usize;

	let expr_tokens = &clause[i..(clause.len() - 1)];
	let expr = Expression::parse(expr_tokens)?;
	i += expr_tokens.len();

	let Token::Semicolon = &clause[i] else {
		r_panic!("Invalid token at end of output clause: {clause:#?}");
	};

	Ok(Clause::OutputValue { value: expr })
}

fn parse_input_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let mut i = 1usize;

	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	let Token::Semicolon = &clause[i] else {
		r_panic!("Invalid token at end of input clause: {clause:#?}");
	};

	Ok(Clause::InputVariable { var })
}

fn parse_assert_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let mut i = 1usize;

	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	if let Token::Unknown = &clause[i] {
		Ok(Clause::AssertVariableValue { var, value: None })
	} else {
		let Token::Equals = &clause[i] else {
			r_panic!("Expected assertion value in assert clause: {clause:#?}");
		};
		i += 1;

		let Token::Semicolon = &clause[clause.len() - 1] else {
			r_panic!("Invalid token at end of assert clause: {clause:#?}");
		};

		let remaining = &clause[i..(clause.len() - 1)];
		let expr = Expression::parse(remaining)?;

		Ok(Clause::AssertVariableValue {
			var,
			value: Some(expr),
		})
	}
}

fn parse_brainfuck_clause<TC: TapeCellLocation, OC: OpcodeVariant>(
	clause: &[Token],
) -> Result<Clause<TC, OC>, String> {
	// bf {++--<><}
	// bf @3 {++--<><}
	// bf clobbers var1 var2 {++--<><}
	// bf @2 clobbers *arr {++--<><}

	let mut clobbers = Vec::new();
	let mut i = 1usize;

	// check for location specifier
	let (mem_offset, len) = TC::parse_location_specifier(&clause[i..])?;
	i += len;

	if let Token::Clobbers = &clause[i] {
		i += 1;

		loop {
			match &clause[i] {
				Token::Name(_) | Token::Asterisk => {
					let (var, len) = parse_var_target(&clause[i..])?;
					i += len;
					clobbers.push(var);
				}
				Token::OpenBrace => {
					break;
				}
				_ => {
					r_panic!("Unexpected token in drain clause: {clause:#?}");
				}
			}
		}
	}

	let bf_tokens = get_braced_tokens(&clause[i..], BRACES)?;
	let mut ops = Vec::new();
	let mut j = 0;
	while j < bf_tokens.len() {
		match &bf_tokens[j] {
			Token::OpenBrace => {
				// embedded mastermind
				let block_tokens = get_braced_tokens(&bf_tokens[j..], BRACES)?;
				let clauses = parse(block_tokens)?;
				ops.push(ExtendedOpcode::Block(clauses));
				j += block_tokens.len() + 2;
			}
			token @ _ => {
				ops.push(ExtendedOpcode::Opcode(OC::from_token(token)?));
				j += 1;
			}
		}
	}

	Ok(Clause::InlineBrainfuck {
		location_specifier: mem_offset,
		clobbered_variables: clobbers,
		operations: ops,
	})
}

fn parse_function_definition_clause<TC: TapeCellLocation, OC: OpcodeVariant>(
	clause: &[Token],
) -> Result<Clause<TC, OC>, String> {
	let mut i = 1usize;
	// function name
	let Token::Name(name) = &clause[i] else {
		r_panic!("Expected function name after in function definition clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;
	let Token::OpenParenthesis = &clause[i] else {
		r_panic!("Expected argument list in function definition clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], PARENTHESES)?;
	let mut j = 0usize;
	// parse function argument names
	while j < arg_tokens.len() {
		// break if no more arguments
		let (Token::Cell | Token::Struct) = &arg_tokens[j] else {
			break;
		};
		let (var, len) = parse_var_definition(&arg_tokens[j..], false)?;
		j += len;

		args.push(var);

		if j >= arg_tokens.len() {
			break;
		} else if let Token::Comma = &arg_tokens[j] {
			j += 1;
		} else {
			r_panic!("Unexpected token in function definition arguments: {arg_tokens:#?}");
		}
	}

	i += 2 + arg_tokens.len();

	// recursively parse the inner block
	let Token::OpenBrace = &clause[i] else {
		r_panic!("Expected execution block in function definition: {clause:#?}");
	};

	let block_tokens = get_braced_tokens(&clause[i..], BRACES)?;
	let parsed_block = parse(block_tokens)?;

	Ok(Clause::DefineFunction {
		name: name.clone(),
		arguments: args,
		block: parsed_block,
	})
}

fn parse_function_call_clause<T, O>(clause: &[Token]) -> Result<Clause<T, O>, String> {
	let mut i = 0usize;
	// Okay I didn't know this rust syntax, could have used it all over the place
	let Token::Name(name) = &clause[i] else {
		r_panic!("Expected function identifier at start of function call clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;

	let Token::OpenParenthesis = &clause[i] else {
		r_panic!("Expected argument list in function call clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], PARENTHESES)?;

	let mut j = 0usize;
	while j < arg_tokens.len() {
		// this used to be in the while condition but moved it here to check for the case of no arguments
		let Token::Name(_) = &arg_tokens[j] else {
			break;
		};
		let (var, len) = parse_var_target(&arg_tokens[j..])?;
		j += len;

		args.push(var);

		if j >= arg_tokens.len() {
			break;
		} else if let Token::Comma = &arg_tokens[j] {
			j += 1;
		} else {
			r_panic!("Unexpected token in function call arguments: {arg_tokens:#?}");
		}
	}

	i += 2 + arg_tokens.len();

	let Token::Semicolon = &clause[i] else {
		r_panic!("Expected clause delimiter at end of function call clause: {clause:#?}");
	};

	Ok(Clause::CallFunction {
		function_name: name.clone(),
		arguments: args,
	})
}
