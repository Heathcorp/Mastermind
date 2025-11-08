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

	#[test]
	fn parse_if_1() {
		assert!(parse_program::<TapeCell, Opcode>("if true {{}}")
			.unwrap()
			.iter()
			.eq(&[Clause::If {
				condition: Expression::NaturalNumber(1),
				if_block: vec![Clause::<TapeCell, Opcode>::Block(vec![])],
			}]));
	}

	#[test]
	fn end_tokens_1() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("clobbers").unwrap_err(),
			""
		);
	}

	#[test]
	fn end_tokens_2() {
		assert_eq!(parse_program::<TapeCell, Opcode>(";").unwrap_err(), "");
		assert_eq!(parse_program::<TapeCell, Opcode>(";;").unwrap_err(), "");
		assert_eq!(parse_program::<TapeCell, Opcode>(";;;").unwrap_err(), "");
	}

	#[test]
	fn end_tokens_3() {
		assert_eq!(parse_program::<TapeCell, Opcode>("cell;").unwrap_err(), "")
	}

	#[test]
	fn while_condition_1() {
		assert!(parse_program::<TapeCell, Opcode>("while x {{}}")
			.unwrap()
			.iter()
			.eq(&[Clause::While {
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
		assert_eq!(
			parse_program::<TapeCell, Opcode>("cell x @(0, 1);").unwrap_err(),
			"Invalid location specifier @(0, 1)"
		);
	}

	#[test]
	fn two_dimensional_2() {
		assert!(parse_program::<TapeCell2D, Opcode2D>("cell x @(0, 1);")
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
		assert!(
			parse_program::<TapeCell2D, Opcode2D>("cell xyz @(-10, -101);")
				.unwrap()
				.iter()
				.eq(&[Clause::DeclareVariable {
					var: VariableTypeDefinition {
						name: String::from("xyz"),
						var_type: VariableTypeReference::Cell,
						location_specifier: LocationSpecifier::Cell(TapeCell2D(-10, -101))
					}
				}])
		);
	}

	#[test]
	fn var_v_1d() {
		assert!(parse_program::<TapeCell, Opcode>("cell v;")
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
	fn var_v_2d() {
		assert!(parse_program::<TapeCell2D, Opcode2D>("cell v;")
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
		assert!(
			parse_program::<TapeCell, Opcode>("cell v; bf {+{cell v;}-}")
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
							ExtendedOpcode::Opcode(Opcode::Add),
							ExtendedOpcode::Block(vec![Clause::DeclareVariable {
								var: VariableTypeDefinition {
									name: String::from("v"),
									var_type: VariableTypeReference::Cell,
									location_specifier: LocationSpecifier::None
								}
							}]),
							ExtendedOpcode::Opcode(Opcode::Subtract),
						]
					}
				])
		)
	}

	#[test]
	fn inline_bf_2() {
		assert!(
			parse_program::<TapeCell2D, Opcode2D>("cell v; bf {v{cell v;}^}")
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
				])
		)
	}

	#[test]
	fn inline_bf_3() {
		assert!(parse_program::<TapeCell2D, Opcode2D>("bf {vvvv>}")
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

	#[test]
	fn inline_bf_4() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>("bf {vvvv>}").unwrap_err(),
			""
		);
	}

	#[test]
	fn strings_1() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[5] ggghh = "hello";
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("ggghh"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 5),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::StringLiteral(String::from("hello"))
		}]));
	}

	#[test]
	fn strings_1a() {
		assert_eq!(
			parse_program::<TapeCell, Opcode>(
				r#"
cell[0] ggghh = "";
"#
			)
			.unwrap_err(),
			""
		);
	}

	#[test]
	fn strings_1b() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[1] ggghh = "hello";
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("ggghh"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 1),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::StringLiteral(String::from("hello"))
		}]));
	}

	#[test]
	fn strings_2() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[6] ggghh = "hel'lo";
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("ggghh"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 6),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::StringLiteral(String::from("hel'lo"))
		}]));
	}

	#[test]
	fn strings_3() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[7] ggghh = "\"hello\"";
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("ggghh"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 7),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::StringLiteral(String::from("\"hello\""))
		}]));
	}

	#[test]
	fn arrays_1() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[0] ggghh = [];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("ggghh"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 0),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::ArrayLiteral(vec![])
		}]));
	}

	#[test]
	fn arrays_2() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[333] arr = [45, 53];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 333),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::ArrayLiteral(vec![
				Expression::NaturalNumber(45),
				Expression::NaturalNumber(53)
			])
		}]));
	}

	#[test]
	fn arrays_2a() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[333] arr = [45 + 123, 53];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 333),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::ArrayLiteral(vec![
				Expression::SumExpression {
					sign: Sign::Positive,
					summands: vec![
						Expression::NaturalNumber(45),
						Expression::NaturalNumber(123)
					]
				},
				Expression::NaturalNumber(53)
			])
		}]));
	}

	#[test]
	fn arrays_2b() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[333] arr = [45 + 123, -(53 + 0+78-9)];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 333),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::ArrayLiteral(vec![
				Expression::SumExpression {
					sign: Sign::Positive,
					summands: vec![
						Expression::NaturalNumber(45),
						Expression::NaturalNumber(123)
					]
				},
				Expression::SumExpression {
					sign: Sign::Negative,
					summands: vec![
						Expression::NaturalNumber(53),
						Expression::NaturalNumber(0),
						Expression::NaturalNumber(78),
						Expression::SumExpression {
							sign: Sign::Negative,
							summands: vec![Expression::NaturalNumber(9)]
						}
					]
				}
			])
		}]));
	}

	#[test]
	fn arrays_3() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
cell[3] arr = ['h', 53, (((4)))];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 3),
				location_specifier: LocationSpecifier::None
			},
			value: Expression::ArrayLiteral(vec![
				Expression::NaturalNumber(104),
				Expression::NaturalNumber(53),
				Expression::NaturalNumber(4)
			])
		}]));
	}

	#[test]
	fn arrays_4() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
struct nonsense[39] arr @-56 = ["hello!", 53, [4,5,6]];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 39),
				location_specifier: LocationSpecifier::Cell(-56)
			},
			value: Expression::ArrayLiteral(vec![
				Expression::StringLiteral(String::from("hello!")),
				Expression::NaturalNumber(53),
				Expression::ArrayLiteral(vec![
					Expression::NaturalNumber(4),
					Expression::NaturalNumber(5),
					Expression::NaturalNumber(6)
				])
			])
		}]));
	}

	#[test]
	fn arrays_5() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
struct nonsense[39] arr @-56 = ["hello!", ',', [4,"hello comma: ,",6]];
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 39),
				location_specifier: LocationSpecifier::Cell(-56)
			},
			value: Expression::ArrayLiteral(vec![
				Expression::StringLiteral(String::from("hello!")),
				Expression::NaturalNumber(44),
				Expression::ArrayLiteral(vec![
					Expression::NaturalNumber(4),
					Expression::StringLiteral(String::from("hello comma: ,")),
					Expression::NaturalNumber(6)
				])
			])
		}]));
	}

	#[test]
	fn sums_1() {
		assert!(parse_program::<TapeCell, Opcode>(
			r#"
struct nonsense[39] arr @-56 = 56 - ( 4+3+( -7-5 +(6)-(((( (0) )))) ) );
"#
		)
		.unwrap()
		.iter()
		.eq(&[Clause::DefineVariable {
			var: VariableTypeDefinition {
				name: String::from("arr"),
				var_type: VariableTypeReference::Array(Box::new(VariableTypeReference::Cell), 39),
				location_specifier: LocationSpecifier::Cell(-56)
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
										summands: vec![Expression::NaturalNumber(7)]
									},
									Expression::SumExpression {
										sign: Sign::Negative,
										summands: vec![Expression::NaturalNumber(5)]
									},
									Expression::NaturalNumber(6)
								]
							}
						]
					}
				]
			}
		}]));
	}

	#[test]
	fn blocks_1() {
		assert!(parse_program::<TapeCell, Opcode>("{}")
			.unwrap()
			.iter()
			.eq(&[Clause::Block(vec![])]));
	}

	#[test]
	fn blocks_1a() {
		assert!(parse_program::<TapeCell, Opcode>(" {}{} {}  {} ")
			.unwrap()
			.iter()
			.eq(&[
				Clause::Block(vec![]),
				Clause::Block(vec![]),
				Clause::Block(vec![]),
				Clause::Block(vec![])
			]));
	}

	#[test]
	fn blocks_1b() {
		assert!(parse_program::<TapeCell, Opcode>(" {}{{{{}}{}}} {}  {} ")
			.unwrap()
			.iter()
			.eq(&[
				Clause::Block(vec![]),
				Clause::Block(vec![Clause::Block(vec![
					Clause::Block(vec![Clause::Block(vec![])]),
					Clause::Block(vec![])
				])]),
				Clause::Block(vec![]),
				Clause::Block(vec![])
			]));
	}

	#[test]
	fn blocks_2() {
		assert!(
			parse_program::<TapeCell, Opcode>("{output 1;output 2;}{{{} output 3;}}")
				.unwrap()
				.iter()
				.eq(&[
					Clause::Block(vec![
						Clause::OutputValue {
							value: Expression::NaturalNumber(1),
						},
						Clause::OutputValue {
							value: Expression::NaturalNumber(2),
						}
					]),
					Clause::Block(vec![Clause::Block(vec![
						Clause::Block(vec![]),
						Clause::OutputValue {
							value: Expression::NaturalNumber(3)
						}
					])])
				])
		);
	}
}
