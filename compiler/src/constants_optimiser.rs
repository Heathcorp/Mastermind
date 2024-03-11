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
	let abs_value = value.abs();

	// STAGE 0:
	// for efficiency's sake, calculate the cost of just adding the constant to the cell
	let solution_0 = {
		let mut ops = BrainfuckCodeBuilder::new();
		ops.head_pos = start_cell;
		ops.move_to_cell(target_cell);
		ops.add_to_current_cell(value);
		ops
	};
	// https://esolangs.org/wiki/Brainfuck_constants
	if abs_value < 15 {
		return solution_0;
	}

	// STAGE 1:
	// find best solution of form a * b + c
	let solution_1 = {
		let mut previous_best: Vec<(usize, usize, usize)> = vec![(0, 0, 0)];

		for i in 1..=(abs_value as usize) {
			let mut cheapest: (usize, usize, usize) = (1, i, 0);
			let mut j = 2;
			while j * j <= i {
				if i % j == 0 {
					let o = i / j;
					if (j + o) < (cheapest.0 + cheapest.1) {
						cheapest = (j, o, 0);
					}
				}

				j += 1;
			}

			for j in 0..i {
				let diff = i - j;
				let (a, b, c) = previous_best[j];
				if (a + b + c + diff) < (cheapest.0 + cheapest.1 + cheapest.2) {
					cheapest = (a, b, c + diff);
				}
			}

			previous_best.push(cheapest);
		}

		let (a, b, c) = previous_best.into_iter().last().unwrap();
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

	if solution_1.len() < solution_0.len() {
		solution_1
	} else {
		solution_0
	}
}
