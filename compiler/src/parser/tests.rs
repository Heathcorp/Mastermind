use crate::parser::parser::parse_program;

#[cfg(test)]
mod parser_tests {
	use super::super::{
		expressions::{Expression, Sign},
		parser::parse_program,
		types::{
			Clause, ExtendedOpcode, LocationSpecifier, VariableTarget, VariableTypeDefinition,
			VariableTypeReference,
		},
	};
	use crate::backend::{
		bf::{Opcode, TapeCell},
		bf2d::{Opcode2D, TapeCell2D},
	};

	fn _parser_test(raw: &str, expected: &[Clause<TapeCell, Opcode>]) {
		assert_eq!(parse_program(raw).unwrap(), expected);
	}

	fn _parser_test_2d(raw: &str, expected: &[Clause<TapeCell2D, Opcode2D>]) {
		assert_eq!(parse_program(raw).unwrap(), expected);
	}

	#[test]
	fn parse_if_1() {
		_parser_test(
			"if true {{}}",
			&[Clause::If {
				condition: Expression::NaturalNumber(1),
				if_block: vec![Clause::<TapeCell, Opcode>::Block(vec![])],
			}],
		);
	}

	#[test]
	fn end_tokens_1() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("clobbers").unwrap_err(),
			"Invalid starting token: `clobbers`"
		);
	}

	#[test]
	fn end_tokens_2() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>(";").unwrap_err(),
			"Invalid starting token: `;`"
		);
		assert_eq!(
			parse_program::<TapeCell, Opcode>(";;").unwrap_err(),
			"Invalid starting token: `;`"
		);
		assert_eq!(
			parse_program::<TapeCell, Opcode>(";;;").unwrap_err(),
			"Invalid starting token: `;`"
		);
	}

	#[test]
	fn end_tokens_3() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("cell;").unwrap_err(),
			"Expected name in variable definition."
		)
	}

	#[test]
	fn while_condition_1() {
		_parser_test(
			"while x {{}}",
			&[Clause::While {
				var: VariableTarget {
					name: String::from("x"),
					subfields: None,
					is_spread: false,
				},
				block: vec![Clause::Block(vec![])],
			}],
		);
	}

	#[test]
	fn two_dimensional_1() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("cell x @(0, 1);").unwrap_err(),
			"Invalid location specifier @(0, 1)"
		);
	}

	#[test]
	fn two_dimensional_2() {
		_parser_test_2d(
			"cell x @(0, 1);",
			&[Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("x"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::Cell(TapeCell2D(0, 1)),
				},
			}],
		);
	}

	#[test]
	fn two_dimensional_3() {
		_parser_test_2d(
			"cell xyz @(-10, -101);",
			&[Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("xyz"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::Cell(TapeCell2D(-10, -101)),
				},
			}],
		);
	}

	#[test]
	fn var_v_1d() {
		_parser_test(
			"cell v;",
			&[Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("v"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::None,
				},
			}],
		);
	}

	#[test]
	fn var_v_2d() {
		_parser_test_2d(
			"cell v;",
			&[Clause::DeclareVariable {
				var: VariableTypeDefinition {
					name: String::from("v"),
					var_type: VariableTypeReference::Cell,
					location_specifier: LocationSpecifier::None,
				},
			}],
		);
	}

	#[test]
	fn inline_bf_1() {
		_parser_test(
			"cell v; bf {+{cell v;}-}",
			&[
				Clause::DeclareVariable {
					var: VariableTypeDefinition {
						name: String::from("v"),
						var_type: VariableTypeReference::Cell,
						location_specifier: LocationSpecifier::None,
					},
				},
				Clause::InlineBrainfuck {
					location_specifier: LocationSpecifier::None,
					clobbered_variables: vec![],
					operations: vec![
						ExtendedOpcode::Opcode(Opcode::Add),
						ExtendedOpcode::Block(vec![Clause::DeclareVariable {
							var: VariableTypeDefinition {
								name: String::from("v"),
								var_type: VariableTypeReference::Cell,
								location_specifier: LocationSpecifier::None,
							},
						}]),
						ExtendedOpcode::Opcode(Opcode::Subtract),
					],
				},
			],
		);
	}

	#[test]
	fn inline_bf_2() {
		_parser_test_2d(
			"cell v; bf {v{cell v;}^}",
			&[
				Clause::DeclareVariable {
					var: VariableTypeDefinition {
						name: String::from("v"),
						var_type: VariableTypeReference::Cell,
						location_specifier: LocationSpecifier::None,
					},
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
								location_specifier: LocationSpecifier::None,
							},
						}]),
						ExtendedOpcode::Opcode(Opcode2D::Up),
					],
				},
			],
		)
	}

	#[test]
	fn inline_bf_3() {
		_parser_test_2d(
			"bf {vvvv>}",
			&[Clause::InlineBrainfuck {
				location_specifier: LocationSpecifier::None,
				clobbered_variables: vec![],
				operations: vec![
					ExtendedOpcode::Opcode(Opcode2D::Down),
					ExtendedOpcode::Opcode(Opcode2D::Down),
					ExtendedOpcode::Opcode(Opcode2D::Down),
					ExtendedOpcode::Opcode(Opcode2D::Down),
					ExtendedOpcode::Opcode(Opcode2D::Right),
				],
			}],
		)
	}

	#[test]
	fn inline_bf_4() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("bf {vvvv>}").unwrap_err(),
			""
		);
	}

	#[test]
	fn strings_1() {
		_parser_test(
			r#"
cell[5] ggghh = "hello";
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						5,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::StringLiteral(String::from("hello")),
			}],
		);
	}

	#[test]
	fn strings_1a() {
		_parser_test(
			r#"
cell[0] ggghh = "";
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						0,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::StringLiteral(String::from("")),
			}],
		);
	}

	#[test]
	fn strings_1b() {
		_parser_test(
			r#"
cell[1] ggghh = "hello";
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						1,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::StringLiteral(String::from("hello")),
			}],
		);
	}

	#[test]
	fn strings_2() {
		_parser_test(
			r#"
cell[6] ggghh = "hel'lo";
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						6,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::StringLiteral(String::from("hel'lo")),
			}],
		);
	}

	#[test]
	fn strings_3() {
		_parser_test(
			r#"
cell[7] ggghh = "\"hello\"";
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						7,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::StringLiteral(String::from("\"hello\"")),
			}],
		);
	}

	#[test]
	fn arrays_1() {
		_parser_test(
			r#"
cell[0] ggghh = [];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("ggghh"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						0,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::ArrayLiteral(vec![]),
			}],
		);
	}

	#[test]
	fn arrays_2() {
		_parser_test(
			r#"
cell[333] arr = [45, 53];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						333,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::ArrayLiteral(vec![
					Expression::NaturalNumber(45),
					Expression::NaturalNumber(53),
				]),
			}],
		);
	}

	#[test]
	fn arrays_2a() {
		_parser_test(
			r#"
cell[333] arr = [45 + 123, 53];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						333,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::ArrayLiteral(vec![
					Expression::SumExpression {
						sign: Sign::Positive,
						summands: vec![
							Expression::NaturalNumber(45),
							Expression::NaturalNumber(123),
						],
					},
					Expression::NaturalNumber(53),
				]),
			}],
		);
	}

	#[test]
	fn arrays_2b() {
		_parser_test(
			r#"
cell[333] arr = [45 + 123, -(53 + 0+78-9)];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						333,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::ArrayLiteral(vec![
					Expression::SumExpression {
						sign: Sign::Positive,
						summands: vec![
							Expression::NaturalNumber(45),
							Expression::NaturalNumber(123),
						],
					},
					Expression::SumExpression {
						sign: Sign::Negative,
						summands: vec![
							Expression::NaturalNumber(53),
							Expression::NaturalNumber(0),
							Expression::NaturalNumber(78),
							Expression::SumExpression {
								sign: Sign::Negative,
								summands: vec![Expression::NaturalNumber(9)],
							},
						],
					},
				]),
			}],
		);
	}

	#[test]
	fn arrays_3() {
		_parser_test(
			r#"
cell[3] arr = ['h', 53, (((4)))];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						3,
					),
					location_specifier: LocationSpecifier::None,
				},
				value: Expression::ArrayLiteral(vec![
					Expression::NaturalNumber(104),
					Expression::NaturalNumber(53),
					Expression::NaturalNumber(4),
				]),
			}],
		);
	}

	#[test]
	fn arrays_4() {
		_parser_test(
			r#"
struct nonsense[39] arr @-56 = ["hello!", 53, [4,5,6]];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						39,
					),
					location_specifier: LocationSpecifier::Cell(-56),
				},
				value: Expression::ArrayLiteral(vec![
					Expression::StringLiteral(String::from("hello!")),
					Expression::NaturalNumber(53),
					Expression::ArrayLiteral(vec![
						Expression::NaturalNumber(4),
						Expression::NaturalNumber(5),
						Expression::NaturalNumber(6),
					]),
				]),
			}],
		);
	}

	#[test]
	fn arrays_5() {
		_parser_test(
			r#"
struct nonsense[39] arr @-56 = ["hello!", ',', [4,"hello comma: ,",6]];
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						39,
					),
					location_specifier: LocationSpecifier::Cell(-56),
				},
				value: Expression::ArrayLiteral(vec![
					Expression::StringLiteral(String::from("hello!")),
					Expression::NaturalNumber(44),
					Expression::ArrayLiteral(vec![
						Expression::NaturalNumber(4),
						Expression::StringLiteral(String::from("hello comma: ,")),
						Expression::NaturalNumber(6),
					]),
				]),
			}],
		);
	}

	#[test]
	fn sums_1() {
		_parser_test(
			r#"
struct nonsense[39] arr @-56 = 56 - ( 4+3+( -7-5 +(6)-(((( (0) )))) ) );
"#,
			&[Clause::DefineVariable {
				var: VariableTypeDefinition {
					name: String::from("arr"),
					var_type: VariableTypeReference::Array(
						Box::new(VariableTypeReference::Cell),
						39,
					),
					location_specifier: LocationSpecifier::Cell(-56),
				},
				value: Expression::SumExpression {
					sign: Sign::Positive,
					summands: vec![
						Expression::NaturalNumber(56),
						Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![
								Expression::NaturalNumber(4),
								Expression::NaturalNumber(3),
								Expression::SumExpression {
									sign: Sign::Positive,
									summands: vec![
										Expression::SumExpression {
											sign: Sign::Negative,
											summands: vec![Expression::NaturalNumber(7)],
										},
										Expression::SumExpression {
											sign: Sign::Negative,
											summands: vec![Expression::NaturalNumber(5)],
										},
										Expression::NaturalNumber(6),
									],
								},
							],
						},
					],
				},
			}],
		);
	}

	#[test]
	fn blocks_1() {
		_parser_test("{}", &[Clause::Block(vec![])]);
	}

	#[test]
	fn blocks_1a() {
		_parser_test(
			" {}{} {}  {} ",
			&[
				Clause::Block(vec![]),
				Clause::Block(vec![]),
				Clause::Block(vec![]),
				Clause::Block(vec![]),
			],
		);
	}

	#[test]
	fn blocks_1b() {
		_parser_test(
			" {}{{{{}}{}}} {}  {} ",
			&[
				Clause::Block(vec![]),
				Clause::Block(vec![Clause::Block(vec![
					Clause::Block(vec![Clause::Block(vec![])]),
					Clause::Block(vec![]),
				])]),
				Clause::Block(vec![]),
				Clause::Block(vec![]),
			],
		);
	}

	#[test]
	fn blocks_2() {
		_parser_test(
			"{output 1;output 2;}{{{} output 3;}}",
			&[
				Clause::Block(vec![
					Clause::Output {
						value: Expression::NaturalNumber(1),
					},
					Clause::Output {
						value: Expression::NaturalNumber(2),
					},
				]),
				Clause::Block(vec![Clause::Block(vec![
					Clause::Block(vec![]),
					Clause::Output {
						value: Expression::NaturalNumber(3),
					},
				])]),
			],
		);
	}
}
