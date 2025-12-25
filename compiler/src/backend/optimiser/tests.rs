#![cfg(test)]

use std::io::Cursor;

use crate::{
	backend::{bf::*, bf2d::*, common::BrainfuckProgram},
	brainfuck::{BrainfuckConfig, BrainfuckContext},
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

// TODO: implement this, would have to refactor the BVM
// fn get_tape_changes(code: &str) {
// let ctx = BrainfuckContext {
// 	config: BrainfuckConfig {
// 		enable_debug_symbols: false,
// 		enable_2d_grid: false,
// 	},
// };

// let input_bytes: Vec<u8> = vec![];
// let mut input_stream = Cursor::new(input_bytes);
// let mut output_stream = Cursor::new(vec![]);

// ctx.run(
// 	code.chars().collect(),
// 	&mut input_stream,
// 	&mut output_stream,
// 	Some(1000),
// )
// .unwrap();
// }

/// Count each type of opcode, for naive equivalence tests
fn tally_opcodes(input: &str) -> [usize; 11] {
	let mut t = [0; 11];
	for c in input.chars() {
		*match c {
			'+' => &mut t[0],
			'-' => &mut t[1],
			'>' => &mut t[2],
			'<' => &mut t[3],
			'[' => &mut t[4],
			']' => &mut t[5],
			'.' => &mut t[6],
			',' => &mut t[7],
			'^' => &mut t[8],
			'v' => &mut t[9],
			_ => &mut t[10],
		} += 1;
	}
	t
}

fn _characteristic_test(input: &str, expected: &str) {
	let ops: Vec<Opcode> = BrainfuckProgram::from_str(input);
	let optimised = CTX_OPT.optimise_bf(ops).to_string();
	println!("OPTIMISED ({}): {}", optimised.len(), optimised);
	println!("EXPECTED  ({}): {}", expected.len(), expected);
	assert_eq!(optimised.len(), expected.len());
	// TODO: implement actually running both codes, would require refactoring BVM
	assert_eq!(tally_opcodes(&optimised), tally_opcodes(&expected));
}
fn _characteristic_test_2d(ctx: MastermindContext, input: &str, expected: &str) {
	todo!();
}

#[test]
fn standard_0() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++>><<++>--->+++<><><><><<<<<+++[>>>]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf(v).to_string();
	let e = "+++<---<+++++<<<+++[>>>]";
	assert_eq!(o, e);
}
#[test]
fn standard_1() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("<><><>++<+[--++>>+<<-]");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "++<+[->>+<<]";
	assert_eq!(o, e);
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
	let e = "+++++++++>>+++++++>---->>>++<<<<[>++<]";
	assert_eq!(o, e);
}
#[test]
fn standard_3() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(".>><.");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = ".>.";
	assert_eq!(o, e);
}
#[test]
fn standard_4() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++<+++>[-]+++[>.<+]");
	let o = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), "+++>[-]+++[>.<+]".len());
}
#[test]
fn standard_5() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("+++<+++>[-]+++[-]<[-]--+>-[>,]");
	let o = CTX_OPT.optimise_bf(v).to_string();
	assert_eq!(o.len(), "[-]->[-]-[>,]".len());
}
#[test]
fn standard_6() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(
		"+++++[-]+++++++++>>+++>---->>>++++--<--++<<hello<++++[[-]<+>>++<+<->]++--->+",
	);
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "[-]+++++++++>>+++++++>---->>>++<<<<[[-]+>++<]";
	assert_eq!(o, e);
}

#[test]
fn greedy_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++^^vv++^---^+++v^v^v^v^vvvvv+++[>>>>>>>]");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "+++++^---^+++vvvvv+++[>>>>>>>]";
	assert_eq!(o, e);
}
#[test]
fn greedy_2d_1() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("v^v^v^++v+[--++^^+vv-]");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "++v+[-^^+vv]";
	assert_eq!(o, e);
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
	let e = "+++++++++^^+++++++^----^^^++vvvv[^++v]";
	assert_eq!(o, e);
}
#[test]
fn greedy_2d_3() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(",^^v.");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	let e = ",^.";
	assert_eq!(o, e);
}
#[test]
fn greedy_2d_4() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++,");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "[-]+++v+++^,";
	assert_eq!(o, e);
}
#[test]
fn greedy_2d_5() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("+++v+++^[-]+++[-]v[-]--+^-,,,,...");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "[-]-v[-]-^,,,,...";
	assert_eq!(o, e);
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
	let e = "[-]+++++++++^^+++++++^----^^^++vvvv[[-]+^++v]";
	assert_eq!(o, e);
}

#[test]
fn exhaustive_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(",+++^^vv++^---^+++v^v^v^v^vvvvv+++,");
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), ",^^+++v---v+++++vvv+++,".len());
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
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "+++++++++^^+++++++^----^^^++vvvv[^++v]".len());
}
#[test]
fn exhaustive_2d_3() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(".^^v.");
	let o: String = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	let e = ".^.";
	assert_eq!(o, e);
}
#[test]
fn exhaustive_2d_4() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(",+++v+++^[-]+++.");
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), ",[-]+++v+++^.".len());
}
#[test]
fn exhaustive_2d_5() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(",+++v+++^[-]+++[-]v[-]--+^-.");
	//(3) 0  0 [5] -3 3
	let o = CTX_OPT_EXHAUSTIVE.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), ",[-]-v[-]-^.".len());
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
fn wrapping_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"-++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++,",
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}
#[test]
fn wrapping_2d_1() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.",
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}
#[test]
fn wrapping_2d_2() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"+--------------------------------------------------------------------------------------------------------------------------------,"
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}
#[test]
fn wrapping_2d_3() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 128 + 1);
}
#[test]
fn wrapping_2d_3a() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"- --------------------------------------------------------------------------------------------------------------------------------,"
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 127 + 1);
}
#[test]
fn wrapping_2d_4() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str(
			"[-]--------------------------------------------------------------------------------------------------------------------------------."
		);
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), 131 + 1);
}

#[test]
fn offset_toplevel_0() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("++>>>++++<<<-->>>.");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "++++.";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_0a() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("[++>>>++++<<<-->>>.]");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "[>>>++++.]";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_1() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str(">>++>-+<++<[++>>>++++<<<-->>>.]<<");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "++++<[>>>++++.]";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_2() {
	let v: Vec<Opcode> = BrainfuckProgram::from_str("[++>>>++++<<<-->>>.]>>>+++<<");
	let o: String = CTX_OPT.optimise_bf(v).to_string();
	let e = "[>>>++++.]";
	assert_eq!(o, e);
}

#[test]
fn offset_toplevel_2d_0() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("++>>>vvv++++<<^^^<--vv>.");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "++++<<^.";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_2d_0a() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("[++v>>vv>++++<<^^<^--vv>.]");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "[>>>vvv++++<<^.]";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_2d_1() {
	let v: Vec<Opcode2D> =
		BrainfuckProgram::from_str(">vv>++>^-+v<++<[++>vv>>++++<^^<<-->^^^>>.]<<");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	let e = "++++<[>>>vv++++^^^^^.]";
	assert_eq!(o, e);
}
#[test]
fn offset_toplevel_2d_2() {
	let v: Vec<Opcode2D> = BrainfuckProgram::from_str("[++>v>>++++<^<<-->v>>.]^^>>>+++<vvv<");
	let o: String = CTX_OPT.optimise_bf2d(v).to_string();
	assert_eq!(o.len(), "[v>>>++++.]".len());
}
