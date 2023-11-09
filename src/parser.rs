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
	while let Some(clause) = get_clause_tokens(&tokens[i..]) {
		clauses.push(
			match (
				&clause[0],
				&clause[1],
				if tokens.len() > 2 {
					&clause[2]
				} else {
					&Token::None
				},
			) {
				(Token::Let, _, _) => parse_let_clause(clause),
				(Token::Drain, _, _) => parse_drain_copy_clause(clause, true),
				(Token::Copy, _, _) => parse_drain_copy_clause(clause, false),
				(Token::While, _, _) => parse_while_clause(clause),
				(Token::Output, _, _) => parse_output_clause(clause),
				(Token::Name(_), Token::OpenParenthesis, _) => parse_function_call_clause(clause),
				(Token::Def, _, _) => parse_function_definition_clause(clause),
				(Token::Name(_), Token::Plus | Token::Minus, Token::Equals) => {
					parse_add_clause(clause)
				}
				(Token::Plus, Token::Plus, _) | (Token::Minus, Token::Minus, _) => {
					parse_increment_clause(clause)
				}
				(Token::Name(_), Token::Equals, _) => parse_set_clause(clause),
				(Token::If, _, _) => parse_if_else_clause(clause),
				// the None token usually represents whitespace, it should be filtered out before reaching this function
				// Wrote out all of these possibilities so that the compiler will tell me when I haven't implemented a token
				(
					Token::None
					| Token::ClauseDelimiter
					| Token::Else
					| Token::Not
					| Token::OpenBrace
					| Token::ClosingBrace
					| Token::OpenSquareBracket
					| Token::ClosingSquareBracket
					| Token::OpenParenthesis
					| Token::ClosingParenthesis
					| Token::Comma
					| Token::Plus
					| Token::Minus
					| Token::Into
					| Token::Number(_)
					| Token::Name(_)
					| Token::Equals,
					_,
					_,
				) => {
					panic!("Invalid clause: {clause:#?}");
				}
			},
		);
		i += clause.len();
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
	// this kind of logic could probably be done with iterators, (TODO for future refactors)
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;

	if let Token::Equals = &clause[i] {
		i += 1;
		let remaining = &clause[i..(clause.len() - 1)];
		let expr = parse_expression(remaining);
		Clause::DefineVariable {
			var,
			initial_value: Some(expr),
		}
	} else if let Token::ClauseDelimiter = &clause[i] {
		Clause::DefineVariable {
			var,
			initial_value: None,
		}
	} else {
		panic!("Invalid token at end of let clause: {clause:#?}");
	}
}

fn parse_add_clause(clause: &[Token]) -> Clause {
	let mut i = 0usize;
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;

	let positive = match &clause[i] {
		Token::Plus => true,
		Token::Minus => false,
		_ => {
			panic!("Unexpected second token in add clause: {clause:#?}");
		}
	};
	i += 2; // assume the equals sign is there because it was checked by the main loop
	let mut expr = parse_expression(&clause[i..(clause.len() - 1)]);
	match (&mut expr, positive) {
		(Expression::Constant(n), false) => {
			*n = -*n;
		}
		_ => (),
	}
	Clause::AddToVariable { var, value: expr }
}

fn parse_increment_clause(clause: &[Token]) -> Clause {
	let (var, len) = parse_var_details(&clause[2..]);

	match (&clause[0], &clause[1]) {
		(Token::Plus, Token::Plus) => Clause::AddToVariable {
			var,
			value: Expression::Constant(1),
		},
		(Token::Minus, Token::Minus) => Clause::AddToVariable {
			var,
			value: Expression::Constant(-1),
		},
		_ => {
			panic!("Invalid pattern in increment clause: {clause:#?}");
		}
	}
	// assumed that the final token is a semicolon
}

fn parse_set_clause(clause: &[Token]) -> Clause {
	let mut i = 0usize;
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;

	// definitely could use iterators instead (TODO for refactor)
	let Token::Equals = &clause[i] else {
		panic!("Expected equals sign in set clause: {clause:#?}");
	};
	i += 1;

	let expr = parse_expression(&clause[i..(clause.len() - 1)]);

	Clause::SetVariable { var, value: expr }
}

fn parse_drain_copy_clause(clause: &[Token], is_draining: bool) -> Clause {
	// drain g {i += 1;};
	// drain g into j;
	// copy foo into bar {g += 2; etc;};
	// TODO: make a tuple-parsing function and use it here instead of a space seperated list of targets

	let mut targets = Vec::new();
	let mut block = Vec::new();
	let mut i = 1usize;
	let (source, len) = parse_var_details(&clause[i..]);
	i += len;

	if let Token::Into = &clause[i] {
		// simple drain/copy move operations
		i += 1;

		loop {
			match &clause[i] {
				Token::Name(identifier) => {
					let (var, len) = parse_var_details(&clause[i..]);
					i += len;
					targets.push(var);
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
			source,
			targets,
			block,
			is_draining,
		}
	} else {
		panic!("Invalid token at end of copy/drain clause: {clause:#?}");
	}
}

fn parse_while_clause(clause: &[Token]) -> Clause {
	// TODO: make this able to accept expressions
	let mut i = 1usize;
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;
	// loop {
	// 	if let Token::OpenBrace = &clause[i] {
	// 		break;
	// 	};
	// 	i += 1;
	// }

	// let expr = parse_expression(&clause[1..i]);
	let block_tokens = get_braced_tokens(&clause[i..], BRACES);
	i += 2 + block_tokens.len();

	Clause::WhileLoop {
		var,
		block: parse(block_tokens),
	}
}

fn parse_if_else_clause(clause: &[Token]) -> Clause {
	// skip first token, assumed to start with if
	let mut i = 1usize;
	let mut not = false;
	if let Token::Not = &clause[i] {
		not = true;
		i += 1;
	}

	// TODO: use expressions here instead
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;

	let Token::OpenBrace = &clause[i] else {
		panic!("Expected block in if statement: {clause:#?}");
	};

	let block_one = {
		let block_tokens = get_braced_tokens(&clause[i..], BRACES);
		i += 2 + block_tokens.len();
		parse(block_tokens)
	};

	let block_two = if let Token::Else = &clause[i] {
		i += 1;
		let block_tokens = get_braced_tokens(&clause[i..], BRACES);
		i += 2 + block_tokens.len();
		Some(parse(block_tokens))
	} else {
		None
	};

	let empty = Vec::new();

	match (not, block_one, block_two) {
		(false, block_one, block_two) => Clause::IfStatement {
			var,
			if_block: block_one,
			else_block: block_two.unwrap_or(empty),
		},
		(true, block_one, block_two) => Clause::IfStatement {
			var,
			if_block: block_two.unwrap_or(empty),
			else_block: block_one,
		},
	}
}

fn parse_output_clause(clause: &[Token]) -> Clause {
	let mut i = 1usize;
	let (var, len) = parse_var_details(&clause[i..]);
	i += len;

	let Token::ClauseDelimiter = &clause[i] else {
		panic!("Invalid token at end of output clause: {clause:#?}");
	};

	Clause::OutputByte { var }
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
	while let Token::Name(_) = &arg_tokens[j] {
		let (var, len) = parse_var_details(&arg_tokens[j..]);
		j += len;

		args.push(var);

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
	while let Token::Name(_) = &arg_tokens[j] {
		let (var, len) = parse_var_details(&arg_tokens[j..]);
		j += len;

		args.push(var);

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
		arguments: args,
	}
}

fn parse_var_details(tokens: &[Token]) -> (VariableSpec, usize) {
	let mut i = 0usize;
	let Token::Name(var_name) = &tokens[i] else {
		panic!("Expected identifier at start of variable identifier: {tokens:#?}");
	};
	i += 1;

	let arr_num = if i < tokens.len() {
		if let Token::OpenSquareBracket = &tokens[i] {
			let subscript = get_braced_tokens(&tokens[i..], SQUARE_BRACKETS);
			let Expression::Constant(num_len) = parse_expression(subscript) else {
				panic!("Expected a constant array length specifier in variable identifier: {tokens:#?}");
			};
			i += 2 + subscript.len();
			Some(num_len.try_into().unwrap())
		} else {
			None
		}
	} else {
		None
	};

	(
		VariableSpec {
			name: var_name.clone(),
			arr_num,
		},
		i,
	)
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

// Okay so apparently I did try to use iterators here, but didn't continue that convention for the rest of the code
fn parse_expression(tokens: &[Token]) -> Expression {
	// TODO: make this more sophisticated
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
// TODO: yes, but no multiplication or division
#[derive(Debug)]
pub enum Expression {
	Constant(i32),
	// + means true sign, - means false sign
	// Variable{sign: bool, name: String},
	// ConstantsArray(Vec<i32>),
}

#[derive(Debug)]
pub enum Clause {
	DefineVariable {
		var: VariableSpec,
		initial_value: Option<Expression>,
	},
	AddToVariable {
		var: VariableSpec,
		value: Expression,
	},
	SetVariable {
		var: VariableSpec,
		value: Expression,
	},
	CopyLoop {
		source: VariableSpec,
		targets: Vec<VariableSpec>,
		block: Vec<Clause>,
		is_draining: bool,
	},
	WhileLoop {
		var: VariableSpec,
		block: Vec<Clause>,
	},
	OutputByte {
		var: VariableSpec,
	},
	DefineFunction {
		name: String,
		arguments: Vec<VariableSpec>,
		block: Vec<Clause>,
	},
	CallFunction {
		function_name: String,
		arguments: Vec<VariableSpec>,
	},
	IfStatement {
		var: VariableSpec,
		if_block: Vec<Clause>,
		else_block: Vec<Clause>,
	},
}

#[derive(Debug)]
pub struct VariableSpec {
	name: String,
	arr_num: Option<usize>,
}
