#[cfg(test)]
mod tokeniser_tests {
	use super::super::{parser::next_token, types::Token};

	fn tokenise(input_str: &str) -> Result<Vec<Token>, String> {
		let mut tokens = vec![];
		let chars = input_str.chars().collect::<Vec<char>>();
		let mut c = &chars;
		while let Some(token) = next_token(&c)? {
			tokens.push(token);
		}
		Ok(tokens)
	}

	fn _tokenisation_test(input_str: &str, desired_output: &[Token]) {
		let input_string = String::from(input_str);
		let actual_output = tokenise(&input_string).unwrap();
		println!("desired: {desired_output:#?}");
		println!("actual: {actual_output:#?}");
		assert!(actual_output.iter().eq(desired_output));
	}

	#[test]
	fn character_literals_1() {
		_tokenisation_test(
			r#"'a' 'b' 'c' ' '"#,
			&[
				Token::Character('a'),
				Token::Character('b'),
				Token::Character('c'),
				Token::Character(' '),
			],
		);
	}

	#[test]
	fn character_literals_2() {
		_tokenisation_test(r#"'\n'"#, &[Token::Character('\n')]);
	}

	#[test]
	fn character_literals_3() {
		_tokenisation_test(r#"'"'"#, &[Token::Character('"')]);
	}

	#[test]
	fn character_literals_4() {
		_tokenisation_test(r#"'\''"#, &[Token::Character('\'')]);
	}

	#[test]
	#[should_panic]
	fn character_literals_5() {
		_tokenisation_test(r#"'\'"#, &[Token::Character('\\')]);
	}

	#[test]
	#[should_panic]
	fn character_literals_6() {
		_tokenisation_test(r#"'aa'"#, &[Token::String(String::from("aa"))]);
	}

	#[test]
	fn string_literals_1() {
		_tokenisation_test("\"hello\"", &[Token::String(String::from("hello"))]);
	}

	#[test]
	fn string_literals_2() {
		_tokenisation_test(r#""""#, &[Token::String(String::from(""))]);
	}

	#[test]
	fn string_literals_2a() {
		_tokenisation_test(
			r#""""""#,
			&[
				Token::String(String::from("")),
				Token::String(String::from("")),
			],
		);
	}

	#[test]
	fn string_literals_3() {
		_tokenisation_test(
			r#""\"" " ""#,
			&[
				Token::String(String::from("\"")),
				Token::String(String::from(" ")),
			],
		);
	}
}

#[cfg(test)]
mod parser_tests {
	use super::super::{
		expressions::Expression,
		types::{
			Clause, ExtendedOpcode, LocationSpecifier, Token, VariableTarget,
			VariableTypeDefinition, VariableTypeReference,
		},
	};
	use crate::backend::{
		bf::TapeCell,
		bf2d::{Opcode2D, TapeCell2D},
	};

	#[test]
	fn parse_if_1() {
		assert!(parse::<TapeCell, Opcode2D>(&[
			// if true {{}}
			Token::If,
			Token::True,
			Token::OpenBrace,
			Token::OpenBrace,
			Token::ClosingBrace,
			Token::ClosingBrace,
		])
		.unwrap()
		.iter()
		.eq(&[Clause::IfElse {
			condition: Expression::NaturalNumber(1),
			if_block: Some(vec![Clause::<TapeCell, Opcode2D>::Block(vec![])]),
			else_block: None,
		}]));
	}

	#[test]
	fn end_tokens_1() {
		let _ = parse::<TapeCell, Opcode2D>(&[Token::Clobbers]).expect_err("");
	}

	#[test]
	fn end_tokens_2() {
		let _ = parse::<TapeCell, Opcode2D>(&[Token::Semicolon]).unwrap();
		let _ = parse::<TapeCell, Opcode2D>(&[Token::Semicolon, Token::Semicolon]).unwrap();
		let _ =
			parse::<TapeCell, Opcode2D>(&[Token::Semicolon, Token::Semicolon, Token::Semicolon])
				.unwrap();
	}

	#[test]
	fn end_tokens_3() {
		let _ = parse::<TapeCell, Opcode2D>(&[Token::Cell, Token::Semicolon]).expect_err("");
	}

	#[test]
	fn while_condition_1() {
		assert!(parse::<TapeCell, Opcode2D>(&[
			Token::While,
			Token::Name(String::from("x")),
			Token::OpenBrace,
			Token::OpenBrace,
			Token::ClosingBrace,
			Token::ClosingBrace,
		])
		.unwrap()
		.iter()
		.eq(&[Clause::WhileLoop {
			var: VariableTarget {
				name: String::from("x"),
				subfields: None,
				is_spread: false
			},
			block: vec![Clause::Block(vec![])]
		}]))
	}

	#[test]
	fn two_dimensional_1() {
		assert!(parse::<TapeCell, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("x")),
			Token::At,
			Token::OpenParenthesis,
			Token::Digits(String::from("0")),
			Token::Comma,
			Token::Digits(String::from("1")),
			Token::ClosingParenthesis,
			Token::Semicolon,
		])
		.unwrap_err()
		.contains("Invalid location specifier"));
	}

	#[test]
	fn two_dimensional_2() {
		assert!(parse::<TapeCell2D, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("x")),
			Token::At,
			Token::OpenParenthesis,
			Token::Digits(String::from("0")),
			Token::Comma,
			Token::Digits(String::from("1")),
			Token::ClosingParenthesis,
			Token::Semicolon,
		])
		.unwrap()
		.iter()
		.eq(&[Clause::DeclareVariable {
			var: VariableTypeDefinition {
				name: String::from("x"),
				var_type: VariableTypeReference::Cell,
				location_specifier: LocationSpecifier::Cell(TapeCell2D(0, 1))
			}
		}]));
	}

	#[test]
	fn two_dimensional_3() {
		assert!(parse::<TapeCell2D, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("xyz")),
			Token::At,
			Token::OpenParenthesis,
			Token::Minus,
			Token::Digits(String::from("10")),
			Token::Comma,
			Token::Minus,
			Token::Digits(String::from("101")),
			Token::ClosingParenthesis,
			Token::Semicolon,
		])
		.unwrap()
		.iter()
		.eq(&[Clause::DeclareVariable {
			var: VariableTypeDefinition {
				name: String::from("xyz"),
				var_type: VariableTypeReference::Cell,
				location_specifier: LocationSpecifier::Cell(TapeCell2D(-10, -101))
			}
		}]));
	}

	#[test]
	fn var_v() {
		assert!(parse::<TapeCell, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("v")),
			Token::Semicolon
		])
		.unwrap()
		.iter()
		.eq(&[Clause::DeclareVariable {
			var: VariableTypeDefinition {
				name: String::from("v"),
				var_type: VariableTypeReference::Cell,
				location_specifier: LocationSpecifier::None
			}
		}]))
	}

	#[test]
	fn inline_bf_1() {
		assert!(parse::<TapeCell, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("v")),
			Token::Semicolon,
			Token::Bf,
			Token::OpenBrace,
			Token::Plus,
			Token::OpenBrace,
			Token::Cell,
			Token::Name(String::from("v")),
			Token::Semicolon,
			Token::ClosingBrace,
			Token::Minus,
			Token::ClosingBrace
		])
		.unwrap()
		.iter()
		.eq(&[
			Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("v"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::None
				}
			},
			Clause::InlineBrainfuck {
				location_specifier: LocationSpecifier::None,
				clobbered_variables: vec![],
				operations: vec![
					ExtendedOpcode::Opcode(Opcode2D::Add),
					ExtendedOpcode::Block(vec![Clause::DeclareVariable {
						var: VariableTypeDefinition {
							name: String::from("v"),
							var_type: VariableTypeReference::Cell,
							location_specifier: LocationSpecifier::None
						}
					}]),
					ExtendedOpcode::Opcode(Opcode2D::Subtract),
				]
			}
		]))
	}

	// TODO: make context-based parser for brainfuck and refactor these tests
	#[test]
	fn inline_bf_2() {
		assert!(parse::<TapeCell2D, Opcode2D>(&[
			Token::Cell,
			Token::Name(String::from("v")),
			Token::Semicolon,
			Token::Bf,
			Token::OpenBrace,
			Token::Name(String::from("v")),
			Token::OpenBrace,
			Token::Cell,
			Token::Name(String::from("v")),
			Token::Semicolon,
			Token::ClosingBrace,
			Token::Caret,
			Token::ClosingBrace
		])
		.unwrap()
		.iter()
		.eq(&[
			Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("v"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::None
				}
			},
			Clause::InlineBrainfuck {
				location_specifier: LocationSpecifier::None,
				clobbered_variables: vec![],
				operations: vec![
					ExtendedOpcode::Opcode(Opcode2D::Down),
					ExtendedOpcode::Block(vec![Clause::DeclareVariable {
						var: VariableTypeDefinition {
							name: String::from("v"),
							var_type: VariableTypeReference::Cell,
							location_specifier: LocationSpecifier::None
						}
					}]),
					ExtendedOpcode::Opcode(Opcode2D::Up),
				]
			}
		]))
	}

	#[test]
	fn inline_bf_3() {
		assert!(parse::<TapeCell2D, Opcode2D>(&[
			Token::Bf,
			Token::OpenBrace,
			Token::Name(String::from("vvvv")),
			Token::MoreThan,
			Token::ClosingBrace
		])
		.unwrap()
		.iter()
		.eq(&[Clause::InlineBrainfuck {
			location_specifier: LocationSpecifier::None,
			clobbered_variables: vec![],
			operations: vec![
				ExtendedOpcode::Opcode(Opcode2D::Down),
				ExtendedOpcode::Opcode(Opcode2D::Down),
				ExtendedOpcode::Opcode(Opcode2D::Down),
				ExtendedOpcode::Opcode(Opcode2D::Down),
				ExtendedOpcode::Opcode(Opcode2D::Right),
			]
		}]))
	}
}
