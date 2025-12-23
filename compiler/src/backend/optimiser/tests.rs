#![cfg(test)]

use crate::{
	backend::{bf::*, bf2d::*, common::BrainfuckProgram},
	misc::{MastermindConfig, MastermindContext},
};

const CTX_OPT: MastermindContext = MastermindContext {
	config: MastermindConfig {
		optimise_generated_code: true,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		optimise_unreachable_loops: false,

		// optimise_variable_usage: false,

		// optimise_memory_allocation: false,
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

		// optimise_variable_usage: false,

		// optimise_memory_allocation: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 0,
		enable_2d_grid: false,
	},
};

// TODO:
// fn _characteristic_test()

#[test]
fn standard_0() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++>><<++>--->+++<><><><><<<<<+++[>>>]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), "+++++>--->+++<<<<<+++[>>>]".len());
}

#[test]
fn standard_1() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("<><><>++<+[--++>>+<<-]");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o, "++<+[->>+<<]");
}

#[test]
fn standard_2() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
		"+++++++++>>+++>---->>>++++--<--++<<hello<++++[-<+>>++<+<->]++--->+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o, "+++++++++>>+++++++>---->>>++<<<<[>++<]");
}

#[test]
fn standard_3() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(">><.");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o, ">.");
}

#[test]
fn standard_4() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++<+++>[-]+++[>.<+]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), "[-]+++<+++>[>.<+]".len());
}

#[test]
fn standard_5() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++<+++>[-]+++[-]<[-]--+>-[>,]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), "[-]-<[-]->[>,]".len());
}

#[test]
fn standard_6() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
		"+++++[-]+++++++++>>+++>---->>>++++--<--++<<hello<++++[[-]<+>>++<+<->]++--->+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o, "[-]+++++++++>>+++++++>---->>>++<<<<[[-]+>++<]");
}

#[test]
fn greedy_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++[>>>>>>>]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "+++++^---^+++vvvvv+++[>>>>>>>]");
}

#[test]
fn greedy_2d_1() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("v^v^v^++v+[--++^^+vv-]");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "++v+[-^^+vv]");
}

#[test]
fn greedy_2d_2() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
		"+++++++++^^+++^----^^^++++--v--++vvhellov++++[-v+^^++v+v-^]++---^+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "+++++++++^^+++++++^----^^^++vvvv[^++v]");
}

#[test]
fn greedy_2d_3() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("^^v.");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "^.");
}

#[test]
fn greedy_2d_4() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++,");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "[-]+++v+++^,");
}

#[test]
fn greedy_2d_5() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++[-]v[-]--+^-,,,,...");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "[-]-v[-]-^,,,,...");
}

#[test]
fn greedy_2d_6() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
		"+++++[-]+++++++++^^+++^----^^^++++--v--++vvhellov++++[[-]v+^^++v+v-^]++---^+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o, "[-]+++++++++^^+++++++^----^^^++vvvv[[-]+^++v]");
}

#[test]
fn exhaustive_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++,");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "^^+++v---v+++++vvv+++,".len());
}

#[test]
fn exhaustive_2d_1() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("v^v^v^++v+[--++^^+vv-]");
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "++v+[^^+vv-]".len());
}

#[test]
fn exhaustive_2d_2() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
		"+++++++++^^+++^----^^^++++--v--++vvhellov++++[-v+^^++v+v-^]++---^+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "+++++++++^^+++++++^----^^^++vvvv[^++v]".len());
}

#[test]
fn exhaustive_2d_3() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("^^v.");
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o, "^.");
}

#[test]
fn exhaustive_2d_4() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++.");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "[-]+++v+++^.".len());
}

#[test]
fn exhaustive_2d_5() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++[-]v[-]--+^-.");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "[-]-v[-]-^.".len());
}

#[test]
fn exhaustive_2d_6() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
		"+++++[-]+++++++++^^+++^----^^^++++--v--++vvhellov++++[[-]v+^^++v+v-^]++---^+",
	);
	// [9] 0 (7) -4 0 0 2
	// [(0)] 2
	// -1 1
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(
		o.len(),
		"[-]+++++++++^^^^^^++vvv----v+++++++[^++v[-]+]".len()
	);
}

#[test]
fn wrapping_0() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"-++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.",
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_1() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++,",
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}

#[test]
fn wrapping_2() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"+--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_3() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"--------------------------------------------------------------------------------------------------------------------------------,"
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}

#[test]
fn wrapping_3a() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"- --------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_4() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
			"[-]--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), 131 + 1);
}

#[test]
fn wrapping_0_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"-++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++,",
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_1_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.",
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}

#[test]
fn wrapping_2_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"+--------------------------------------------------------------------------------------------------------------------------------,"
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_3_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}

#[test]
fn wrapping_3a_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"- --------------------------------------------------------------------------------------------------------------------------------,"
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}

#[test]
fn wrapping_4_2d() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"[-]--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 131 + 1);
}
