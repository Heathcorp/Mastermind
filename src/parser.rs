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
	let mut clauses: Vec<Clause> = Vec::new();
	let mut i = 0usize;
	while let Some(clause_tokens) = get_clause_tokens(&tokens[i..]) {
		clauses.push(
			match (
				&clause_tokens[0],
				&clause_tokens[1],
				if tokens.len() > 2 {
					&clause_tokens[2]
				} else {
					&Token::None
				},
			) {
				(Token::Let, _, _) => parse_let_clause(clause_tokens),
				(Token::Drain, _, _) => parse_drain_copy_clause(clause_tokens, true),
				(Token::Copy, _, _) => parse_drain_copy_clause(clause_tokens, false),
				(Token::Output, _, _) => parse_output_clause(clause_tokens),
				(Token::Name(_), Token::OpenParenthesis, _) => {
					parse_function_call_clause(clause_tokens)
				}
				(Token::Def, _, _) => parse_function_definition_clause(clause_tokens),
				(Token::Name(_), Token::Plus | Token::Minus, Token::Equals) => {
					parse_add_clause(clause_tokens)
				}

				_ => {
					panic!("Clause type not implemented: {clause_tokens:#?}");
				}
			},
		);
		i += clause_tokens.len();
	}

	clauses
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
		let subscript = get_braced_tokens(&clause[i..], SQUARE_BRACKETS);
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
			initial_value: Some(expr),
		}
	} else if let Token::ClauseDelimiter = &clause[i] {
		Clause::DefineVariable {
			name: name.clone(),
			arr_len,
			initial_value: None,
		}
	} else {
		panic!("Invalid token at end of let clause: {clause:#?}");
	}
}

fn parse_add_clause(clause: &[Token]) -> Clause {
	let mut i = 0usize;
	let Token::Name(name) = &clause[i] else {
		panic!("Expected name at start of add clause: {clause:#?}");
	};
	i += 1;
	let positive = match &clause[i] {
		Token::Plus => true,
		Token::Minus => false,
		_ => {
			panic!("Unexpected second token in add clause: {clause:#?}");
		}
	};
	let mut expr = parse_expression(&clause[i..(clause.len() - 1)]);
	match (&mut expr, positive) {
		(Expression::Constant(n), false) => {
			*n = -*n;
		}
		_ => (),
	}
	Clause::AddToVariable {
		var_name: name.clone(),
		value: expr,
	}
}

fn parse_drain_copy_clause(clause: &[Token], is_draining: bool) -> Clause {
	// drain g {i += 1;};
	// drain g into j;
	// copy foo into bar {g += 2; etc;};

	let source_name: String;
	let mut target_names = Vec::new();
	let mut block = Vec::new();
	let mut i = 1;
	if let Token::Name(identifier) = &clause[i] {
		source_name = identifier.clone();
		i += 1;
	} else {
		panic!("Invalid identifier name in second position of drain clause: {clause:#?}");
	}

	if let Token::Into = &clause[i] {
		// simple drain/copy move operations
		i += 1;

		loop {
			match &clause[i] {
				Token::Name(identifier) => {
					target_names.push(identifier.clone());
					i += 1;
				}
				Token::OpenBrace | Token::ClauseDelimiter => {
					break;
				}
				_ => {
					panic!("Unexpected token in drain clause: {clause:#?}");
				}
			}
		}
	}

	if let Token::OpenBrace = &clause[i] {
		// code block to execute at each loop iteration
		let braced_tokens = get_braced_tokens(&clause[i..], BRACES);
		// recursion
		block.extend(parse(braced_tokens));
		i += 2 + braced_tokens.len();
	}

	if let Token::ClauseDelimiter = &clause[i] {
		Clause::CopyLoop {
			source_name,
			target_names,
			block,
			is_draining,
		}
	} else {
		panic!("Invalid token at end of copy/drain clause: {clause:#?}");
	}
}

fn parse_output_clause(clause: &[Token]) -> Clause {
	let mut i = 1usize;
	let var_name: String;
	if let Token::Name(identifier) = &clause[i] {
		// TODO: implement expressions here
		var_name = identifier.clone();
		i += 1;
	} else {
		panic!("Unexpected token in output clause: {clause:#?}");
	}

	if let Token::OpenSquareBracket = &clause[i] {
		let subscript = get_braced_tokens(&clause[i..], SQUARE_BRACKETS);
		i += 2 + subscript.len();
		let Expression::Constant(i) = parse_expression(subscript);
		Clause::OutputByte {
			var_name,
			arr_index: Some(i.try_into().unwrap()),
		}
	} else if let Token::ClauseDelimiter = &clause[i] {
		Clause::OutputByte {
			var_name,
			arr_index: None,
		}
	} else {
		panic!("Invalid token at end of output clause: {clause:#?}");
	}
}
fn parse_function_definition_clause(clause: &[Token]) -> Clause {
	let mut i = 1usize;
	// function name
	let Token::Name(name) = &clause[i] else {
		panic!("Expected function name after in function definition clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;
	let Token::OpenParenthesis = &clause[i] else {
		panic!("Expected open parenthesis in function definition clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], PARENTHESES);
	let mut j = 0usize;
	// parse function argument names
	while let Token::Name(arg_name) = &arg_tokens[j] {
		let mut arr_len: Option<usize> = None;
		j += 1;

		if j >= arg_tokens.len() {
			args.push((arg_name.clone(), arr_len));
			break;
		} else if let Token::OpenSquareBracket = &arg_tokens[j] {
			// the argument is an array/multi-byte value
			let subscript = get_braced_tokens(&arg_tokens[j..], SQUARE_BRACKETS);
			let Expression::Constant(num_len) = parse_expression(subscript) else {
				panic!("Expected a constant array length specifier in argument definition: {arg_tokens:#?}");
			};
			arr_len = Some(num_len.try_into().unwrap());
			j += 2 + subscript.len();
		}

		args.push((arg_name.clone(), arr_len));

		if j >= arg_tokens.len() {
			break;
		} else if let Token::Comma = &arg_tokens[j] {
			j += 1;
		} else {
			panic!("Unexpected token in function definition arguments: {arg_tokens:#?}");
		}
	}

	i += 2 + arg_tokens.len();

	// recursively parse the inner block
	let Token::OpenBrace = &clause[i] else {
		panic!("Expected execution block in function definition: {clause:#?}");
	};

	let block_tokens = get_braced_tokens(&clause[i..], BRACES);
	let parsed_block = parse(block_tokens);

	Clause::DefineFunction {
		name: name.clone(),
		arguments: args,
		block: parsed_block,
	}
}

fn parse_function_call_clause(clause: &[Token]) -> Clause {
	let mut i = 0usize;
	// Okay I didn't know this rust syntax, could have used it all over the place
	let Token::Name(name) = &clause[i] else {
		panic!("Expected function identifier at start of function call clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;

	let Token::OpenParenthesis = &clause[i] else {
		panic!("Expected open parenthesis in function call clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], PARENTHESES);

	let mut j = 0usize;
	while let Token::Name(arg_name) = &arg_tokens[j] {
		args.push(arg_name.clone());
		j += 1;
		if j >= arg_tokens.len() {
			break;
		} else if let Token::Comma = &arg_tokens[j] {
			j += 1;
		} else {
			panic!("Unexpected token in function call arguments: {arg_tokens:#?}");
		}
	}

	i += 2 + arg_tokens.len();

	let Token::ClauseDelimiter = &clause[i] else {
		panic!("Expected clause delimiter at end of function call clause: {clause:#?}");
	};

	Clause::CallFunction {
		function_name: name.clone(),
		argument_names: args,
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
					let braced_block = get_braced_tokens(&tokens[i..], BRACES);
					i += 2 + braced_block.len();
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

const SQUARE_BRACKETS: (Token, Token) = (Token::OpenSquareBracket, Token::ClosingSquareBracket);
const BRACES: (Token, Token) = (Token::OpenBrace, Token::ClosingBrace);
const PARENTHESES: (Token, Token) = (Token::OpenParenthesis, Token::ClosingParenthesis);
// this should be a generic function but rust doesn't support enum variants as type arguments yet
// find tokens bounded by matching brackets
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

	if len >= 2 {
		if _braces.0 == mem::discriminant(&tokens[0])
			&& _braces.1 == mem::discriminant(&tokens[len - 1])
		{
			return &tokens[1..(len - 1)];
		}
	}
	panic!("Invalid braced tokens: {tokens:#?}");
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
	DefineVariable {
		name: String,
		arr_len: Option<usize>,
		initial_value: Option<Expression>,
	},
	AddToVariable {
		var_name: String,
		value: Expression,
	},
	// TODO
	SetVariable {},
	CopyLoop {
		source_name: String,
		target_names: Vec<String>,
		block: Vec<Clause>,
		is_draining: bool,
	},
	OutputByte {
		var_name: String,
		arr_index: Option<usize>,
	},
	DefineFunction {
		name: String,
		arguments: Vec<(String, Option<usize>)>,
		block: Vec<Clause>,
	},
	CallFunction {
		function_name: String,
		argument_names: Vec<String>,
	},
}
