use super::{
	parser::parse_var_target,
	tokeniser::{next_token, Token},
	types::VariableTarget,
};
use crate::macros::macros::{r_assert, r_panic};

use std::num::Wrapping;

// TODO: add multiplication
// yes, but no variable * variable multiplication or division
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
#[cfg_attr(test, derive(PartialEq))]
pub enum Sign {
	Positive,
	Negative,
}
impl Sign {
	fn flipped(self) -> Sign {
		match self {
			Sign::Positive => Sign::Negative,
			Sign::Negative => Sign::Positive,
		}
	}
}

impl Expression {
	// Iterators?
	// TODO: support post/pre increment in expressions
	pub fn parse(chars: &mut &[char]) -> Result<Expression, String> {
		{
			let mut s = *chars;
			if let Ok(Token::String(literal)) = next_token(&mut s) {
				let Ok(Token::None) = next_token(&mut s) else {
					// TODO: add source snippet
					r_panic!("String literal must entirely comprise expression.");
				};
				return Ok(Expression::StringLiteral(literal));
			}
		}

		{
			let mut s = *chars;
			if let Ok(Token::LeftSquareBracket) = next_token(&mut s) {
				*chars = s;
				let mut expressions = vec![];
				loop {
					expressions.push(Self::parse(chars)?);
					match next_token(chars) {
						Ok(Token::RightSquareBracket) => break,
						Ok(Token::Comma) => {
							*chars = s;
						}
						_ => r_panic!("Unexpected token in array literal."),
					}
				}
				s = *chars;
				// check for delimiters
				let Ok(
					Token::Semicolon
					| Token::Comma
					| Token::RightParenthesis
					| Token::RightSquareBracket
					| Token::None,
				) = next_token(&mut s)
				else {
					// TODO: add source snippet
					r_panic!("Array literal must entirely comprise expression.");
				};
				return Ok(Expression::ArrayLiteral(expressions));
			}
		}

		// this loop is basically a state machine based on the current sign:
		let mut current_sign = Some(Sign::Positive); // by default the first summand is positive
		let mut summands = Vec::new();
		loop {
			let mut s = *chars;
			match (&current_sign, next_token(&mut s)) {
				(None, Ok(Token::Plus)) => {
					*chars = s;
					current_sign = Some(Sign::Positive);
				}
				(None, Ok(Token::Minus)) => {
					*chars = s;
					current_sign = Some(Sign::Negative);
				}
				(Some(Sign::Positive), Ok(Token::Minus)) => {
					*chars = s;
					current_sign = Some(Sign::Negative);
				}
				(Some(Sign::Negative), Ok(Token::Minus)) => {
					*chars = s;
					current_sign = Some(Sign::Positive);
				}
				(
					Some(sign),
					Ok(
						token @ (Token::Digits(_)
						| Token::Character(_)
						| Token::True
						| Token::False),
					),
				) => {
					*chars = s;
					let parsed_int = match token {
						Token::Digits(digits) => digits.parse::<usize>().unwrap(),
						Token::Character(c) => {
							let chr_int = c as usize;
							r_assert!(chr_int < 0xff, "Character tokens must be single-byte: {c}");
							chr_int
						}
						Token::True => 1,
						Token::False => 0,
						_ => unreachable!(),
					};
					summands.push(match sign {
						Sign::Positive => Expression::NaturalNumber(parsed_int),
						Sign::Negative => Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::NaturalNumber(parsed_int)],
						},
					});
					current_sign = None;
				}
				(Some(sign), Ok(Token::Name(_) | Token::Asterisk)) => {
					let var = parse_var_target(chars)?;
					summands.push(match sign {
						Sign::Positive => Expression::VariableReference(var),
						Sign::Negative => Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::VariableReference(var)],
						},
					});
					current_sign = None;
				}
				(Some(sign), Ok(Token::LeftParenthesis)) => {
					*chars = s;
					let braced_expr = Self::parse(chars)?;
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
				// TODO: add delimiters here: `)` `;` `,` `{` `into`
				(
					sign,
					Ok(
						Token::RightParenthesis
						| Token::Semicolon
						| Token::Comma
						| Token::LeftBrace
						| Token::Into,
					),
				) => {
					r_assert!(sign.is_none(), "Expected more terms in expression.");
					break;
				}
				// TODO: add source snippet
				token => r_panic!("Unexpected token {token:#?} found in expression."),
			}
		}

		Ok(match summands.len() {
			1 => summands.into_iter().next().unwrap(),
			1.. => Expression::SumExpression {
				sign: Sign::Positive,
				summands,
			},
			// TODO: add source snippet
			_ => r_panic!("Expected value in expression."),
		})
	}

	/// flip the sign of an expression, equivalent to `x => -(x)`
	pub fn flipped_sign(self) -> Result<Self, String> {
		Ok(match self {
			Expression::SumExpression { sign, summands } => Expression::SumExpression {
				sign: sign.flipped(),
				summands,
			},
			Expression::NaturalNumber(_) | Expression::VariableReference(_) => {
				Expression::SumExpression {
					sign: Sign::Negative,
					summands: vec![self],
				}
			}
			Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => {
				r_panic!(
					"Attempted to invert sign of array or string literal, \
	do not use += or -= on arrays or strings."
				);
			}
		})
	}

	// not sure if this is the compiler's concern or if it should be the parser
	// (constant to add, variables to add, variables to subtract)
	// currently multiplication is not supported so order of operations and flattening is very trivial
	// If we add multiplication in future it will likely be constant multiplication only, so no variable on variable multiplication
	pub fn flatten(&self) -> Result<(u8, Vec<VariableTarget>, Vec<VariableTarget>), String> {
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
				imm_sum += Wrapping(*number as u8);
			}
			Expression::VariableReference(var) => {
				additions.push(var.clone());
			}
			Expression::ArrayLiteral(_) | Expression::StringLiteral(_) => {
				r_panic!("Attempt to flatten an array-like expression: {expr:#?}");
			}
		}

		Ok((imm_sum.0, additions, subtractions))
	}

	//Recursively Check If This Is Self Referencing
	pub fn check_self_referencing(&self, parent: &VariableTarget) -> bool {
		// TODO: make sure nested values work correctly
		match self {
			Expression::SumExpression {
				sign: _sign,
				summands,
			} => summands
				.iter()
				.any(|summand| summand.check_self_referencing(parent)),
			Expression::VariableReference(var) => *var == *parent,
			Expression::ArrayLiteral(_)
			| Expression::StringLiteral(_)
			| Expression::NaturalNumber(_) => false,
		}
	}
}
