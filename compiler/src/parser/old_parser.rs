// project dependencies:
use crate::{
	backend::{bf::TapeCell, bf2d::TapeCell2D, common::OpcodeVariant},
	macros::macros::{r_assert, r_panic},
	tokeniser::Token,
};

// stdlib dependencies
use std::{fmt::Display, mem::discriminant, num::Wrapping};

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
