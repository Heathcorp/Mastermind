use std::{
	collections::HashMap,
	mem::{self, discriminant, Discriminant},
};

use crate::tokeniser::Token;

// recursive function to create a tree representation of the program
pub fn parse(tokens: &[Token]) -> Vec<Clause> {
	// basic steps:
	// chew off tokens from the front, recursively parse blocks of tokens
	// TODO: down the track with expressions we will need another function to parse those
	let mut commands: Vec<Clause> = Vec::new();
	let mut i = 0usize;
	while let Some(clause) = get_clause_tokens(&tokens[i..]) {
		commands.push(match (&clause[0], &clause[1]) {
			(Token::Let, Token::Name(_)) => parse_let_clause(clause),
			_ => {
				panic!("Clause type not implemented: {clause:#?}");
			}
		});
		i += clause.len();
	}

	commands
}

fn parse_let_clause(clause: &[Token]) -> Clause {
	// let foo = 9;
	// let arr[2] = ??;
	// let g;
	// let why[9];
	let name: &String;
	let mut i = 1;
	if let Token::Name(identifier) = &clause[i] {
		name = identifier;
		i += 1;
	} else {
		panic!("Invalid identifier name in second position of let clause: {clause:#?}");
	}
	let mut arr_len: Option<usize> = None;
	if let Token::OpenSquareBracket = clause[i] {
		let subscript = get_braced_tokens(
			&clause[i..],
			(Token::OpenSquareBracket, Token::ClosingSquareBracket),
		);
		let len_expr = parse_expression(subscript);
		if let Expression::Constant(constant) = len_expr {
			arr_len = Some(constant.try_into().unwrap());
		} else {
			panic!("Invalid variable array length expression: {len_expr:#?}");
		}
		i += 2 + subscript.len(); // should be 3
	}

	if let Token::Equals = &clause[i] {
		i += 1;
		let remaining = &clause[i..(clause.len() - 1)];
		let expr = parse_expression(remaining);
		Clause::DefineVariable {
			name: name.clone(),
			arr_len,
			value: expr,
		}
	} else if let Token::ClauseDelimiter = &clause[i] {
		Clause::DeclareVariable {
			name: name.clone(),
			arr_len,
		}
	} else {
		panic!("Invalid token at end of let clause: {clause:#?}");
	}
}

// get a clause, typically a line, bounded by ;
fn get_clause_tokens(tokens: &[Token]) -> Option<&[Token]> {
	if tokens.len() == 0 {
		None
	} else {
		let mut i = 0usize;
		while i < tokens.len() {
			match tokens[i] {
				Token::OpenBrace => {
					i += get_block(&tokens[i..]).unwrap().len();
				}
				Token::ClauseDelimiter => {
					i += 1;
					return Some(&tokens[..i]);
				}
				_ => {
					i += 1;
				}
			}
		}

		panic!("Found no clause delimiter at end of input");
	}
}

// given a list of tokens starting with {, find the range bounded by the corresponding closing }
fn get_block(tokens: &[Token]) -> Option<&[Token]> {
	if tokens.len() == 0 {
		None
	} else if let Token::OpenBrace = tokens[0] {
		let mut i = 1usize;
		let mut depth = 0;
		while depth != 0 && i < tokens.len() {
			depth += match tokens[i] {
				Token::OpenBrace => 1,
				Token::ClosingBrace => -1,
				_ => 0,
			};

			i += 1;
		}

		Some(&tokens[..i])
	} else {
		None
	}
}

// find matching square brackets
fn get_braced_tokens(tokens: &[Token], braces: (Token, Token)) -> &[Token] {
	let _braces = (mem::discriminant(&braces.0), mem::discriminant(&braces.1));
	// find corresponding bracket, the depth check is unnecessary but whatever
	let len = {
		let mut i = 1usize;
		let mut depth = 1;
		while i < tokens.len() && depth > 0 {
			let g = mem::discriminant(&tokens[i]);
			if g == _braces.0 {
				depth += 1;
			} else if g == _braces.1 {
				depth -= 1;
			}
			i += 1;
		}
		i
	};

	if len >= 3 {
		if _braces.0 == mem::discriminant(&tokens[0])
			&& _braces.1 == mem::discriminant(&tokens[len - 1])
		{
			return &tokens[1..(len - 1)];
		}
	}
	panic!("Invalid subscript tokens: {tokens:#?}");
}

fn parse_expression(tokens: &[Token]) -> Expression {
	// currently only signed constants are supported
	let mut t = tokens.iter();

	let mut token = t.next().unwrap();
	let expr = match token {
		Token::Minus | Token::Number(_) => {
			// number
			let mut negative = false;
			if let Token::Minus = token {
				negative = true;
				token = t.next().unwrap();
			}
			if let Token::Number(num_literal) = token {
				let mut number: i32 = num_literal.parse().unwrap();
				if negative {
					number = -number;
				}
				Expression::Constant(number)
			} else {
				panic!("Unexpected token found in expression: {tokens:#?}");
			}
		}
		// Token::OpenSquareBracket => {
		// 	// array
		// }
		_ => {
			panic!("Unexpected token found in expression: {tokens:#?}");
		}
	};

	if t.next().is_some() {
		panic!("Unexpected token found after expression: {tokens:#?}");
	}

	expr
}

// TODO: do we need crazy recursive expressions?
#[derive(Debug)]
pub enum Expression {
	Constant(i32),
	// + means true sign, - means false sign
	// Variable{sign: bool, name: String},
	// ContstantsArray(Vec<i32>),
}

#[derive(Debug)]
pub enum Clause {
	DeclareVariable {
		name: String,
		arr_len: Option<usize>,
	},
	DefineVariable {
		name: String,
		arr_len: Option<usize>,
		value: Expression,
	},
}
