use std::{
	fmt::{Display, Write},
	mem::discriminant,
	num::Wrapping,
};

use crate::{
	builder::TapeCell,
	macros::macros::{r_assert, r_panic},
	tokeniser::Token,
};

// recursive function to create a tree representation of the program
pub fn parse(tokens: &[Token]) -> Result<Vec<Clause>, String> {
	// basic steps:
	// chew off tokens from the front, recursively parse blocks of tokens
	let mut clauses: Vec<Clause> = Vec::new();
	let mut i = 0usize;
	while let Some(clause) = get_clause_tokens(&tokens[i..])? {
		match (
			&clause[0],
			&clause.get(1).unwrap_or(&Token::None),
			&clause.get(2).unwrap_or(&Token::None),
		) {
			(Token::Struct, Token::Name(_), Token::OpenBrace) => {}
			(Token::Cell, _, _) | (Token::Struct, Token::Name(_), Token::Name(_)) => {
				clauses.push(parse_let_clause(clause)?);
			}
			(Token::Plus, Token::Plus, _) | (Token::Minus, Token::Minus, _) => {
				clauses.push(parse_increment_clause(clause)?);
			}
			(Token::Name(_), Token::EqualsSign, _) => {
				clauses.extend(parse_set_clause(clause)?);
			}
			(Token::Drain, _, _) => {
				clauses.push(parse_drain_copy_clause(clause, true)?);
			}
			(Token::Copy, _, _) => {
				clauses.push(parse_drain_copy_clause(clause, false)?);
			}
			(Token::While, _, _) => {
				clauses.push(parse_while_clause(clause)?);
			}
			(Token::Output, _, _) => {
				clauses.push(parse_output_clause(clause)?);
			}
			(Token::Input, _, _) => {
				clauses.push(parse_input_clause(clause)?);
			}
			(Token::Name(_), Token::OpenAngledBracket, _) => {
				clauses.push(parse_function_call_clause(clause)?);
			}
			(Token::Def, _, _) => {
				clauses.push(parse_function_definition_clause(clause)?);
			}
			(Token::Name(_), Token::Plus | Token::Minus, Token::EqualsSign) => {
				clauses.extend(parse_add_clause(clause)?);
			}
			(Token::If, _, _) => {
				clauses.push(parse_if_else_clause(clause)?);
			}
			(Token::Name(_), Token::OpenSquareBracket, _)
			| (Token::Asterisk, Token::Name(_), _) => {
				let (_, len) = parse_var_target(clause)?;
				let remaining = &clause[len..];
				match (&remaining[0], &remaining[1]) {
					(Token::EqualsSign, _) => {
						clauses.extend(parse_set_clause(clause)?);
					}
					(Token::Plus | Token::Minus, Token::EqualsSign) => {
						clauses.extend(parse_add_clause(clause)?);
					}
					_ => r_panic!("Invalid clause: {clause:#?}"),
				}
			}
			(Token::OpenBrace, _, _) => {
				let braced_tokens = get_braced_tokens(clause, BRACES)?;
				let inner_clauses = parse(braced_tokens)?;
				clauses.push(Clause::Block(inner_clauses));
			}
			(Token::Bf, _, _) => {
				clauses.push(parse_brainfuck_clause(clause)?);
			}
			(Token::Assert, _, _) => clauses.push(parse_assert_clause(clause)?),
			// empty clause
			(Token::Semicolon, _, _) => (),
			// the None token usually represents whitespace, it should be filtered out before reaching this function
			// Wrote out all of these possibilities so that the compiler will tell me when I haven't implemented a token
			(
				Token::None
				| Token::Else
				| Token::Not
				| Token::ClosingBrace
				| Token::OpenSquareBracket
				| Token::ClosingSquareBracket
				| Token::OpenParenthesis
				| Token::ClosingParenthesis
				| Token::OpenAngledBracket
				| Token::ClosingAngledBracket
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
			) => r_panic!("Invalid clause: {clause:#?}"),
		};
		i += clause.len();
	}

	Ok(clauses)
}

fn parse_let_clause(clause: &[Token]) -> Result<Clause, String> {
	// cell x = 0;
	// struct DummyStruct y
	let mut i = 0usize;
	// this kind of logic could probably be done with iterators, (TODO for future refactors)

	let (var, len) = parse_var_definition(&clause[i..])?;
	i += len;

	let (location_specifier, len) = parse_location_specifier(&clause[i..])?;
	i += len;

	if let Token::EqualsSign = &clause[i] {
		i += 1;
		let remaining = &clause[i..(clause.len() - 1)];
		let expr = Expression::parse(remaining)?;
		// equivalent to set clause stuff
		// except we need to convert a variable definition to a variable target
		Ok(Clause::DefineVariable {
			var,
			location_specifier,
			value: expr,
		})
	} else if i < (clause.len() - 1) {
		r_panic!("Invalid token in let clause: {clause:#?}");
	} else {
		Ok(Clause::DeclareVariable {
			var,
			location_specifier,
		})
	}
}

fn parse_add_clause(clause: &[Token]) -> Result<Vec<Clause>, String> {
	let mut clauses: Vec<Clause> = Vec::new();
	let mut i = 0usize;
	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	let positive = match &clause[i] {
		Token::Plus => true,
		Token::Minus => false,
		_ => {
			r_panic!("Unexpected second token in add clause: {clause:#?}");
		}
	};
	i += 2; // assume the equals sign is there because it was checked by the main loop
	let raw_expr = Expression::parse(&clause[i..(clause.len() - 1)])?;
	let expr = match positive {
		true => raw_expr,
		false => Expression::SumExpression {
			sign: Sign::Negative,
			summands: vec![raw_expr],
		},
	};

	clauses.push(Clause::AddToVariable { var, value: expr });

	Ok(clauses)
}

// currently just syntax sugar, should make it actually do post/pre increments
fn parse_increment_clause(clause: &[Token]) -> Result<Clause, String> {
	let (var, _) = parse_var_target(&clause[2..])?;

	Ok(match (&clause[0], &clause[1]) {
		(Token::Plus, Token::Plus) => Clause::AddToVariable {
			var,
			value: Expression::NaturalNumber(1),
		},
		(Token::Minus, Token::Minus) => Clause::AddToVariable {
			var,
			value: Expression::NaturalNumber((-1i8 as u8) as usize),
		},
		_ => {
			r_panic!("Invalid pattern in increment clause: {clause:#?}");
		}
	})
	// assumed that the final token is a semicolon
}

fn parse_set_clause(clause: &[Token]) -> Result<Vec<Clause>, String> {
	// TODO: what do we do about arrays and strings?
	let mut clauses: Vec<Clause> = Vec::new();
	let mut i = 0usize;
	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	// definitely could use iterators instead (TODO for refactor)
	let Token::EqualsSign = &clause[i] else {
		r_panic!("Expected equals sign in set clause: {clause:#?}");
	};
	i += 1;

	let expr = Expression::parse(&clause[i..(clause.len() - 1)])?;

	clauses.push(Clause::SetVariable { var, value: expr });

	Ok(clauses)
}

fn parse_drain_copy_clause(clause: &[Token], is_draining: bool) -> Result<Clause, String> {
	// drain g {i += 1;};
	// drain g into j;
	// copy foo into bar {g += 2; etc;};
	// TODO: make a tuple-parsing function and use it here instead of a space seperated list of targets

	let mut targets = Vec::new();
	let mut block: Vec<Clause> = Vec::new();
	let mut i = 1usize;

	let condition_start_token = i;

	i += 1;
	while let Some(token) = clause.get(i) {
		if let Token::Into | Token::OpenBrace | Token::Semicolon = token {
			break;
		}
		i += 1;
	}
	r_assert!(
		i < clause.len(),
		"Expected source expression in draining/copying loop: {clause:#?}"
	);

	let source = Expression::parse(&clause[condition_start_token..i])?;

	if let Token::Into = &clause[i] {
		// simple drain/copy move operations
		i += 1;

		loop {
			match &clause[i] {
				Token::Name(_) | Token::Asterisk => {
					let (var, len) = parse_var_target(&clause[i..])?;
					i += len;
					targets.push(var);
				}
				Token::OpenBrace | Token::Semicolon => {
					break;
				}
				_ => {
					r_panic!("Unexpected token in drain clause: {clause:#?}");
				}
			}
		}
	}

	if let Token::OpenBrace = &clause[i] {
		// code block to execute at each loop iteration
		let braced_tokens = get_braced_tokens(&clause[i..], BRACES)?;
		// recursion
		block.extend(parse(braced_tokens)?);
		// i += 2 + braced_tokens.len();
	}

	Ok(Clause::CopyLoop {
		source,
		targets,
		block,
		is_draining,
	})
}

fn parse_while_clause(clause: &[Token]) -> Result<Clause, String> {
	// TODO: make this able to accept expressions
	let mut i = 1usize;
	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;
	// loop {
	// 	if let Token::OpenBrace = &clause[i] {
	// 		break;
	// 	};
	// 	i += 1;
	// }

	// let expr = parse_expression(&clause[1..i]);
	let block_tokens = get_braced_tokens(&clause[i..], BRACES)?;
	// i += 2 + block_tokens.len();

	Ok(Clause::WhileLoop {
		var,
		block: parse(block_tokens)?,
	})
}

fn parse_if_else_clause(clause: &[Token]) -> Result<Clause, String> {
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

	let block_one: Vec<Clause> = {
		let block_tokens = get_braced_tokens(&clause[i..], BRACES)?;
		i += 2 + block_tokens.len();
		parse(block_tokens)?
	};

	let block_two: Option<Vec<Clause>> = if let Some(Token::Else) = &clause.get(i) {
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

fn parse_output_clause(clause: &[Token]) -> Result<Clause, String> {
	let mut i = 1usize;

	let expr_tokens = &clause[i..(clause.len() - 1)];
	let expr = Expression::parse(expr_tokens)?;
	i += expr_tokens.len();

	let Token::Semicolon = &clause[i] else {
		r_panic!("Invalid token at end of output clause: {clause:#?}");
	};

	Ok(Clause::OutputValue { value: expr })
}

fn parse_input_clause(clause: &[Token]) -> Result<Clause, String> {
	let mut i = 1usize;

	let (var, len) = parse_var_target(&clause[i..])?;
	i += len;

	let Token::Semicolon = &clause[i] else {
		r_panic!("Invalid token at end of input clause: {clause:#?}");
	};

	Ok(Clause::InputVariable { var })
}

fn parse_assert_clause(clause: &[Token]) -> Result<Clause, String> {
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

// parse any memory location specifiers
// let g @4 = 68;
fn parse_location_specifier(tokens: &[Token]) -> Result<(Option<i32>, usize), String> {
	if let Token::At = &tokens[0] {
		let mut i = 1;
		let positive = if let Token::Minus = &tokens[i] {
			i += 1;
			false
		} else {
			true
		};

		let Token::Digits(raw) = &tokens[i] else {
			r_panic!("Expected constant number in memory location specifier: {tokens:#?}");
		};
		i += 1;

		// TODO: error handling
		let mut offset: i32 = raw.parse().unwrap();
		if !positive {
			offset = -offset;
		}
		Ok((Some(offset), i))
	} else {
		Ok((None, 0))
	}
}

fn parse_brainfuck_clause(clause: &[Token]) -> Result<Clause, String> {
	// bf {++--<><}
	// bf @3 {++--<><}
	// bf clobbers var1 var2 {++--<><}
	// bf @2 clobbers *arr {++--<><}

	let mut clobbers = Vec::new();
	let mut i = 1usize;

	// check for location specifier
	let (mem_offset, len) = parse_location_specifier(&clause[i..])?;
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
		// TODO: support embedded mastermind in the embedded brainfuck
		// TODO: combine [-] into clear opcodes
		match &bf_tokens[j] {
			Token::Plus => ops.push(ExtendedOpcode::Add),
			Token::Minus => ops.push(ExtendedOpcode::Subtract),
			Token::ClosingAngledBracket => ops.push(ExtendedOpcode::Right),
			Token::OpenAngledBracket => ops.push(ExtendedOpcode::Left),
			Token::OpenSquareBracket => ops.push(ExtendedOpcode::OpenLoop),
			Token::ClosingSquareBracket => ops.push(ExtendedOpcode::CloseLoop),
			Token::Dot => ops.push(ExtendedOpcode::Output),
			Token::Comma => ops.push(ExtendedOpcode::Input),
			Token::OpenBrace => {
				// embedded mastermind
				let block_tokens = get_braced_tokens(&bf_tokens[j..], BRACES)?;
				let clauses = parse(block_tokens)?;
				ops.push(ExtendedOpcode::Block(clauses));
				j += block_tokens.len() + 1;
			}
			// not sure whether to panic here or do nothing
			_ => (),
		}
		j += 1;
	}

	Ok(Clause::InlineBrainfuck {
		location_specifier: mem_offset,
		clobbered_variables: clobbers,
		operations: ops,
	})
}

fn parse_function_definition_clause(clause: &[Token]) -> Result<Clause, String> {
	let mut i = 1usize;
	// function name
	let Token::Name(name) = &clause[i] else {
		r_panic!("Expected function name after in function definition clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;
	let Token::OpenAngledBracket = &clause[i] else {
		r_panic!("Expected argument list in function definition clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], ANGLED_BRACKETS)?;
	let mut j = 0usize;
	// parse function argument names
	while j < arg_tokens.len() {
		// this used to be in the while condition but moved it here to check for the case of no arguments
		let Token::Name(_) = &arg_tokens[j] else {
			break;
		};
		let (var, len) = parse_var_definition(&arg_tokens[j..])?;
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
	let parsed_block: Vec<Clause> = parse(block_tokens)?;

	Ok(Clause::DefineFunction {
		name: name.clone(),
		arguments: args,
		block: parsed_block,
	})
}

fn parse_function_call_clause(clause: &[Token]) -> Result<Clause, String> {
	let mut i = 0usize;
	// Okay I didn't know this rust syntax, could have used it all over the place
	let Token::Name(name) = &clause[i] else {
		r_panic!("Expected function identifier at start of function call clause: {clause:#?}");
	};
	let mut args = Vec::new();
	i += 1;

	let Token::OpenAngledBracket = &clause[i] else {
		r_panic!("Expected argument list in function call clause: {clause:#?}");
	};
	let arg_tokens = get_braced_tokens(&clause[i..], ANGLED_BRACKETS)?;

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

fn parse_var_target(tokens: &[Token]) -> Result<(VariableTarget, usize), String> {
	let mut i = 0usize;
	let Token::Name(var_name) = &tokens[i] else {
		r_panic!("Expected identifier in variable target identifier: {tokens:#?}");
	};
	i += 1;

	let mut ref_chain = vec![];
	while i < tokens.len() {
		match &tokens[i] {
			Token::OpenSquareBracket => {
				let (index, tokens_used) = parse_subscript(&tokens[i..])?;
				i += tokens_used;
				ref_chain.push(Reference::Index(index));
			}
			Token::Dot => {
				i += 1;
				let Token::Name(subfield_name) = &tokens[i] else {
					r_panic!("Expected subfield name in variable target identifier: {tokens:#?}");
				};
				i += 1;

				ref_chain.push(Reference::NamedField(subfield_name.clone()));
			}
			_ => {
				break;
			}
		}
	}

	Ok((VariableTarget(var_name.clone(), ref_chain), i))
}

/// convert tokens of a variable definition into data representation, e.g. `cell x`, `struct G g`, `cell[5] x_arr`, `struct H[100] hs`
fn parse_var_definition(tokens: &[Token]) -> Result<(VariableDefinition, usize), String> {
	let mut i = 0usize;
	let mut var_type = match &tokens[i] {
		Token::Cell => {
			i += 1;

			VariableType::Cell
		}
		Token::Struct => {
			i += 1;

			let Token::Name(struct_name) = &tokens[i] else {
				r_panic!("Expected struct type name in variable definition: {tokens:#?}");
			};
			i += 1;

			VariableType::Struct(struct_name.clone())
		}
		_ => {
			r_panic!("Unexpected token in variable definition, this should not occur: {tokens:#?}")
		}
	};

	// parse array specifiers
	// TODO: make this work for multi-dimension arrays [5][4]
	while let Token::OpenSquareBracket = &tokens[i] {
		let (len, j) = parse_array_length(&tokens[i..])?;
		i += j;

		var_type = VariableType::Array(Box::new(var_type), len);
	}

	let Token::Name(var_name) = &tokens[i] else {
		r_panic!("Expected identifier in variable definition: {tokens:#?}");
	};
	i += 1;

	Ok((
		VariableDefinition {
			var_type,
			name: var_name.clone(),
		},
		i,
	))
}

/// parse the subscript of an array variable, e.g. [4] [6]
/// must be compile-time constant
/// returns (array length, tokens used)
/// assumes the first token is an open square bracket
fn parse_subscript(tokens: &[Token]) -> Result<(usize, usize), String> {
	let mut i = 0usize;
	let subscript = get_braced_tokens(&tokens[i..], SQUARE_BRACKETS)?;
	let Expression::NaturalNumber(len) = Expression::parse(subscript)? else {
		r_panic!("Expected a compile-time constant in subscript: {tokens:#?}");
	};

	i += 2 + subscript.len();

	Ok((len, i))
}

/// parse_array_subscript but with a length check
fn parse_array_length(tokens: &[Token]) -> Result<(usize, usize), String> {
	let (len, i) = parse_subscript(tokens)?;
	r_assert!(len > 0, "Array variable cannot be zero-length: {tokens:#?}");
	Ok((len, i))
}

// get a clause, typically a line, bounded by ;
fn get_clause_tokens(tokens: &[Token]) -> Result<Option<&[Token]>, String> {
	if tokens.len() < 2 {
		Ok(None)
	} else {
		let mut i = 0usize;
		while i < tokens.len() {
			match tokens[i] {
				Token::OpenBrace => {
					let braced_block = get_braced_tokens(&tokens[i..], BRACES)?;
					i += 2 + braced_block.len();
					// handle blocks marking the end of clauses, if/else being the exception
					if i < tokens.len() {
						if let Token::Else = tokens[i] {
							i += 1;
							let else_block = get_braced_tokens(&tokens[i..], BRACES)?;
							i += 2 + else_block.len();
						}
					}
					return Ok(Some(&tokens[..i]));
				}
				Token::Semicolon => {
					i += 1;
					return Ok(Some(&tokens[..i]));
				}
				_ => {
					i += 1;
				}
			}
		}

		r_panic!("No clause could be found in: {tokens:#?}");
	}
}

const SQUARE_BRACKETS: (Token, Token) = (Token::OpenSquareBracket, Token::ClosingSquareBracket);
const BRACES: (Token, Token) = (Token::OpenBrace, Token::ClosingBrace);
const PARENTHESES: (Token, Token) = (Token::OpenParenthesis, Token::ClosingParenthesis);
const ANGLED_BRACKETS: (Token, Token) = (Token::OpenAngledBracket, Token::ClosingAngledBracket);
// this should be a generic function but rust doesn't support enum variants as type arguments yet
// find tokens bounded by matching brackets
// TODO: make an impl for &[Token] and put all these functions in it
fn get_braced_tokens(tokens: &[Token], braces: (Token, Token)) -> Result<&[Token], String> {
	let _braces = (discriminant(&braces.0), discriminant(&braces.1));
	// find corresponding bracket, the depth check is unnecessary but whatever
	let len = {
		let mut i = 1usize;
		let mut depth = 1;
		while i < tokens.len() && depth > 0 {
			let g = discriminant(&tokens[i]);
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
		if _braces.0 == discriminant(&tokens[0]) && _braces.1 == discriminant(&tokens[len - 1]) {
			return Ok(&tokens[1..(len - 1)]);
		}
	}
	r_panic!("Invalid braced tokens: {tokens:#?}");
}

impl Expression {
	// Iterators?
	// TODO: support post/pre increment in expressions
	fn parse(tokens: &[Token]) -> Result<Expression, String> {
		let mut i = 0usize;

		if let Token::String(s) = &tokens[i] {
			i += 1;
			r_assert!(
				i == tokens.len(),
				"Expected semicolon after string literal {tokens:#?}"
			);
			return Ok(Expression::StringLiteral(s.clone()));
		}

		if let Token::OpenSquareBracket = &tokens[i] {
			let braced_tokens = get_braced_tokens(&tokens[i..], SQUARE_BRACKETS)?;
			i += 2 + braced_tokens.len();
			r_assert!(
				i == tokens.len(),
				"Expected semicolon after array literal {tokens:#?}"
			);
			// parse the array
			let results: Result<Vec<Expression>, String> = braced_tokens
				.split(|t| if let Token::Comma = t { true } else { false })
				.map(Self::parse)
				.collect();
			// TODO: why do I need to split collect result into a seperate variable like here?
			return Ok(Expression::ArrayLiteral(results?));
		}

		let mut current_sign = Some(Sign::Positive); // by default the first summand is positive
		let mut summands = Vec::new();
		while i < tokens.len() {
			match (&current_sign, &tokens[i]) {
				(None, Token::Plus) => {
					current_sign = Some(Sign::Positive);
					i += 1;
				}
				(None, Token::Minus) => {
					current_sign = Some(Sign::Negative);
					i += 1;
				}
				(Some(Sign::Positive), Token::Minus) => {
					current_sign = Some(Sign::Negative);
					i += 1;
				}
				(Some(Sign::Negative), Token::Minus) => {
					current_sign = Some(Sign::Positive);
					i += 1;
				}
				(Some(sign), Token::Digits(literal)) => {
					let parsed_int: usize = literal.parse().unwrap();
					i += 1;
					match sign {
						Sign::Positive => summands.push(Expression::NaturalNumber(parsed_int)),
						Sign::Negative => summands.push(Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::NaturalNumber(parsed_int)],
						}),
					}
					current_sign = None;
				}
				(Some(sign), Token::True | Token::False) => {
					let parsed_int = match &tokens[i] {
						Token::True => 1,
						Token::False => 0,
						_ => r_panic!(
							"Unreachable error occured while parsing boolean value: {tokens:#?}"
						),
					};
					i += 1;
					match sign {
						Sign::Positive => summands.push(Expression::NaturalNumber(parsed_int)),
						Sign::Negative => summands.push(Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::NaturalNumber(parsed_int)],
						}),
					}
					current_sign = None;
				}
				(Some(sign), Token::Character(chr)) => {
					let chr_int: usize = *chr as usize;

					r_assert!(
						chr_int < 0xff,
						"Character tokens must be single-byte: {chr}"
					);

					i += 1;
					match sign {
						Sign::Positive => summands.push(Expression::NaturalNumber(chr_int)),
						Sign::Negative => summands.push(Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::NaturalNumber(chr_int)],
						}),
					}
					current_sign = None;
				}
				(Some(sign), Token::Name(_) | Token::Asterisk) => {
					let (var, len) = parse_var_target(&tokens[i..])?;
					i += len;
					match sign {
						Sign::Positive => summands.push(Expression::VariableReference(var)),
						Sign::Negative => summands.push(Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::VariableReference(var)],
						}),
					}
					current_sign = None;
				}
				(Some(sign), Token::OpenParenthesis) => {
					let braced_tokens = get_braced_tokens(&tokens[i..], PARENTHESES)?;
					i += 2 + braced_tokens.len();
					let braced_expr = Self::parse(braced_tokens)?;
					// probably inefficent but everything needs to be flattened at some point anyway so won't matter
					// TODO: make expression structure more efficient (don't use vectors every time there is a negative)
					summands.push(match (sign, braced_expr.clone()) {
						(
							Sign::Negative,
							Expression::NaturalNumber(_) | Expression::VariableReference(_),
						) => Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![braced_expr],
						},
						(
							Sign::Negative,
							Expression::SumExpression {
								sign: Sign::Negative,
								summands,
							},
						) => Expression::SumExpression {
							sign: Sign::Positive,
							summands,
						},
						(
							Sign::Negative,
							Expression::SumExpression {
								sign: Sign::Positive,
								summands,
							},
						) => Expression::SumExpression {
							sign: Sign::Negative,
							summands,
						},
						_ => braced_expr,
					});
					current_sign = None;
				}
				_ => {
					r_panic!(
						"Unexpected token {:#?} found in expression: {tokens:#?}",
						tokens[i]
					);
				}
			}
		}

		match summands.len() {
			1 => Ok(summands.into_iter().next().unwrap()),
			1.. => Ok(Expression::SumExpression {
				sign: Sign::Positive,
				summands,
			}),
			_ => r_panic!("Expected value in expression: {tokens:#?}"),
		}
	}

	// not sure if this is the compiler's concern or if it should be the parser
	// (constant to add, variables to add, variables to subtract)
	// currently multiplication is not supported so order of operations and flattening is very trivial
	// If we add multiplication in future it will likely be constant multiplication only, so no variable on variable multiplication
	pub fn flatten(self) -> Result<(u8, Vec<VariableTarget>, Vec<VariableTarget>), String> {
		let expr = self;
		let mut imm_sum = Wrapping(0u8);
		let mut additions = Vec::new();
		let mut subtractions = Vec::new();

		match expr {
			Expression::SumExpression { sign, summands } => {
				let results: Result<Vec<(u8, Vec<VariableTarget>, Vec<VariableTarget>)>, String> =
					summands.into_iter().map(|expr| expr.flatten()).collect();
				let flattened = results?
					.into_iter()
					.reduce(|acc, (imm, adds, subs)| {
						(
							(Wrapping(acc.0) + Wrapping(imm)).0,
							[acc.1, adds].concat(),
							[acc.2, subs].concat(),
						)
					})
					.unwrap_or((0, vec![], vec![]));

				match sign {
					Sign::Positive => {
						imm_sum += flattened.0;
						additions.extend(flattened.1);
						subtractions.extend(flattened.2);
					}
					Sign::Negative => {
						imm_sum -= flattened.0;
						subtractions.extend(flattened.1);
						additions.extend(flattened.2);
					}
				};
			}
			Expression::NaturalNumber(number) => {
				imm_sum += Wrapping(number as u8);
			}
			Expression::VariableReference(var) => {
				additions.push(var);
			}
			Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => {
				r_panic!("Attempt to flatten an array-like expression: {expr:#?}");
			}
		}

		Ok((imm_sum.0, additions, subtractions))
	}
}

// TODO: add multiplication
// yes, but no variable * variable multiplication or division
#[derive(Debug, Clone)]
pub enum Expression {
	SumExpression {
		sign: Sign,
		summands: Vec<Expression>,
	},
	NaturalNumber(usize),
	VariableReference(VariableTarget),
	ArrayLiteral(Vec<Expression>),
	StringLiteral(String),
}

#[derive(Debug, Clone)]
pub enum Sign {
	Positive,
	Negative,
}

#[derive(Debug, Clone)]
pub enum Clause {
	DeclareVariable {
		var: VariableDefinition,
		location_specifier: Option<TapeCell>,
	},
	DefineVariable {
		var: VariableDefinition,
		location_specifier: Option<TapeCell>,
		value: Expression,
	},
	AddToVariable {
		var: VariableTarget,
		value: Expression,
	},
	SetVariable {
		var: VariableTarget,
		value: Expression,
	},
	AssertVariableValue {
		var: VariableTarget,
		// Some(constant) indicates we know the value, None indicates we don't know the value
		// typically will either be used for assert unknown or assert 0
		value: Option<Expression>,
	},
	CopyLoop {
		source: Expression,
		targets: Vec<VariableTarget>,
		block: Vec<Clause>,
		is_draining: bool,
	},
	WhileLoop {
		var: VariableTarget,
		block: Vec<Clause>,
	},
	OutputValue {
		value: Expression,
	},
	InputVariable {
		var: VariableTarget,
	},
	DefineFunction {
		name: String,
		arguments: Vec<VariableDefinition>,
		block: Vec<Clause>,
	},
	CallFunction {
		function_name: String,
		arguments: Vec<VariableTarget>,
	},
	IfElse {
		condition: Expression,
		if_block: Option<Vec<Clause>>,
		else_block: Option<Vec<Clause>>,
	},
	Block(Vec<Clause>),
	InlineBrainfuck {
		location_specifier: Option<TapeCell>,
		clobbered_variables: Vec<VariableTarget>,
		// TODO: make this support embedded mastermind
		operations: Vec<ExtendedOpcode>,
	},
}

// extended brainfuck opcodes to include mastermind code blocks
#[derive(Debug, Clone)]
pub enum ExtendedOpcode {
	Add,
	Subtract,
	Right,
	Left,
	OpenLoop,
	CloseLoop,
	Output,
	Input,
	Block(Vec<Clause>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VariableType {
	Cell,
	Struct(String),
	Array(Box<VariableType>, usize),
}
impl VariableType {
	/// recursively calculate the size in cells of this variable
	pub fn size(&self) -> Option<usize> {
		todo!();
	}
}

// TODO: refactor to this instead:
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VariableDefinition {
	pub var_type: VariableType,
	pub name: String,
	// Infinite {name: String, pattern: ???},
}

impl VariableDefinition {
	pub fn len(&self) -> Option<usize> {
		match self.var_type {
			VariableType::Cell | VariableType::Struct(_) => None,
			VariableType::Array(_, len) => Some(len),
		}
	}
	/// gets this definition as a cell target for definition clauses (as opposed to declarations)
	pub fn target_cell(&self) -> Result<VariableTarget, String> {
		match self.var_type {
			VariableType::Cell => Ok(VariableTarget(self.name.clone(), vec![])),
			VariableType::Struct(_) | VariableType::Array(_, _) => {
				r_panic!(
					"Cannot get single cell target from a struct or array definition ({self:#?})."
				)
			}
		}
	}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Reference {
	NamedField(String),
	Index(usize),
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
// TODO: make the vec reference optional?
pub struct VariableTarget(pub String, pub Vec<Reference>);
impl VariableTarget {
	pub fn name(&self) -> &str {
		&self.0
	}
}

impl Display for VariableType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self {
			VariableType::Cell => f.write_str(&format!("cell")),
			VariableType::Struct(struct_name) => f.write_str(&format!("struct {struct_name}")),
			VariableType::Array(element_type, len) => {
				f.write_str(&format!("{element_type}[{len}]"))
			}
		}
	}
}

impl Display for VariableDefinition {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&format!("{} {}", self.var_type, self.name))
	}
}

impl Display for VariableTarget {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0);
		for ref_step in self.1.iter() {
			match ref_step {
				Reference::NamedField(subfield_name) => {
					f.write_str(&format!(".{subfield_name}"))?
				}
				Reference::Index(index) => f.write_str(&format!("[{index}]"))?,
			}
		}

		Ok(())
	}
}
