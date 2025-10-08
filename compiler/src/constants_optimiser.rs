// TODO: make unit tests for this
use crate::builder::{BrainfuckCodeBuilder, Opcode, TapeCell};

// basically, most ascii characters are large numbers, which are more efficient to calculate with multiplication than with a bunch of + or -
// an optimising brainfuck runtime will prefer a long string of +++++ or ----- however the goal of mastermind is to be used for code golf, which is not about speed
// I made a mock-up version of this algorithm in python in another repo:
// https://github.com/Heathcorp/algorithms

// 7 * 4 : {>}(tricky)+++++++[<++++>-]<
// 5 * 5 * 7 : +++++[>+++++<-]>[<+++++++>-]<
pub fn calculate_optimal_addition(
	value: i8,
	start_cell: TapeCell,
	target_cell: TapeCell,
	temp_cell: TapeCell,
) -> BrainfuckCodeBuilder {
	// can't abs() i8 directly because there is no +128i8, so abs(-128i8) crashes
	let abs_value = (value as i32).abs();

	// STAGE 0:
	// for efficiency's sake, calculate the cost of just adding the constant to the cell
	let naive_solution = {
		let mut ops = BrainfuckCodeBuilder::new();
		ops.head_pos = start_cell;
		ops.move_to_cell(target_cell);
		ops.add_to_current_cell(value);
		ops
	};

	// below 15 is pointless according to: https://esolangs.org/wiki/Brainfuck_constants
	if abs_value < 15 {
		return naive_solution;
	}

	// STAGE 1:
	// find best solution of form a * b + c
	let solution_1 = {
		// dynamic programming algorithm, although not generalised
		// initialise so element 0 is also valid
		let mut best_combinations: Vec<(usize, usize, usize)> = vec![(0, 0, 0)];

		// Loop until the target number,
		//  inner loop finds any (a, b)s where a * b = the iteration number i.
		// Second inner loop finds c terms so that for each main iteration:
		//  there is some (a, b, c) where a * b + c = i.
		// This finds the "cheapest" meaning the (a, b, c) where a + b + c is lowest.
		for i in 1..=(abs_value as usize) {
			let mut current_best: (usize, usize, usize) = (1, i, 0);
			let mut j = 2;
			while j * j <= i {
				if i % j == 0 {
					let o = i / j;
					if (j + o) < (current_best.0 + current_best.1) {
						current_best = (j, o, 0);
					}
				}

				j += 1;
			}

			for j in 0..i {
				let diff = i - j;
				let (a, b, c) = best_combinations[j];
				if (a + b + c + diff) < (current_best.0 + current_best.1 + current_best.2) {
					current_best = (a, b, c + diff);
				}
			}

			best_combinations.push(current_best);
		}

		assert_eq!(best_combinations.len(), (abs_value as usize) + 1);
		let (a, b, c) = best_combinations.into_iter().last().unwrap();
		let mut ops = BrainfuckCodeBuilder::new();
		ops.head_pos = start_cell;

		ops.move_to_cell(temp_cell);
		ops.add_to_current_cell(a as i8);
		ops.push(Opcode::OpenLoop);
		ops.add_to_current_cell(-1);
		ops.move_to_cell(target_cell);
		if value < 0 {
			ops.add_to_current_cell(-(b as i8));
		} else {
			ops.add_to_current_cell(b as i8);
		}
		ops.move_to_cell(temp_cell);
		ops.push(Opcode::CloseLoop);
		ops.move_to_cell(target_cell);
		if value < 0 {
			ops.add_to_current_cell(-(c as i8));
		} else {
			ops.add_to_current_cell(c as i8);
		}

		ops
	};

	// STAGE 2:
	// find best solution of form (a * b + c) * d + e
	// TODO:

	// compare best solutions

	if solution_1.len() < naive_solution.len() {
		solution_1
	} else {
		naive_solution
	}
}
