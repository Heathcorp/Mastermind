use crate::{backend::bf::*, misc::MastermindContext};
use std::{collections::HashMap, num::Wrapping};

impl MastermindContext {
	pub fn optimise_bf(&self, ops: Vec<Opcode>) -> Vec<Opcode> {
		let mut output = Vec::new();

		// get stretch of characters to optimise (+-<>)
		let mut subset = Vec::new();
		for op in ops {
			match op {
				Opcode::Add | Opcode::Subtract | Opcode::Right | Opcode::Left | Opcode::Clear => {
					subset.push(op);
				}
				Opcode::OpenLoop | Opcode::CloseLoop | Opcode::Input | Opcode::Output => {
					// optimise subset and push
					let optimised_subset = optimise_bf_subset(subset);
					subset = vec![];

					// remove any redundant movement at the beginning
					//  (this shouldn't really be in the loop,
					//   but it's tested and works, and compiler code isn't performance critical)
					for subset_op in optimised_subset {
						if let (0, Opcode::Left | Opcode::Right) = (output.len(), subset_op) {
							continue;
						}
						output.push(subset_op);
					}
					output.push(op);
				}
			}
		}

		output
	}
}

fn optimise_bf_subset(run: Vec<Opcode>) -> Vec<Opcode> {
	#[derive(Clone)]
	enum Change {
		Add(Wrapping<i8>),
		Set(Wrapping<i8>),
	}
	let mut tape: HashMap<i32, Change> = HashMap::new();
	let mut head: i32 = 0;

	let mut i = 0;
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
		tape.insert(0, Change::Add(Wrapping(0i8)));
	}
	if !tape.contains_key(&head) {
		tape.insert(head, Change::Add(Wrapping(0i8)));
	}

	// This whole algorithm is probably really efficient and I reckon there's almost certainly a better way
	// It's also just really poorly done in general, I don't understand what everything does and I wrote the damned thing
	// TODO: refactor this properly
	// convert hashmap to array
	// start by making a negative and positive array
	let mut pos_arr = Vec::new();
	let mut neg_arr = Vec::new();
	for (cell, value) in tape.into_iter() {
		let i: usize;
		let arr: &mut Vec<Change>;
		if cell < 0 {
			i = (-(cell + 1)) as usize;
			arr = &mut neg_arr;
		} else {
			i = cell as usize;
			arr = &mut pos_arr;
		}

		if i >= arr.len() {
			arr.resize(i + 1, Change::Add(Wrapping(0i8)));
		}
		arr[i] = value;
	}
	let start_index = neg_arr.len();
	// now combine the two arrays
	let mut tape_arr: Vec<Change> = Vec::new();
	tape_arr.extend(neg_arr.into_iter().rev());
	tape_arr.extend(pos_arr.into_iter());

	if ((start_index) + 1) >= (tape_arr.len()) {
		tape_arr.resize(start_index + 1, Change::Add(Wrapping(0i8)));
	}
	let final_index = ((start_index as i32) + head) as usize;

	let mut output = Vec::new();

	// Also this following algorithm for zig-zagging around the tape is pretty poor as well, there has to be a nicer way of doing it

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
				let change = &tape_arr[cell];
				if let Change::Set(_) = change {
					output.push(Opcode::Clear);
				}
				let (Change::Add(v) | Change::Set(v)) = change;
				let v = v.0;

				for _ in 0..(v as i32).abs() {
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
				let change = &tape_arr[cell];
				if let Change::Set(_) = change {
					output.push(Opcode::Clear);
				}
				let (Change::Add(v) | Change::Set(v)) = change;
				let v = v.0;

				for _ in 0..(v as i32).abs() {
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
