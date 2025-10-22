use itertools::Itertools;
use std::{collections::HashMap, num::Wrapping};

use crate::{
	backend::bf2d::{Opcode2D, TapeCell2D},
	misc::MastermindContext,
};

// originally trivial post-compilation brainfuck optimisations
// extended to 2D which makes it more difficult
impl MastermindContext {
	pub fn optimise_bf_code(&self, program: Vec<Opcode2D>) -> Vec<Opcode2D> {
		let mut output = Vec::new();

		// get stretch of characters to optimise (+-<>)
		let mut i = 0;
		let mut subset = Vec::new();
		while i < program.len() {
			let op = program[i];
			match op {
				Opcode2D::Add
				| Opcode2D::Subtract
				| Opcode2D::Right
				| Opcode2D::Left
				| Opcode2D::Clear
				| Opcode2D::Up
				| Opcode2D::Down => {
					subset.push(op);
				}
				Opcode2D::OpenLoop | Opcode2D::CloseLoop | Opcode2D::Input | Opcode2D::Output => {
					// optimise subset and push
					let optimised_subset = self.optimise_subset(subset);
					output.extend(optimised_subset);

					subset = Vec::new();
					output.push(op);
				}
			}
			i += 1;
		}

		output
	}

	fn optimise_subset(&self, run: Vec<Opcode2D>) -> Vec<Opcode2D> {
		#[derive(Clone)]
		enum Change {
			Add(Wrapping<i8>),
			Set(Wrapping<i8>),
		}
		let mut tape: HashMap<TapeCell2D, Change> = HashMap::new();
		let start = TapeCell2D(0, 0);
		let mut head = TapeCell2D(0, 0);
		let mut i = 0;
		// simulate the subprogram to find the exact changes made to the tape
		while i < run.len() {
			let op = run[i];
			match op {
				Opcode2D::Clear => {
					tape.insert(head, Change::Set(Wrapping(0i8)));
				}
				Opcode2D::Subtract | Opcode2D::Add => {
					let mut change = tape.remove(&head).unwrap_or(Change::Add(Wrapping(0i8)));

					let (Change::Add(val) | Change::Set(val)) = &mut change;
					*val += match op {
						Opcode2D::Add => 1,
						Opcode2D::Subtract => -1,
						_ => 0,
					};

					match &change {
						Change::Add(val) => {
							if *val != Wrapping(0i8) {
								tape.insert(head, change);
							}
						}
						Change::Set(_) => {
							tape.insert(head, change);
						}
					}
				}
				Opcode2D::Right => {
					head.0 += 1;
				}
				Opcode2D::Left => {
					head.0 -= 1;
				}
				Opcode2D::Up => {
					head.1 += 1;
				}
				Opcode2D::Down => {
					head.1 -= 1;
				}
				_ => (),
			}
			i += 1;
		}
		let mut output = Vec::new();
		if self.config.optimise_generated_all_permutations {
			//Exhaustive approach checks all permutations
			let mut output_length = i32::MAX;
			let mut best_permutation = Vec::new();
			for perm in tape.iter().permutations(tape.len()) {
				let mut position = start;
				let mut current_output_length = 0;
				//Calculate the distance of this
				for (cell, _) in &perm {
					current_output_length += (cell.0 - position.0).abs();
					current_output_length += (cell.1 - position.1).abs();
					position = **cell;
					if current_output_length > output_length {
						break;
					}
				}
				if current_output_length > output_length {
					continue;
				}
				//Add the distance to the finishing location
				current_output_length += (head.0 - position.0).abs();
				current_output_length += (head.1 - position.1).abs();
				if current_output_length < output_length {
					best_permutation = perm;
					output_length = current_output_length;
				}
			}
			let mut position = start;
			for (cell, change) in best_permutation {
				output = _move_position(output, &position, cell);
				position = *cell;
				if let Change::Set(_) = change {
					output.push(Opcode2D::Clear);
				}
				let (Change::Add(v) | Change::Set(v)) = change;
				let v = v.0;
				for _ in 0..(v as i32).abs() {
					output.push(match v == -128 || v > 0 {
						true => Opcode2D::Add,
						false => Opcode2D::Subtract,
					});
				}
			}
			output = _move_position(output, &position, &head);
		} else {
			//Greedy approach faster for bigger datasets
			let mut position = start;
			//For the number of cells navigate to the nearest cell
			for _ in 0..tape.len() {
				if !tape.is_empty() {
					let mut min_distance = i32::MAX;
					let mut next_position = TapeCell2D(0, 0);
					for (cell, _value) in tape.iter() {
						if (cell.0 - position.0).abs() + (cell.1 - position.1).abs() < min_distance
						{
							min_distance =
								(cell.0 - position.0).abs() + (cell.1 - position.1).abs();
							next_position = *cell;
						}
					}
					// Move to next position
					output = _move_position(output, &position, &next_position);
					position = next_position;
					//Now Update the output with correct opcodes
					let change = tape.remove(&next_position).unwrap();
					if let Change::Set(_) = change {
						output.push(Opcode2D::Clear);
					}
					let (Change::Add(v) | Change::Set(v)) = change;
					let v = v.0;
					for _ in 0..(v as i32).abs() {
						output.push(match v == -128 || v > 0 {
							true => Opcode2D::Add,
							false => Opcode2D::Subtract,
						});
					}
				}
			}
			output = _move_position(output, &position, &head);
		}
		output
	}
}

fn _move_position(
	mut program: Vec<Opcode2D>,
	old_position: &TapeCell2D,
	new_position: &TapeCell2D,
) -> Vec<Opcode2D> {
	if old_position != new_position {
		if old_position.0 < new_position.0 {
			for _ in 0..(new_position.0 - old_position.0) {
				program.push(Opcode2D::Right);
			}
		} else {
			for _ in 0..(old_position.0 - new_position.0) {
				program.push(Opcode2D::Left);
			}
		}
		if old_position.1 < new_position.1 {
			for _ in 0..(new_position.1 - old_position.1) {
				program.push(Opcode2D::Up);
			}
		} else {
			for _ in 0..(old_position.1 - new_position.1) {
				program.push(Opcode2D::Down);
			}
		}
	}
	program
}

#[cfg(test)]
mod bf_optimiser_tests {
	use crate::{
		backend::common::BrainfuckProgram,
		misc::{MastermindConfig, MastermindContext},
	};

	const CTX_OPT: MastermindContext = MastermindContext {
		config: MastermindConfig {
			optimise_generated_code: true,
			optimise_generated_all_permutations: false,
			optimise_cell_clearing: false,
			optimise_unreachable_loops: false,
			optimise_variable_usage: false,
			optimise_memory_allocation: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
			memory_allocation_method: 0,
			enable_2d_grid: false,
		},
	};

	const CTX_OPT_EXHAUSTIVE: MastermindContext = MastermindContext {
		config: MastermindConfig {
			optimise_generated_code: true,
			optimise_generated_all_permutations: true,
			optimise_cell_clearing: false,
			optimise_unreachable_loops: false,
			optimise_variable_usage: false,
			optimise_memory_allocation: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
			memory_allocation_method: 0,
			enable_2d_grid: false,
		},
	};

	#[test]
	fn greedy_subset_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("+++>><<++>--->+++<><><><><<<<<+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "+++++>--->+++<<<<<+++");
	}

	#[test]
	fn greedy_program_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("<><><>++<+[--++>>+<<-]");
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "++<+[->>+<<]");
	}

	#[test]
	fn greedy_program_equivalence_test_1() {
		let v = BrainfuckProgram::from_str(
			"+++++++++>>+++>---->>>++++--<--++<<hello<++++[-<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "+++++++++>>+++++++>---->>>++<<<<[>++<]");
	}

	#[test]
	fn greedy_program_equivalence_test_2() {
		let v = BrainfuckProgram::from_str(">><.");
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, ">.");
	}

	#[test]
	fn greedy_subset_equivalence_test_1() {
		let v = BrainfuckProgram::from_str("+++<+++>[-]+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "[-]+++<+++>");
	}

	#[test]
	fn greedy_subset_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("+++<+++>[-]+++[-]<[-]--+>-"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "[-]-<[-]->");
	}

	#[test]
	fn greedy_program_equivalence_test_3() {
		let v = BrainfuckProgram::from_str(
			"+++++[-]+++++++++>>+++>---->>>++++--<--++<<hello<++++[[-]<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "[-]+++++++++>>+++++++>---->>>++<<<<[[-]+>++<]");
	}

	#[test]
	fn greedy_two_dimensional_subset_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "+++++^---^+++vvvvv+++");
	}

	#[test]
	fn greedy_two_dimensional_program_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("v^v^v^++v+[--++^^+vv-]");
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "++v+[-^^+vv]");
	}

	#[test]
	fn greedy_two_dimensional_program_equivalence_test_1() {
		let v = BrainfuckProgram::from_str(
			"+++++++++^^+++^----^^^++++--v--++vvhellov++++[-v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "+++++++++^^+++++++^----^^^++vvvv[^++v]");
	}

	#[test]
	fn greedy_two_dimensional_program_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("^^v.");
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "^.");
	}

	#[test]
	fn greedy_two_dimensional_subset_equivalence_test_1() {
		let v = BrainfuckProgram::from_str("+++v+++^[-]+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "[-]+++v+++^");
	}

	#[test]
	fn greedy_two_dimensional_subset_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("+++v+++^[-]+++[-]v[-]--+^-"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT.optimise_subset(v).to_string();
		assert_eq!(o, "[-]-v[-]-^");
	}

	#[test]
	fn greedy_two_dimensional_program_equivalence_test_3() {
		let v = BrainfuckProgram::from_str(
			"+++++[-]+++++++++^^+++^----^^^++++--v--++vvhellov++++[[-]v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT.optimise_bf_code(v).to_string();
		assert_eq!(o, "[-]+++++++++^^+++++++^----^^^++vvvv[[-]+^++v]");
	}

	#[test]
	#[ignore]
	fn exhaustive_subset_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("+++>><<++>--->+++<><><><><<<<<+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, ">--->+++<<+++++<<<+++");
	}

	#[test]
	#[ignore]
	fn exhaustive_program_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("<><><>++<+[--++>>+<<-]");
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "++<+[>>+<<-]");
	}

	#[test]
	#[ignore]
	fn exhaustive_program_equivalence_test_1() {
		let v = BrainfuckProgram::from_str(
			"+++++++++>>+++>---->>>++++--<--++<<hello<++++[-<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "+++++++++>>+++++++>>>>++<<<----<[>++<]");
	}

	#[test]
	#[ignore]
	fn exhaustive_program_equivalence_test_2() {
		let v = BrainfuckProgram::from_str(">><.");
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, ">.");
	}

	#[test]
	#[ignore]
	fn exhaustive_subset_equivalence_test_1() {
		let v = BrainfuckProgram::from_str("+++<+++>[-]+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, "[-]+++<+++>");
	}

	#[test]
	#[ignore]
	fn exhaustive_subset_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("+++<+++>[-]+++[-]<[-]--+>-"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, "[-]-<[-]->");
	}

	#[test]
	#[ignore]
	fn exhaustive_program_equivalence_test_3() {
		let v = BrainfuckProgram::from_str(
			"+++++[-]+++++++++>>+++>---->>>++++--<--++<<hello<++++[[-]<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "[-]+++++++++>>+++++++>---->>>++<<<<[[-]+>++<]");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_subset_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, "^^+++v---v+++++vvv+++");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_program_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("v^v^v^++v+[--++^^+vv-]");
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "++v+[^^+vv-]");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_program_equivalence_test_1() {
		let v = BrainfuckProgram::from_str(
			"+++++++++^^+++^----^^^++++--v--++vvhellov++++[-v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "+++++++++^^+++++++^----^^^++vvvv[^++v]");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_program_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("^^v.");
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "^.");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_subset_equivalence_test_1() {
		let v = BrainfuckProgram::from_str("+++v+++^[-]+++"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, "[-]+++v+++^");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_subset_equivalence_test_2() {
		let v = BrainfuckProgram::from_str("+++v+++^[-]+++[-]v[-]--+^-"); //(3) 0  0 [5] -3 3
		let o = CTX_OPT_EXHAUSTIVE.optimise_subset(v).to_string();
		assert_eq!(o, "[-]-v[-]-^");
	}

	#[test]
	#[ignore]
	fn exhaustive_two_dimensional_program_equivalence_test_3() {
		let v = BrainfuckProgram::from_str(
			"+++++[-]+++++++++^^+++^----^^^++++--v--++vvhellov++++[[-]v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf_code(v).to_string();
		assert_eq!(o, "[-]+++++++++^^^^^^++vvv----v+++++++[^++v[-]+]");
	}

	fn subset_edge_case_0() {
		let v = BrainfuckProgram::from_str(
			"-++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++",
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 127);
	}

	#[test]
	fn subset_edge_case_1() {
		let v = BrainfuckProgram::from_str(
			"++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++",
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 128);
	}

	#[test]
	fn subset_edge_case_2() {
		let v = BrainfuckProgram::from_str(
			"+--------------------------------------------------------------------------------------------------------------------------------"
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 127);
	}

	#[test]
	fn subset_edge_case_3() {
		let v = BrainfuckProgram::from_str(
			"--------------------------------------------------------------------------------------------------------------------------------"
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 128);
	}

	#[test]
	fn subset_edge_case_3a() {
		let v = BrainfuckProgram::from_str(
			"- --------------------------------------------------------------------------------------------------------------------------------"
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 127);
	}

	#[test]
	fn subset_edge_case_4() {
		let v = BrainfuckProgram::from_str(
			"[-]--------------------------------------------------------------------------------------------------------------------------------"
		);
		let o: String = CTX_OPT.optimise_subset(v).to_string();
		println!("{o}");
		assert_eq!(o.len(), 131);
	}
}
