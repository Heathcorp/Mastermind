#[cfg(test)]
mod parser_tests {
	use super::super::{
		expressions::Expression,
		parser::parse_program,
		tokeniser::Token,
		types::{
			Clause, ExtendedOpcode, LocationSpecifier, VariableTarget, VariableTypeDefinition,
			VariableTypeReference,
		},
	};
	use crate::{
		backend::{
			bf::{Opcode, TapeCell},
			bf2d::{Opcode2D, TapeCell2D},
		},
		macros::macros::r_assert,
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
}
