use std::{collections::HashMap, num::Wrapping};

use crate::builder::Opcode;

// post-compilation optimisations

// simple naive brainfuck optimisations
// TODO: factor in [-] into optimisations (doing)

pub fn optimise(program: Vec<Opcode>) -> Vec<Opcode> {
	let mut output = Vec::new();

	// get stretch of characters to optimise (+-<>)
	let mut i = 0;
	let mut subset = Vec::new();
	while i < program.len() {
		let op = program[i];
		match op {
			Opcode::Add | Opcode::Subtract | Opcode::Right | Opcode::Left => {
				subset.push(op);
			}
			Opcode::OpenLoop | Opcode::CloseLoop | Opcode::Input | Opcode::Output => {
				// optimise subset and push
				let optimised_subset = optimise_subset(subset);
				output.extend(optimised_subset);

				subset = Vec::new();
				output.push(op);
			}
			_ => (),
		}
		i += 1;
	}

	output
}

fn optimise_subset(run: Vec<Opcode>) -> Vec<Opcode> {
	let mut tape: HashMap<i32, Wrapping<i8>> = HashMap::new();
	let mut head: i32 = 0;

	let mut i = 0;
	while i < run.len() {
		let op = run[i];
		match op {
			Opcode::Subtract | Opcode::Add => {
				if !tape.contains_key(&head) {
					tape.insert(head, Wrapping(0i8));
				}
				let cell = tape.get_mut(&head).unwrap();
				*cell += match op {
					Opcode::Add => 1,
					Opcode::Subtract => -1,
					_ => 0,
				};
				if cell.0 == 0 {
					tape.remove(&head);
				}
			}
			Opcode::Right => {
				head += 1;
			}
			Opcode::Left => {
				head -= 1;
			}
			_ => (),
		}
		i += 1;
	}
	// always have a start and end cell
	if !tape.contains_key(&0) {
		tape.insert(0, Wrapping(0i8));
	}
	if !tape.contains_key(&head) {
		tape.insert(head, Wrapping(0i8));
	}

	// convert hashmap to array
	// start by making a negative and positive array
	let mut pos_arr = Vec::new();
	let mut neg_arr = Vec::new();
	for (cell, value) in tape.into_iter() {
		let i: usize;
		let arr: &mut Vec<Wrapping<i8>>;
		if cell < 0 {
			i = (-(cell + 1)) as usize;
			arr = &mut neg_arr;
		} else {
			i = cell as usize;
			arr = &mut pos_arr;
		}

		if i >= arr.len() {
			arr.resize(i + 1, Wrapping(0i8));
		}
		arr[i] = value;
	}
	// now combine the two arrays
	let mut tape_arr: Vec<Wrapping<i8>> = Vec::new();
	tape_arr.extend(neg_arr.iter().rev());
	tape_arr.extend(pos_arr.iter());

	let start_index = neg_arr.len();
	if ((start_index) + 1) >= (tape_arr.len()) {
		tape_arr.resize(start_index + 1, Wrapping(0i8));
	}
	let final_index = ((start_index as i32) + head) as usize;

	let mut output = Vec::new();

	// if final cell is to the right of the start cell then we need to go to the left first, and vice-versa
	// 1. go to the furthest point on the tape (opposite of direction to final cell)
	// 2. go from that end of the tape to the opposite end of the tape
	// 3. return to the final cell
	let mut idx = start_index;
	let d2 = final_index >= start_index;
	let d1 = !d2;
	let d3 = !d2;

	//1
	match d1 {
		true => {
			for _ in start_index..(tape_arr.len() - 1) {
				output.push(Opcode::Right);
				idx += 1;
			}
		}
		false => {
			for _ in (1..=start_index).rev() {
				output.push(Opcode::Left);
				idx -= 1;
			}
		}
	}

	//2
	match d2 {
		true => {
			for cell in idx..tape_arr.len() {
				let v = tape_arr[cell].0;
				for _ in 0..v.abs() {
					output.push(match v > 0 {
						true => Opcode::Add,
						false => Opcode::Subtract,
					});
				}

				if cell < (tape_arr.len() - 1) {
					output.push(Opcode::Right);
					idx += 1;
				}
			}
		}
		false => {
			for cell in (0..=idx).rev() {
				let v = tape_arr[cell].0;
				for _ in 0..v.abs() {
					output.push(match v > 0 {
						true => Opcode::Add,
						false => Opcode::Subtract,
					});
				}

				if cell > 0 {
					output.push(Opcode::Left);
					idx -= 1;
				}
			}
		}
	}

	//3
	match d3 {
		true => {
			for _ in idx..final_index {
				output.push(Opcode::Right);
				idx += 1;
			}
		}
		false => {
			for _ in final_index..idx {
				output.push(Opcode::Left);
				idx -= 1;
			}
		}
	}

	output
}

#[cfg(test)]
mod tests {
	use crate::builder::BrainfuckProgram;

	use super::*;

	#[test]
	fn subset_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("+++>><<++>--->+++<><><><><<<<<+++"); //(3) 0  0 [5] -3 3
		let o = optimise_subset(v).to_string();
		assert_eq!(o, ">>+++<---<+++++<<<+++");
	}

	#[test]
	fn program_equivalence_test_0() {
		let v = BrainfuckProgram::from_str("<><><>++<+[--++>>+<<-]");
		let o: String = optimise(v).to_string();
		assert_eq!(o, "++<+[->>+<<]");
	}

	#[test]
	fn program_equivalence_test_1() {
		let v = BrainfuckProgram::from_str(
			"+++++++++>>+++>---->>>++++--<--++<<hello<++++[-<+>>++<+<->]++--->+",
		); // [9] 0 (7) -4 0 0 2 // [(0)] 2 // -1 1
		let o: String = optimise(v).to_string();
		assert_eq!(o, "+++++++++>>+++++++>---->>>++<<<<[>++<]");
	}

	#[test]
	fn program_equivalence_test_2() {
		let v = BrainfuckProgram::from_str(">><.");
		let o: String = optimise(v).to_string();
		assert_eq!(o, ">.");
	}
}
