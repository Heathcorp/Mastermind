use crate::builder::{BrainfuckProgram, Opcode, TapeCell};

// basically, most ascii characters are large numbers, which are more efficient to calculate with multiplication than with a bunch of + or -
// an optimising brainfuck runtime will prefer a long string of +++++ or ----- however the goal of mastermind is to be used for code golf, which is not about speed
// I made a mock-up version of this algorithm in python in another repo:
// https://github.com/Heathcorp/algorithms
pub fn calculate_optimal_addition(
	value: i8,
	head_pos: &mut TapeCell,
	target_cell: TapeCell,
	temp_cell: TapeCell,
) -> Vec<Opcode> {
	let distance = target_cell.abs_diff(temp_cell);
	let abs_val = value.abs() as u8;
	// the extra brainfuck code required to move between the two cells to multiply
	// 7 * 4 : {>}(tricky)+++++++[<++++>-]<
	// 5 * 5 * 7 : +++++[>+++++<-]>[<+++++++>-]<
	// hmm, okay for starters lets assume nothing of the sort, this extra move is actually a little tricky to deal with
	// I wonder if I can do different things based on where the start cell is? This is really tricky actually because there are post-optimisations that nullify this
	// also this kind of nullifies the dynamic programming approach to the problem a little ?

	// it would be cool to include fancy modulo division ones like this:
	// ----[>+<----]>-- = 61 // not sure how the algorithm for this would work
	let mult_cost = 3 + 3 * distance;

	let mut computed_cheapest: Vec<(usize, Value)> = vec![(0, Value::Constant(0))];
	for i in 1..=(abs_val as usize) {
		let mut best = (i, Value::Constant(i));

		// check the min summed two factors
		{
			let mut j = 2;
			// loop until j goes over the square root of i, because we will have already checked all of the pairs if we reach that point
			while j * j <= i {
				if i % j == 0 {
					let o = i / j;
					// i = j * o
					let jo_cost = computed_cheapest[j].0 + o + mult_cost;
					if jo_cost < best.0 {
						best = (jo_cost, Value::NumTimesConstant(j, o))
					}
					// we only support using 2 cells, which is impossible if we need to multiply 2 products of 2 with constants added
					// so we check either constructing j first or o first, with one of them being a direct one-to-one with bf +++s, not sure how to articulate this
					let oj_cost = computed_cheapest[o].0 + j + mult_cost;
					if oj_cost < best.0 {
						best = (oj_cost, Value::NumTimesConstant(o, j))
					}
				}
				j += 1;
			}
		}

		// now check existing values with constants added
		// not sure if we can cut down the iterations here?
		for j in 1..i {
			let diff = i - j;
			let sum_cost = computed_cheapest[j].0 + diff;
			if sum_cost < best.0 {
				best = (sum_cost, Value::NumPlusConstant(j, diff));
			}
		}

		computed_cheapest.push(best);
	}

	// now we have the best multiplication/sum path to the number we want
	// we need to turn that into brainfuck

	let mut ops: Vec<Opcode> = Vec::new();
	{
		// tiny builder context using the builder.rs APIs
		// this head_pos shit should really be part of the opcode list trait, need a custom struct

		let math_ops = parse_operations(&computed_cheapest, &computed_cheapest[abs_val as usize].1);
		let num_multiplications = math_ops.iter().fold(0, |count, math_op| match math_op {
			SequentialOperation::Times(_) => count + 1,
			SequentialOperation::Plus(_) => count,
		});

		// weird logic to save duplicating code
		// essentially switch back and forth between cells using a diff value
		let (mut a_cell, mut b_cell) = match num_multiplications {
			n if n % 2 == 1 => (temp_cell, target_cell),
			_ => (target_cell, temp_cell),
		};

		ops.move_to_cell(head_pos, a_cell);
		let mut mult_count = 0;
		for math_op in math_ops {
			match math_op {
				SequentialOperation::Times(multiplier) => {
					// open a loop on the a-cell, for each iteration of the loop add multiplier to the b-cell
					ops.push(Opcode::OpenLoop);
					ops.add_to_current_cell(-1);
					ops.move_to_cell(head_pos, b_cell);
					// TODO: is this casting correct?
					// handle negative numbers, the last multiplication should be negative
					mult_count += 1;
					if mult_count < num_multiplications || value > 0 {
						ops.add_to_current_cell(multiplier as u8 as i8);
					} else {
						ops.add_to_current_cell(-(multiplier as u8 as i8));
					}
					ops.move_to_cell(head_pos, a_cell);
					ops.push(Opcode::CloseLoop);
					ops.move_to_cell(head_pos, b_cell);
					// now switch cells for the next operation
					(a_cell, b_cell) = (b_cell, a_cell);
				}
				SequentialOperation::Plus(summand) => {
					// add to the a-cell, don't switch
					// also handle negative constants (127 < n < 256)
					if mult_count < num_multiplications || value > 0 {
						ops.add_to_current_cell(summand as u8 as i8);
					} else {
						ops.add_to_current_cell(-(summand as u8 as i8));
					}
				}
			}
		}
		assert_eq!(
			a_cell, target_cell,
			"Logic error occurred trying to optimise constant {value}. \
This should never occur."
		);
	}

	ops
}

#[derive(Debug)]
enum Value {
	Constant(usize),
	NumPlusConstant(usize, usize),
	NumTimesConstant(usize, usize),
}

enum SequentialOperation {
	Times(usize),
	Plus(usize),
}

// have to reverse the operations as found by the dynamic programming algorithm
// there might be a better way to do this all in one go but whatever
fn parse_operations(
	computed_cheapest: &[(usize, Value)],
	value: &Value,
) -> Vec<SequentialOperation> {
	match value {
		// base case
		Value::Constant(num) => vec![SequentialOperation::Plus(*num)],
		Value::NumPlusConstant(num, _) | Value::NumTimesConstant(num, _) => {
			let mut ops = parse_operations(computed_cheapest, &computed_cheapest[*num].1);
			match value {
				Value::NumPlusConstant(_, v) => ops.push(SequentialOperation::Plus(*v)),
				Value::NumTimesConstant(_, v) => ops.push(SequentialOperation::Times(*v)),
				_ => (),
			}
			ops
		}
	}
}
