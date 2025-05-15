use std::{collections::HashMap, num::Wrapping};
use std::alloc::dealloc;
use crate::builder::Opcode;

// post-compilation optimisations

// simple naive brainfuck optimisations
// TODO: factor in [-] into optimisations (doing)

pub fn optimise(program: Vec<Opcode>, exhaustive: bool) -> Vec<Opcode> {
	let mut output = Vec::new();

	// get stretch of characters to optimise (+-<>)
	let mut i = 0;
	let mut subset = Vec::new();
	while i < program.len() {
		let op = program[i];
		match op {
			Opcode::Add | Opcode::Subtract | Opcode::Right | Opcode::Left | Opcode::Clear | Opcode::Up | Opcode::Down => {
				subset.push(op);
			}
			Opcode::OpenLoop | Opcode::CloseLoop | Opcode::Input | Opcode::Output => {
				// optimise subset and push
				let optimised_subset = optimise_subset(subset, exhaustive);
				output.extend(optimised_subset);

				subset = Vec::new();
				output.push(op);
			}
		}
		i += 1;
	}

	output
}

fn move_position(mut program: Vec<Opcode>, old_position: &(i32, i32), new_position: &(i32, i32)) -> Vec<Opcode> {
	if old_position != new_position {
		if old_position.0 < new_position.0 {
			for _ in 0..(new_position.0 - old_position.0) {
				program.push(Opcode::Right);
			}
		} else {
			for _ in 0..(old_position.0 - new_position.0) {
				program.push(Opcode::Left);
			}
		}
		if old_position.1 < new_position.1 {
			for _ in 0..(new_position.1 - old_position.1) {
				program.push(Opcode::Up);
			}
		} else {
			for _ in 0..(old_position.1 - new_position.1) {
				program.push(Opcode::Down);
			}
		}
	}
	program
}

fn optimise_subset(run: Vec<Opcode>, exhaustive: bool) -> Vec<Opcode> {
	#[derive(Clone)]
	enum Change {
		Add(Wrapping<i8>),
		Set(Wrapping<i8>),
	}
	let mut tape: HashMap<(i32, i32), Change> = HashMap::new();
	let start = (0, 0);
	let mut head  = (0, 0);
	let mut i = 0;
	//Generate a map of cells we change and how we plan to change them
	while i < run.len() {
		let op = run[i];
		match op {
			Opcode::Clear => {
				tape.insert(head, Change::Set(Wrapping(0i8)));
			}
			Opcode::Subtract | Opcode::Add => {
				let mut change = tape.remove(&head).unwrap_or(Change::Add(Wrapping(0i8)));

				let (Change::Add(val) | Change::Set(val)) = &mut change;
				*val += match op {
					Opcode::Add => 1,
					Opcode::Subtract => -1,
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
			Opcode::Right => {
				head.0 += 1;
			}
			Opcode::Left => {
				head.0 -= 1;
			}
			Opcode::Up => {
				head.1 += 1;
			}
			Opcode::Down => {
				head.1 -= 1;
			}
			_ => (),
		}
		i += 1;
	}
	//Lets start with greedy approach find the nearest cell and move to it
	let mut position = start;
	let mut output = Vec::new();
	if exhaustive {

	}
	else {
		//For the number of cells navigate to the nearest cell
		for _ in 0..tape.len() {
			if !tape.is_empty() {
				let mut min_distance = i32::MAX;
				let mut next_position= (0,0);
				for (cell, value) in tape.iter() {
					if (cell.0 - position.0).abs() + (cell.1 - position.1).abs()  < min_distance {
						min_distance = (cell.0 - position.0).abs() + (cell.1 - position.1).abs();
						next_position = *cell;
					}
				}
				// Move to next position
				output = move_position(output, &position, &next_position);
				position = next_position;
				//Now Update the output with correct opcodes
				let change = tape.remove(&next_position).unwrap();
				if let Change::Set(_) = change {
					output.push(Opcode::Clear);
				}
				let (Change::Add(v) | Change::Set(v)) = change;
				let v = v.0;
				if v > 0 {
					for _ in 0..v {
						output.push(Opcode::Add);
					}
				} else if v < 0 {
					for _ in 0..(-v) {
						output.push(Opcode::Subtract);
					}
				}
			}
		}
		output = move_position(output, &position, &head);
	}
	output
}

#[cfg(test)]
mod tests {
	use crate::builder::BrainfuckOpcodes;

	use super::*;

	#[test]
	fn subset_equivalence_test_0() {
		let v = BrainfuckOpcodes::from_str("+++>><<++>--->+++<><><><><<<<<+++"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "+++++>--->+++<<<<<+++");
	}

	#[test]
	fn program_equivalence_test_0() {
		let v = BrainfuckOpcodes::from_str("<><><>++<+[--++>>+<<-]");
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "++<+[->>+<<]");
	}

	#[test]
	fn program_equivalence_test_1() {
		let v = BrainfuckOpcodes::from_str(
			"+++++++++>>+++>---->>>++++--<--++<<hello<++++[-<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "+++++++++>>+++++++>---->>>++<<<<[>++<]");
	}

	#[test]
	fn program_equivalence_test_2() {
		let v = BrainfuckOpcodes::from_str(">><.");
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, ">.");
	}

	#[test]
	fn subset_equivalence_test_1() {
		let v = BrainfuckOpcodes::from_str("+++<+++>[-]+++"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "[-]+++<+++>");
	}

	#[test]
	fn subset_equivalence_test_2() {
		let v = BrainfuckOpcodes::from_str("+++<+++>[-]+++[-]<[-]--+>-"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "[-]-<[-]->");
	}

	#[test]
	fn program_equivalence_test_3() {
		let v = BrainfuckOpcodes::from_str(
			"+++++[-]+++++++++>>+++>---->>>++++--<--++<<hello<++++[[-]<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "[-]+++++++++>>+++++++>---->>>++<<<<[[-]+>++<]");
	}

	#[test]
	fn two_dimensional_subset_equivalence_test_0() {
		let v = BrainfuckOpcodes::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "+++++^---^+++vvvvv+++");
	}

	#[test]
		fn two_dimensional_program_equivalence_test_0() {
		let v = BrainfuckOpcodes::from_str("v^v^v^++v+[--++^^+vv-]");
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "++v+[-^^+vv]");
	}

	#[test]
	fn two_dimensional_program_equivalence_test_1() {
		let v = BrainfuckOpcodes::from_str(
		"+++++++++^^+++^----^^^++++--v--++vvhellov++++[-v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "+++++++++^^+++++++^----^^^++vvvv[^++v]");
	}

	#[test]
	fn two_dimensional_program_equivalence_test_2() {
		let v = BrainfuckOpcodes::from_str("^^v.");
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "^.");
	}

	#[test]
	fn two_dimensional_subset_equivalence_test_1() {
		let v = BrainfuckOpcodes::from_str("+++v+++^[-]+++"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "[-]+++v+++^");
	}

	#[test]
	fn two_dimensional_subset_equivalence_test_2() {
		let v = BrainfuckOpcodes::from_str("+++v+++^[-]+++[-]v[-]--+^-"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v, false).to_string();
		assert_eq!(o, "[-]-v[-]-^");
	}

	#[test]
	fn two_dimensional_program_equivalence_test_3() {
		let v = BrainfuckOpcodes::from_str(
		"+++++[-]+++++++++^^+++^----^^^++++--v--++vvhellov++++[[-]v+^^++v+v-^]++---^+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = optimise(v, false).to_string();
		assert_eq!(o, "[-]+++++++++^^+++++++^----^^^++vvvv[[-]+^++v]");
	}
}
