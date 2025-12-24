#![allow(dead_code)]

// black box testing
#[cfg(test)]
pub mod black_box_tests {
	use crate::{
		backend::{
			bf::{Opcode, TapeCell},
			bf2d::{Opcode2D, TapeCell2D},
			common::{
				BrainfuckBuilder, BrainfuckBuilderData, BrainfuckProgram, CellAllocator,
				CellAllocatorData, OpcodeVariant, TapeCellVariant,
			},
		},
		brainfuck::{bvm_tests::run_code, BrainfuckConfig},
		misc::{MastermindConfig, MastermindContext},
		parser::parser::parse_program,
		preprocessor::strip_comments,
	};
	// TODO: run test suite with different optimisations turned on
	const OPT_NONE: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		// optimise_variable_usage: false,
		// optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 0,
		enable_2d_grid: false,
	};

	const OPT_ALL: MastermindConfig = MastermindConfig {
		optimise_generated_code: true,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: true,
		// optimise_variable_usage: true,
		// optimise_memory_allocation: true,
		optimise_unreachable_loops: true,
		optimise_constants: true,
		optimise_empty_blocks: true,
		memory_allocation_method: 0,
		enable_2d_grid: false,
	};

	const OPT_NONE_2D_TILES: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		// optimise_variable_usage: false,
		// optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 3,
		enable_2d_grid: true,
	};

	const OPT_NONE_2D_SPIRAL: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		// optimise_variable_usage: false,
		// optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 2,
		enable_2d_grid: true,
	};

	const OPT_NONE_2D_ZIG_ZAG: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		// optimise_variable_usage: false,
		// optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 1,
		enable_2d_grid: true,
	};

	const BVM_CONFIG_1D: BrainfuckConfig = BrainfuckConfig {
		enable_debug_symbols: false,
		enable_2d_grid: false,
	};

	const BVM_CONFIG_2D: BrainfuckConfig = BrainfuckConfig {
		enable_debug_symbols: false,
		enable_2d_grid: true,
	};

	const TESTING_BVM_MAX_STEPS: usize = 100_000_000;

	fn compile_and_run<'a, TC: 'static + TapeCellVariant, OC: 'static + OpcodeVariant>(
		raw_program: &str,
		input: &str,
	) -> Result<String, String>
	where
		BrainfuckBuilderData<TC, OC>: BrainfuckBuilder<TC, OC>,
		CellAllocatorData<TC>: CellAllocator<TC>,
		Vec<OC>: BrainfuckProgram,
	{
		let ctx = MastermindContext { config: OPT_NONE };
		let stripped_program = strip_comments(raw_program);
		let clauses = parse_program::<TC, OC>(&stripped_program)?;
		let instructions = ctx.create_ir_scope(&clauses, None)?.build_ir(false);
		let bf_program = ctx.ir_to_bf(instructions, None)?;
		let bfs = bf_program.to_string();

		// run generated brainfuck with input
		run_code(BVM_CONFIG_1D, &bfs, input, Some(TESTING_BVM_MAX_STEPS))
	}

	fn compile_program<'a, TC: 'static + TapeCellVariant, OC: 'static + OpcodeVariant>(
		raw_program: &str,
		config: Option<MastermindConfig>,
	) -> Result<String, String>
	where
		BrainfuckBuilderData<TC, OC>: BrainfuckBuilder<TC, OC>,
		CellAllocatorData<TC>: CellAllocator<TC>,
		Vec<OC>: BrainfuckProgram,
	{
		let ctx = MastermindContext {
			config: config.unwrap_or(OPT_NONE),
		};
		let stripped_program = strip_comments(raw_program);
		let clauses = parse_program::<TC, OC>(&stripped_program)?;
		let instructions = ctx.create_ir_scope(&clauses, None)?.build_ir(false);
		let bf_code = ctx.ir_to_bf(instructions, None)?;

		Ok(bf_code.to_string())
	}

	#[test]
	fn empty_program_1() {
		assert_eq!(compile_and_run::<TapeCell, Opcode>("", "").unwrap(), "");
	}

	#[test]
	fn empty_program_1a() {
		assert_eq!(compile_and_run::<TapeCell, Opcode>(";;;", "").unwrap(), "");
	}

	#[test]
	fn empty_program_2() {
		assert_eq!(compile_and_run::<TapeCell, Opcode>("{}", "").unwrap(), "");
	}

	#[test]
	fn empty_program_2a() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>("{;;};", "").unwrap(),
			""
		);
	}

	#[test]
	fn empty_program_2b() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>("{{{{}}}}", "").unwrap(),
			""
		);
	}

	#[test]
	fn empty_program_2c() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(
				"{{}} {} {{{}{}}} {{{ { }{ }} {{ }{ }}} {{{ }{}}{{} {}}}}",
				""
			)
			.unwrap(),
			""
		);
	}

	#[test]
	fn empty_program_2d() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(
				"{{}} {} {{{}{}}} {{{ { }{ ;}}; {{ }{ }};} {{{; }{;};}{;{;};; {};}}}",
				""
			)
			.unwrap(),
			""
		);
	}

	#[test]
	fn empty_program_3() {
		assert_eq!(compile_and_run::<TapeCell, Opcode>(";", "").unwrap(), "");
	}

	#[test]
	fn empty_program_3a() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(";;;;;;", "").unwrap(),
			""
		);
	}

	#[test]
	fn empty_program_3b() {
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(";;{;{;};};;;", "").unwrap(),
			""
		);
	}

	#[test]
	fn hello_1() {
		let program = r#"
cell h = 8;
cell e = 5;
cell l = 12;
cell o = 15;
// comment!
cell a_char = 96;
drain a_char into h e l o;
output h;
output e;
output l;
output l;
output o;
cell ten = 10;
output ten;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\n"
		);
	}

	#[test]
	fn hello_2() {
		let program = r#"
output 'h';
output 'e';
output 'l';
output 'l';
output 'o';
output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\n"
		);
	}

	#[test]
	fn hello_3() {
		let program = r#"
output  'h'  ;;;
// comment
cell[5] EEL =    "ello\n";
output EEL[0];
output EEL[1];
output EEL[2];
output EEL[3];
output EEL[4];
output '\n';
output 0;
output 70;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\n\n\0F"
		)
	}

	#[test]
	fn hello_4() {
		let program = r#"
cell[4] str = [5, 12, 12, 15];
cell a = 'a' - 1;
drain a into *str;
output 'H';
output *str;
output 46;
output 10;
output "What?";
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Hello.\nWhat?"
		);
	}

	#[test]
	fn hello_5() {
		let program = r#"
output "Hell";
output ['o', '.',  '\n'];
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Hello.\n"
		);
	}

	#[test]
	fn expressions_1() {
		let program = r#"
output '@' + 256 + 1 + false + true + 'e' - '@';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"g"
		);
	}

	#[test]
	fn expressions_2() {
		let program = r#"
cell p = 9 - (true + true -(-7));
if not p {
	output "Hi friend!\n";
}

cell q = 8 + p - (4 + p);
q -= 4;
if q {
	output "path a";
} else {
	output "path b";
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Hi friend!\npath b"
		);
	}

	#[test]
	fn expressions_3() {
		let program = r#"
if 56 - 7 {
	output 'A';
} else {
	output 'B';
}

cell not_a = 'a' + (-1) - (0 - 1);
if not not_a - 'a' {
	output 'C';
} else {
	output 'D';
}

not_a += 1;
if not_a - 'a' {
	output not_a;
} else {
	output 'F';
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"ACb"
		);
	}

	#[test]
	fn expressions_4() {
		let program = r#"
cell x = 5;
cell A = 'A';

drain 0 + x + 1 into A {
	output '6';
}

output ' ';
output A;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"666666 G"
		);
	}

	#[test]
	fn assignments_1() {
		let program = r#"
cell x = 5;
output '0' + x;
x += 1;
output '0' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"56"
		);
	}

	#[test]
	fn assignments_2() {
		let program = r#"
cell x = 5;
output '0' + x;
x = x + 1;
output '0' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"56"
		);
	}
	#[test]
	fn assignments_3() {
		let program = r#"
cell x = 5;
output '0' + x;
x += 1 + x;
output '0' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"5;"
		);
	}

	#[test]
	fn assignments_4() {
		let program = r#"
cell x = 2;
output '0' + x;
x = x + x + x;
output '0' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"26"
		);
	}

	#[test]
	fn assignments_5() {
		let program = r#"
cell x = 2;
x = (2 + 3) - ((x + 4) + 1) + 4 - (12) + (3 + 10);
output '0' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"3"
		);
	}

	#[test]
	fn assignments_6() {
		let program = r#"
cell[2] x = [4, 5];
x[0] = x[0] + 4;
x[1] = x[1] - 3;

x[0] += '0';
x[1] += '0';
output *x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"82"
		);
	}

	#[test]
	fn assignments_7() {
		let program = r#"
cell[2] x = [1, 2];
x[0] = x[1] + 5; // 7
x[1] = x[0] + x[1]; // 9

x[0] += '0';
x[1] += '0';
output *x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"79"
		);
	}

	#[test]
	fn assignments_8() {
		let program = r#"
cell x = 128;
output x - 2;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"~"
		);
	}

	#[test]
	fn assignments_8a() {
		let program = r#"
cell x = 127;
cell y = 64;
x += y + y;
output x + 'f' + 1;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"f"
		);
	}

	#[test]
	fn assignments_8b() {
		let program = r#"
cell x = 128;
cell y = 64;
x += y + y;
output x + 'f';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"f"
		)
	}

	#[test]
	fn assignments_9() {
		let program = r#"
cell x = 128;
x += 128;
output x + 'f';
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.len() < 200);
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "f");
	}

	#[test]
	fn assignments_9a() {
		let program = r#"
cell x = 126;
x += 2;
x += 128;
output x + 'f';
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.len() < 200);
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "f");
	}

	#[test]
	fn increment_1() {
		let program = r#"
cell x = 'h';
output x;
++x;
output x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hi"
		)
	}

	#[test]
	fn increment_2() {
		let program = r#"
cell x = 'h';
output x;
--x;
output x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hg"
		)
	}

	// TODO: add pre-increment to expressions? (probably not worth it)
	#[test]
	#[ignore]
	fn increment_3() {
		let program = r#"
cell x = 'a';
output ++x;
output x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"bb"
		)
	}

	#[test]
	#[ignore]
	fn increment_3a() {
		let program = r#"
cell x = 'a';
output x;
output ++x + 2;
output x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"adb"
		)
	}

	#[test]
	#[ignore]
	fn increment_3b() {
		let program = r#"
cell x = 'd';
output --x;
output x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"cc"
		)
	}

	#[test]
	#[ignore]
	fn increment_3c() {
		let program = r#"
cell x = 'd';
output 4+--x;
output --x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"gb"
		)
	}

	#[test]
	#[ignore]
	fn increment_4() {
		let program = r#"
cell x = -1;
if ++x {output 'T';}
else {output 'F';}
output 'e' + ++x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Ff"
		)
	}

	#[test]
	#[ignore]
	fn increment_4a() {
		let program = r#"
cell x = 0;
if --x {output 'T';}
else {output 'F';}
output 'e' + x;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Td"
		)
	}

	#[test]
	fn loops_1() {
		let program = r#"
cell n = '0';
cell a = 10;
cell b = 1;
drain a {
	output n;
	++n;
	output 'A';
	cell c = b;
	drain c {
		output 'B';
	};
	b += 1;
	output 10;
};
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"0AB\n1ABB\n2ABBB\n3ABBBB\n4ABBBBB\n5ABBBBBB\n6ABBBBBBB\n7ABBBBBBBB\n8ABBBBBBBB\
B\n9ABBBBBBBBBB\n"
		)
	}

	#[test]
	fn loops_2() {
		let program = r#"
cell a = 4;
cell[6] b = [65, 65, 65, 65, 65, 1];
copy a into b[0] b[1] b[4] b[5] {
	copy b[5] into b[2];
	
	output b[0];
	output b[1];
	output b[2];
	output b[3];
	output b[4];
	output 10;
}a+='a';output a;

cell g = 5;
drain g into a {output a;}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").expect(""),
			"AABAA\nBBDAB\nCCGAC\nDDKAD\neefghi"
		)
	}

	#[test]
	fn loops_3() {
		let program = r#"
drain 40;
output 'h';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").expect(""),
			"h"
		)
	}

	#[test]
	fn ifs_1() {
		let program = r#"
cell x = 7;
cell y = 9;

cell z = x - y;
if z {
	output 'A';
} else {
	output 'B';
};

y -= 1;

z = x - y;
if z {
	output 'C';
} else {
	output 'D';
};

y -= 1;

z = x - y;
if not z {
	output 'E';
} else {
	output 'F';
};

output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"ACE\n"
		);
	}

	#[test]
	fn ifs_2() {
		let program = r#"
cell x = 7;
cell y = 9;

cell z = x - y;
if z {
	output 'A';
} else {
	output 'B';
}

y -= 1;

if not y {output 'H';}

z = x - y;
if z {
	output 'C';
} else {
	output 'D';
}

y -= 1;

z = x - y;
if not z {
	output 'E';
} else {
	output 'F';
}

output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"ACE\n"
		);
	}

	#[test]
	fn ifs_3() {
		let program = r#"
cell a = 5;
if a {
	cell b = a + '0';
	output b;
}
output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"5\n"
		);
	}

	#[test]
	fn loops_and_ifs_1() {
		let program = r#"
cell n = '0';
cell a = 6;
cell b;
drain a {
	output n;++n;
;;;;;;
	output 'A';

	cell c;
	cell nt_eq = a - b;

	if nt_eq {
		c = 2;
	} else {
		c = 10;
	}

	drain c {output 'B';};

	b += 1;
	output 10;
};
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"0ABB\n1ABB\n2ABB\n3ABBBBBBBBBB\n4ABB\n5ABB\n"
		);
	}

	#[test]
	fn functions_1() {
		let program = r#"
cell global_var = '0';

fn func_0(cell grape) {
	cell n = grape + 1;
	output n;
	n = 0;
};;

fn func_1(cell grape) {
	cell n = grape + 2;
	output n;
	n = 0;
}

output global_var;
func_0(global_var);
output global_var;

global_var += 1;;;
output global_var;
;;func_1(global_var);
output global_var;

output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"010131\n"
		);
	}

	#[test]
	fn functions_2() {
		let program = r#"
cell global_var = '0';

fn func_0(cell grape) {
	cell n = grape + 1;
	output n;

	fn func_1(cell grape) {
		grape += 1;
		output grape;
		grape += 1;
	};

	func_1(n);
	output n;

	grape += 1;
};

output global_var;
func_0(global_var);
output global_var;

output 10;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "01231\n");
	}

	#[test]
	fn functions_3() {
		let program = r#"
cell global_var = '0';

cell[2] global_vars = ['0', 64];

fn func_0(cell grape) {
	cell n = grape + 1;
	output n;

	fn func_1(cell grape) {
		grape += 1;
		output grape;
		grape += 1;

		cell[4] frog;
		cell zero = '0';
		drain zero into *frog;
		frog[1] += 2;

		zero = grape + 3;
		func_2(frog, zero);
		output zero;
	};

	func_1(n);
	output n;

	grape += 1;
};

output global_var;
func_0(global_var);
output global_var;

output 10;

output global_vars[1];
func_0(global_vars[0]);
output global_vars[0];

output 10;

fn func_2(cell[4] think, cell green) {
	think[2] += 7;
	think[3] += 2;

	output think[0];
	output think[1];
	output think[2];
	output think[3];

	output green;
	// this originally worked but I realised I don't actually need this
	// technically green is not declared in this scope because functions are more like templates but I still think removing this functionality is justified
	// cell green = '$';
	// output green;
	// green = 0;
};
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"01202726631\n@1202726631\n"
		);
	}

	#[test]
	fn functions_3a() {
		let program = r#"
cell[4] a = "AACD";
add_one(a[1]);
output *a;

fn add_one(cell cel) {
  ++cel;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"ABCD"
		);
	}

	#[test]
	fn functions_3b() {
		let program = r#"
struct A {cell[3] arr;};
struct A a;
a.arr[0] = '0';
a.arr[1] = '0';
a.arr[2] = '0';

add_one_to_three(a.arr);
output *a.arr;

fn add_one_to_three(cell[3] t) {
  t[0] += 1;
  t[1] += 1;
  t[2] += 1;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"111"
		);
	}

	#[test]
	fn functions_3c() {
		let program = r#"
struct A {cell b; cell c;};
struct A a;
a.b = '0';
a.c = '0';

add_one(a.b);
add_one(a.c);
add_one(a.c);
output a.b;
output a.c;

fn add_one(cell t) {
  ++t;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"12"
		);
	}

	#[test]
	fn functions_3d() {
		let program = r#"
struct A {cell b; cell c;};
struct A a;
a.b = '0';
a.c = '0';

add_one(a.b);
add_one(a.c);
add_one(a.c);
output a.b;
output a.c;

output 10;

add_one(a);
output a.b;
output a.c;

fn add_one(cell t) {
  ++t;
}

fn add_one(struct A t) {
  ++t.b;
	++t.c;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"12\n23"
		);
	}

	#[test]
	fn functions_3e() {
		let program = r#"
struct A {cell b; cell c;};
struct A a;
a.b = '0';
a.c = '0';

add_one(a.b);
add_one(a.c);
add_one(a.c);
output a.b;
output a.c;

output 10;

add_one(a, a.b);
output a.b;
output a.c;

fn add_one(cell t) {
  ++t;
}

fn add_one(struct A t, cell a) {
  ++t.b;
	++t.c;
	++a;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"12\n33"
		);
	}

	#[test]
	fn functions_3f() {
		let program = r#"
struct A {cell b; cell c;};
struct A a;
a.b = '0';
a.c = '0';

add_one(a.b);
add_one(a.c);
add_one(a.c);
output a.b;
output a.c;

output 10;

add_one(a, a.b);
output a.b;
output a.c;

fn add_one(cell t) {
  ++t;
}

fn add_one(struct A t, cell a) {
  ++t.b;
	++t.c;
	++a;
}

fn add_one(struct A tfoaishjdf, cell aaewofjas) {
  output "hello";
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap_err(),
			"Cannot define a function with the same signature more than once in the same scope: \"add_one\""
		);
	}

	#[test]
	fn functions_4() {
		let program = r#"
fn hello() {
	output "hello";
}

hello();
output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\n"
		);
	}

	#[test]
	fn function_overloads_1() {
		let program = r#"
fn hello(cell h) {
  output "hello: ";
	output h;
}
fn hello() {
	output "hello";
}

hello();
output 10;
cell g =  'g';
hello(g);
output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\nhello: g\n"
		);
	}

	#[test]
	fn function_overloads_1a() {
		let program = r#"
fn hello() {
  output "hello";
}
fn hello(cell h) {
  hello();
  output ": ";
  output h;
}

hello();
output 10;
cell g =  'g';
hello(g);
output 10;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hello\nhello: g\n"
		);
	}

	#[test]
	fn input_1() {
		let program = r#"
cell b;
input b;
++b;
output b;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "A").unwrap(),
			"B"
		);
	}

	#[test]
	fn input_2() {
		let program = r#"
cell[3] b;
input b[0];
input b[1];
input b[2];
output b[0];
output b[1];
output b[2];
b[0]+=3;
b[1]+=2;
output '\n';
b[2]+=1;
output b[2];
output b[1];
output b[0];
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "ABC").unwrap(),
			"ABC\nDDD"
		);
	}

	#[test]
	fn memory_1() {
		let program = r#"
cell[3] b = "Foo";

fn inc(cell h, cell g) {
	g += 1;
	if h {h += 1;} else {h = 'Z';}
}

output *b;
inc(b[1], b[2]);
output *b;

output 10;

cell c = -1;
inc(c, c);
output c;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"FooFpp\nZ"
		);
	}

	#[test]
	fn memory_2() {
		let program = r#"
cell[3] b = [1, 2, 3];

fn drain_h(cell h) {
	drain h {
		output 'h';
	}
}

drain_h(b[2]);
drain_h(b[2]);
output ' ';
drain_h(b[1]);
output ' ';

fn drain_into(cell a, cell[5] b) {
	drain a into *b;
}

cell u = 'a' - 1;
cell[5] v = [8, 5, 12, 12, 15];
drain_into(u, v);
output *v;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"hhh hh hello"
		);
	}

	#[test]
	fn blocks_1() {
		let program = r#"
{{{{{{{
	cell g = 0 + 5 + (-(-5));
	output "Freidns";
	{
		output g;
	}
}}}}}}}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"Freidns\n"
		);
	}

	#[test]
	fn blocks_2() {
		let program = r#"
cell f = 'f';
output f;
{
	cell f = 'F';
	output f;
}
output f;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"fFf"
		);
	}

	#[test]
	fn dimensional_arrays_1() {
		let program = r#"
cell[4][3] g;
g[0][0] = 5 + '0';
g[0][1] = 4 + '0';
g[0][2] = 3 + '0';

g[1][0] = 1 + '0';
g[1][1] = 2 + '0';
g[1][2] = 3 + '0';

g[0][3] = 1 + '0';
g[1][3] = 2 + '0';
g[2][3] = 3 + '0';

g[2][0] = 0 + '0';
g[2][1] = 0 + '0';
g[2][2] = 0 + '0';

output g[0][0];
output g[0][1];
output g[0][2];
output g[0][3];
output g[1][0];
output g[1][1];
output g[1][2];
output g[1][3];
output g[2][0];
output g[2][1];
output g[2][2];
output g[2][3];
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"543112320003"
		);
	}

	#[test]
	fn structs_1() {
		let program = r#"
struct AA {
  cell green;
	cell yellow;
}

struct AA a;
output '0' + a.green;
output '0' + a.yellow;

a.green = 6;
a.yellow = 4;
output '0' + a.green;
output '0' + a.yellow;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"0064"
		);
	}

	#[test]
	fn structs_2() {
		let program = r#"
struct AA {
  cell green;
	cell yellow;
}

// struct AA a {green = 3, yellow = 4};
struct AA a; a.green = 3; a.yellow = 4;
output '0' + a.green;
output '0' + a.yellow;

a.green = 5;
a.yellow = 2;
output '0' + a.green;
output '0' + a.yellow;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"3452"
		);
	}

	#[test]
	fn structs_3() {
		let program = r#"
struct AA {
  cell green;
	cell yellow;
}

struct AA a;

fn input_AA(struct AA bbb) {
  input bbb.green;
  input bbb.yellow;
}

input_AA(a);

output a.yellow;
output a.green;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "gh").unwrap(),
			"hg"
		);
	}

	#[test]
	fn structs_3a() {
		let program = r#"
struct AA a;

fn input_AA(struct AA bbb) {
  input bbb.green;
  input bbb.yellow;

  struct AA {
    cell[10] g;
  }
}

input_AA(a);

output a.yellow;
output a.green;

struct AA {
  cell green;
	cell yellow;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "gh").unwrap(),
			"hg"
		);
	}

	#[test]
	fn structs_3b() {
		let program = r#"
struct AA a;

fn input_AA(struct AA bbb) {
  input bbb.green;
  input bbb.yellow;

	struct AA alt_a;
	input *alt_a.l;

	output alt_a.l[4];

  struct AA {
    cell[10] l;
  }
}

input_AA(a);

output a.yellow;
output a.green;

struct AA {
  cell green;
	cell yellow;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "ghpalindrome").unwrap(),
			"nhg"
		);
	}

	#[test]
	fn structs_4a() {
		let program = r#"
struct AA a;
input a.green;
input a.yellow;
input a.reds[3];
input a.reds[0];
input a.reds[1];
input a.reds[2];

struct AA {
  cell green;
  cell yellow;
  cell[4] reds;
}

output a.green;
output a.yellow;
output a.reds[0];
output a.reds[1];
output a.reds[2];
output a.reds[3];
output '\n';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "hellow").unwrap(),
			"helowl\n"
		);
	}

	#[test]
	fn structs_4b() {
		let program = r#"
struct AA a;
input a.green;
input a.yellow;
input *a.reds;

struct AA {
  cell green;
  cell yellow;
  cell[4] reds;
}

output *a.reds;
output a.yellow;
output a.green;
output '\n';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "gy0123").unwrap(),
			"0123yg\n"
		);
	}

	#[test]
	fn structs_4c() {
		let program = r#"
struct AA a;
input a.green;
input a.yellow;
// input *a.reds;
input *a.sub.blues;
input a.sub.t;

struct BB {
  cell[2] blues;
	cell t;
}

struct AA {
  cell green;
  cell yellow;
  // cell[4] reds;
	struct BB sub;
}

output a.sub.t;
output *a.sub.blues;
// output *a.reds;
output a.yellow;
output a.green;
output '\n';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "gy-+t").unwrap(),
			"t-+yg\n"
		);
	}

	#[test]
	fn structs_4d() {
		let program = r#"
struct AA a;
input *a.reds;

struct AA {
  cell[4] reds;
  cell green;
}

output a.reds[4];
output '\n';
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "0123a").unwrap_err(),
			"Index \"[4]\" must be less than array length (4)."
		);
	}

	#[test]
	fn structs_5() {
		let program = r#"
struct AA {
  cell green;
}

struct AA[2] as;
as[0].green = 5;
as[1].green = 3;

output '0' + as[0].green;
output '0' + as[1].green;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"53"
		);
	}

	#[test]
	fn structs_5a() {
		let program = r#"
struct AAA[2] as;
as[0].green = 5;
as[1].green = 3;

output '0' + as[0].green;
output '0' + as[1].green;

struct AAA {
  cell green;
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"53"
		);
	}

	#[test]
	fn structs_6() {
		let program = r#"
struct AA {
  cell green;
}
struct BB {
  cell green;
}

struct AA[2] as;
// struct BB b {
//   green = 6
// };
struct BB b;
b.green = 6;

fn input_AAs(struct AA[2] aaas) {
  input aaas[0].green;
  input aaas[1].green;
	output "HI\n";
}
input_AAs(as);

output '0' + b.green;
output as[0].green;
output as[1].green;
"#;
		let output = compile_and_run::<TapeCell, Opcode>(program, "tr").expect("");
		println!("{output}");
		assert_eq!(output, "HI\n6tr");
	}

	#[test]
	fn structs_7() {
		let program = r#"
struct BB {
	cell green;
}
struct AA {
  cell green;
	struct BB[3] bbb;
}

struct AA[2] as;

fn input_AAs(struct AA[2] aaas) {
  fn input_BB(struct BB b) {
	  input b.green;
	}
	input_BB(aaas[0].bbb[0]);
	input_BB(aaas[0].bbb[1]);
	input_BB(aaas[0].bbb[2]);
  input_BB(aaas[1].bbb[0]);
	input_BB(aaas[1].bbb[1]);
	input_BB(aaas[1].bbb[2]);
 
  input aaas[0].green;
  input aaas[1].green;
	output "HI\n";
}
input_AAs(as);

output as[0].green;
output as[0].bbb[0].green;
output as[0].bbb[1].green;
output as[0].bbb[2].green;
output as[1].green;
output as[1].bbb[0].green;
output as[1].bbb[1].green;
output as[1].bbb[2].green;
"#;
		let output = compile_and_run::<TapeCell, Opcode>(program, "abcdefgh").expect("");
		println!("{output}");
		assert_eq!(output, "HI\ngabchdef");
	}

	#[test]
	fn structs_7a() {
		let program = r#"
struct BB {
	cell green @2;
}
struct AA {
  cell green @10;
	struct BB[3] bbb @1;
}

struct AA[2] as @-9;

fn input_AAs(struct AA[2] aaas) {
  fn input_BB(struct BB b) {
	  input b.green;
	}
	input_BB(aaas[0].bbb[0]);
	input_BB(aaas[0].bbb[1]);
	input_BB(aaas[0].bbb[2]);
  input_BB(aaas[1].bbb[0]);
	input_BB(aaas[1].bbb[1]);
	input_BB(aaas[1].bbb[2]);
 
  input aaas[0].green;
  input aaas[1].green;
	output "HI\n";
}
input_AAs(as);

output as[0].green;
output as[0].bbb[0].green;
output as[0].bbb[1].green;
output as[0].bbb[2].green;
output as[1].green;
output as[1].bbb[0].green;
output as[1].bbb[1].green;
output as[1].bbb[2].green;
"#;
		let output = compile_and_run::<TapeCell, Opcode>(program, "abcdefgh").expect("");
		println!("{output}");
		assert_eq!(output, "HI\ngabchdef");
	}

	#[test]
	fn structs_bf_1() {
		let program = r#"
struct Frame {
	cell    marker     @3;
	cell    value      @0;
	cell[2] temp_cells @1;
}
struct Vector {
	struct Frame[10]  frames @1;
	cell              marker @0;
}

struct Vector vec1 @2;
vec1.marker = true;

vec1.frames[0].marker = true;
vec1.frames[0].value = 'j';
vec1.frames[1].marker = true;
vec1.frames[1].value = 'k';
vec1.frames[2].value = 'l';

bf @2 {
  [>.>>>]
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"jkl"
		);
	}

	#[test]
	fn structs_bf_1a() {
		let program = r#"
struct Frame {
	cell    marker     @2;
	cell    value      @0;
	cell[2] temp_cells @1;
}

struct Frame f;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap_err(),
			"Subfields \"marker\" and \"temp_cells\" overlap in struct."
		);
	}

	#[test]
	fn structs_bf_1b() {
		let program = r#"
struct Frame {
	cell    marker     @-2;
	cell    value      @0;
	cell[2] temp_cells @1;
}

struct Frame f;
"#;
		assert_eq!(
			compile_program::<TapeCell, Opcode>(program, None).unwrap_err(),
			"Cannot create struct field \"cell marker @-2\". Expected non-negative cell offset."
		);
	}

	#[test]
	fn structs_bf_1c() {
		let program = r#"
struct G {
	cell a @1;
	cell b @1;
}

struct G g;
g.a = 'a';
g.b = 'b';

output g.a;
output g.b;
"#;
		assert_eq!(
			compile_program::<TapeCell, Opcode>(program, None).unwrap_err(),
			"Subfields \"a\" and \"b\" overlap in struct."
		);
	}

	#[test]
	fn structs_bf_2() {
		let program = r#"
struct Green {
  // no @0 cell
  cell blue @1;
}
struct Green g @4;
g.blue = '5';

output g.blue;
bf @4 {
  >.<
}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"55"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_0() {
		let program = r#"
output '0' + sizeof(cell);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"1"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_0a() {
		let program = r#"
output '0' + sizeof(cell[5]);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"5"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_0b() {
		let program = r#"
cell a;
cell b[4];
output '0' + sizeof(a);
output '0' + sizeof(b);
output '0' + sizeof(b[2]);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"141"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_1() {
		let program = r#"
struct Green {
  cell blue;
}
let s = sizeof(struct Green);
output '0' + s;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"1"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_1a() {
		let program = r#"
struct Green {
  cell blue;
}
let s = sizeof(struct Green[3]);
output '0' + s;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"3"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_1b() {
		let program = r#"
struct Green {
  cell blue;
}
let s = sizeof(struct Green[3][2]);
output '0' + s;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"6"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_2() {
		let program = r#"
struct Green {
  cell blue;
	cell red;
}
struct Green g;
output '0' + sizeof(g);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"2"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_3() {
		let program = r#"
struct Green {
  cell blue;
	cell[5] red;
	cell yellow;
}
struct Green[2] g;
output '0' + sizeof(g) - 13;

output '0' + sizeof(g[0].blue);
output '0' + sizeof(g[0].red);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"115"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_4() {
		let program = r#"
struct Green {
  cell blue @2;
}
struct Green[3] g;
output '0' + sizeof(struct Green);
output '0' + sizeof(g);
output '0' + sizeof(g[2].blue)
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"391"
		);
	}

	#[ignore]
	#[test]
	fn sizeof_5() {
		let program = r#"
struct Blue {
  cell[2] blues;
}
struct Red {
  cell a;
	struct Blue blues;
}
struct Green {
  cell blue @2;
  struct Red red;
}
output '0' + sizeof(struct Blue);
output '0' + sizeof(struct Red);
struct Green[3] g;
output '0' + sizeof(struct Green);
output '0' + sizeof(g) - 17;
output '0' + sizeof(g[2].blue)
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"23612"
		);
	}

	#[test]
	fn memory_specifiers_1() {
		let program = r#"
cell foo @3 = 2;
{
	cell n = 12;
	while n {
		n -= 1;
		foo += 10;
	}
}
output foo;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		assert_eq!(code, ">>>++<<<++++++++++++[->>>++++++++++<<<][-]>>>.");
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "z");
	}

	#[test]
	fn memory_specifiers_2() {
		let program = r#"
cell a @5 = 4;
cell foo @0 = 2;
cell b = 10;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(">>>>>++++<<<<<++>++++++++++"));
	}

	#[test]
	fn memory_specifiers_3() {
		let program = r#"
cell a @1 = 1;
cell foo @0 = 2;
cell b = 3;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(">+<++>>+++"));
	}

	#[test]
	fn memory_specifiers_4() {
		let program = r#"
cell a @1 = 1;
cell foo @1 = 2;
cell b = 3;
"#;
		assert_eq!(
			compile_program::<TapeCell, Opcode>(program, None).unwrap_err(),
			"Location specifier @1 conflicts with another allocation"
		);
	}

	#[test]
	fn variable_location_specifiers_1() {
		let program = r#"
cell a = 'h';
bf @a {.}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "wxy").unwrap(),
			"h"
		);
	}

	#[test]
	fn variable_location_specifiers_1a() {
		let program = r#"
cell[100] _;
cell a = 'h';
cell[4] b;
bf @a {.}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"h"
		);
	}

	#[test]
	fn variable_location_specifiers_2() {
		let program = r#"
struct Test {cell[3] a @0; cell b;}
struct Test t;
input *t.a;
bf @t.a {
[+.>]
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		assert_eq!(code, ",>,>,<<[+.>]");
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "wxy", None).unwrap(), "xyz");
	}

	#[test]
	fn variable_location_specifiers_2a() {
		let program = r#"
struct Test {cell[3] a @0; cell b;}
struct Test t;
input *t.a;
bf @t {
[+.>]
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		assert_eq!(code, ",>,>,<<[+.>]");
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "wxy", None).unwrap(), "xyz");
	}

	#[test]
	fn variable_location_specifiers_3() {
		let program = r#"
cell[5] f @6 = "abcde";
bf @f[2] clobbers *f {.+++.}
output 10;
output *f;
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"cf\nabfde"
		);
	}

	#[test]
	fn variable_location_specifiers_3a() {
		let program = r#"
cell[4] f @8 = "xyz ";
bf @f {[.>]}
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"xyz "
		);
	}

	#[test]
	fn variable_location_specifiers_4() {
		let program = r#"
fn func(cell g) {
  bf @g {+.-}
}

cell a = '5';
func(a);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"6"
		);
	}

	#[test]
	fn variable_location_specifiers_4a() {
		let program = r#"
fn func(cell g) {
  bf @g {+.-}
}

cell[3] a = "456";
func(a[1]);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"6"
		);
	}

	#[test]
	fn variable_location_specifiers_4b() {
		let program = r#"
fn func(cell g) {
  bf @g {+.-}
}

struct H {cell[3] r;}
struct H a;
a.r[0] = '4';
a.r[1] = '5';
a.r[2] = '6';
func(a.r[1]);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"6"
		);
	}

	#[test]
	fn variable_location_specifiers_4c() {
		let program = r#"
fn func(struct H h) {
  bf @h {+.-}
}

struct H {cell[3] r @0;}
struct H a;
a.r[0] = '4';
a.r[1] = '5';
a.r[2] = '6';
func(a);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"5"
		);
	}

	#[test]
	fn variable_location_specifiers_4d() {
		let program = r#"
fn func(cell[2] g) {
  bf @g {+.-}
}

struct J {cell[2] j;}
struct H {cell[20] a; struct J jj @1;}
struct H a;
a.jj.j[0] = '3';
a.jj.j[1] = '4';
func(a.jj.j);
"#;
		assert_eq!(
			compile_and_run::<TapeCell, Opcode>(program, "").unwrap(),
			"4"
		);
	}

	#[test]
	fn assertions_1() {
		let program = r#"
cell a @0 = 5;
output a;
assert a equals 2;
a = 0;
output a;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.starts_with("+++++.--."));
	}

	#[test]
	fn assertions_2() {
		let program = r#"
cell a @0 = 2;
output a;
assert a unknown;
a = 0;
output a;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.starts_with("++.[-]."));
	}

	#[test]
	fn inline_brainfuck_1() {
		let program = r#"
bf {
	,.[-]
	+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		assert_eq!(
			code,
			",.[-]+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
		);
		assert_eq!(
			run_code(BVM_CONFIG_1D, &code, "~", None).unwrap(),
			"~Hello, World!"
		);
	}

	#[test]
	fn inline_brainfuck_2() {
		let program = r#"
// cell a @0;
// cell b @1;
bf @3 {
	,.[-]
	+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(
			">>>,.[-]+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
		));
		assert_eq!(
			run_code(BVM_CONFIG_1D, &code, "~", None).unwrap(),
			"~Hello, World!"
		);
	}

	#[test]
	fn inline_brainfuck_3() {
		let program = r#"
cell[3] str @0;

bf @0 clobbers *str {
	,>,>,
	<<
}

bf @0 clobbers *str {
	[+>]
	<<<
}

bf @0 clobbers *str {
	[.[-]>]
	<<<
}
assert *str equals 0;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(",>,>,<<[+>]<<<[.[-]>]<<<"));
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "HEY", None).unwrap(), "IFZ");
	}

	#[test]
	fn inline_brainfuck_4() {
		let program = r#"
bf {
	// enters a line of user input
	// runs some embedded mastermind for each character
	,----------[
		++++++++++
		{
			cell chr @0;
			assert chr unknown;
			output chr;
			chr += 1;
			output chr;
		}
		[-]
		,----------
	]
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(",----------[++++++++++"));
		assert!(code.ends_with("[-],----------]"));
		assert_eq!(
			run_code(BVM_CONFIG_1D, &code, "line of input\n", None).unwrap(),
			"lmijnoef !opfg !ijnopquvtu"
		);
	}

	#[test]
	fn inline_brainfuck_5() {
		let program = r#"
// external function within the same file, could be tricky to implement
fn quote(cell n) {
	// H 'H'
	output 39;
	output n;
	output 39;
}

bf {
	// enters a line of user input
	// runs some embedded mastermind for each character
	,----------[
		++++++++++
		{
			cell chr @0;
			assert chr unknown;
			quote(chr);
			output 10;
			// this time it may be tricky because the compiler needs to return to the start cell
		}
		[-]
		,----------
	]
}
"#;
		let code = compile_program::<TapeCell, Opcode>(program, None).unwrap();
		println!("{code}");
		assert!(code.starts_with(",----------[++++++++++"));
		assert!(code.ends_with("[-],----------]"));
		assert_eq!(
			run_code(BVM_CONFIG_1D, &code, "hello\n", None).unwrap(),
			"'h'\n'e'\n'l'\n'l'\n'o'\n"
		);
	}

	#[test]
	fn inline_brainfuck_6() {
		let program = r#"
cell b = 4;

bf {
	++--
	{
		output b;
	}
	++--
}
"#;
		assert_eq!(
			compile_program::<TapeCell, Opcode>(program, None).unwrap_err(),
			"No variable found in scope with name \"b\"."
		);
	}

	#[test]
	fn inline_brainfuck_7() {
		let program = r#"
	bf {
		,>,>,
		<<
		{{{{{{cell g @5 = 1;}}}}}}
	}
"#;
		assert_eq!(
			compile_program::<TapeCell, Opcode>(program, None).unwrap(),
			",>,>,<<>>>>>+[-]<<<<<"
		);
	}

	#[test]
	fn inline_2d_brainfuck() {
		let program = r#"
bf {,.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+.}
"#;
		let code = compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap();
		assert_eq!(
			code,
			",.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+."
		);
		assert_eq!(
			run_code(BVM_CONFIG_2D, &code, "~", None).unwrap(),
			"~Hello, World!"
		);
	}

	#[test]
	fn invalid_inline_2d_brainfuck() {
		let program = r#"
bf {,.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvstvv.+++.------.vv-.^^^^+.}
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap_err(),
			"Unexpected character `s` in Brainfuck clause."
		);
	}

	#[test]
	fn inline_2d_brainfuck_disabled() {
		assert_eq!(
			run_code(
				BVM_CONFIG_1D,
				",.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+.",
				"~",
				None,
			)
			.unwrap_err(),
			"2D Brainfuck currently disabled"
		);
	}

	#[test]
	fn constant_optimisations_1() {
		let program = r#"
output 'h';
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.len() < 35);
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "h");
	}

	#[test]
	fn constant_optimisations_2() {
		let program = r#"
cell[15] arr @1;
cell a = 'G';
cell b = a + 45;
output b;
b -= 43;
output b;
output a + 3;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.len() < 400);
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "", None).unwrap(), "tIJ");
	}

	#[test]
	#[ignore]
	fn generated_code_optimisations() {
		let program = r#"
cell x;
cell y;
cell z;
z += 4;
x += 3;
z += 4;
x += 3;
z += 4;
y = 7;
input x;
input y;
input z;
"#;
		let code = compile_program::<TapeCell, Opcode>(program, Some(OPT_ALL)).unwrap();
		println!("{code}");
		assert!(code.len() < 30);
		assert_eq!(run_code(BVM_CONFIG_1D, &code, "   ", None).unwrap(), "");
	}

	// TODO: remove the need for this
	#[test]
	fn unimplemented_memory_allocation() {
		let program = r#"
cell[15] arr @1;
cell a = 'G';
"#;
		let cfg = MastermindConfig {
			optimise_generated_code: false,
			optimise_generated_all_permutations: false,
			optimise_cell_clearing: false,
			// optimise_variable_usage: false,
			// optimise_memory_allocation: false,
			optimise_unreachable_loops: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
			memory_allocation_method: 128,
			enable_2d_grid: false,
		};
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(cfg)).unwrap_err(),
			"Memory allocation method 128 not implemented."
		);
	}

	#[test]
	fn memory_specifiers_2d_1() {
		let program = r#"
cell a @(1, 2) = 1;
cell foo @0 = 2;
cell b = 3;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap(),
			">^^+<vv++>+++"
		);
	}

	#[test]
	fn memory_specifiers_2d_2() {
		let program = r#"
cell[4][3] g @(1, 2);
g[0][0] = 1;
g[1][1] = 2;
g[2][2] = 3;
cell foo @0 = 2;
cell b = 3;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap(),
			">^^[-]+>>>>>[-]++>>>>>[-]+++<<<<<<<<<<<vv++>+++"
		);
	}

	#[test]
	fn memory_specifiers_2d_3() {
		let program = r#"
cell a @(1, 3) = 1;
cell foo @(1, 3) = 2;
cell b = 3;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap_err(),
			"Location specifier @(1, 3) conflicts with another allocation"
		);
	}

	#[test]
	fn memory_specifiers_2d_4() {
		let program = r#"
cell a @2 = 1;
cell foo @(2, 0) = 2;
cell b = 3;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap_err(),
			"Location specifier @(2, 0) conflicts with another allocation"
		);
	}

	#[test]
	fn memory_specifiers_2d_5() {
		let program = r#"
cell a @(2, 4) = 1;
cell[4] b @(0, 4);
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, None).unwrap_err(),
			"Location specifier @(0, 4) conflicts with another allocation"
		);
	}

	#[test]
	fn tiles_memory_allocation_1() {
		let program = r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_TILES)).unwrap(),
			"+<v+^+^+>vv+^^+>vv+^+^+"
		);
	}
	#[test]
	fn tiles_memory_allocation_2() {
		let program = r#"
cell a = '1';
cell b = '2';
cell c = '3';
cell d = '4';
cell e = '5';
cell f = '6';
cell g = '7';
cell h = '8';
cell i = '9';
output a;
output b;
output c;
output d;
output e;
output f;
output g;
output h;
output i;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_TILES)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(
			run_code(BVM_CONFIG_2D, &code, "", None).unwrap(),
			"123456789"
		);
	}

	#[test]
	fn tiles_memory_allocation_3() {
		let program = r#"
cell a @(2, 4) = 1;
cell[4] b @(0, 4);
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_TILES)).unwrap_err(),
			"Location specifier @(0, 4) conflicts with another allocation"
		);
	}

	#[test]
	fn tiles_memory_allocation_4() {
		let program = r#"
cell a @2 = 1;
cell[4] b;
a = '5';
b[0] = '1';
b[1] = '2';
b[2] = '3';
b[3] = '4';
output b[0];
output b[1];
output b[2];
output b[3];
output a;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_TILES)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(run_code(BVM_CONFIG_2D, &code, "", None).unwrap(), "12345");
	}

	#[test]
	fn zig_zag_memory_allocation_1() {
		let program = r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_ZIG_ZAG)).unwrap(),
			"+>+<^+>>v+<^+<^+>>>vv+<^+<^+"
		);
	}

	#[test]
	fn zig_zag_memory_allocation_2() {
		let program = r#"
cell a = '1';
cell b = '2';
cell c = '3';
cell d = '4';
cell e = '5';
cell f = '6';
cell g = '7';
cell h = '8';
cell i = '9';
output a;
output b;
output c;
output d;
output e;
output f;
output g;
output h;
output i;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_ZIG_ZAG)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(
			run_code(BVM_CONFIG_2D, &code, "", None).unwrap(),
			"123456789"
		);
	}

	#[test]
	fn zig_zag_memory_allocation_3() {
		let program = r#"
cell a @(2, 4) = 1;
cell[4] b @(0, 4);
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_ZIG_ZAG))
				.unwrap_err(),
			"Location specifier @(0, 4) conflicts with another allocation"
		);
	}

	#[test]
	fn zig_zag_memory_allocation_4() {
		let program = r#"
cell a @2 = 1;
cell[4] b;
a = '5';
b[0] = '1';
b[1] = '2';
b[2] = '3';
b[3] = '4';
output b[0];
output b[1];
output b[2];
output b[3];
output a;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_ZIG_ZAG)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(run_code(BVM_CONFIG_2D, &code, "", None).unwrap(), "12345");
	}

	#[test]
	fn spiral_memory_allocation_1() {
		let program = r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_SPIRAL)).unwrap(),
			"^+>+v+<+<+^+^+>+>+"
		);
	}
	#[test]
	fn spiral_memory_allocation_2() {
		let program = r#"
cell a = '1';
cell b = '2';
cell c = '3';
cell d = '4';
cell e = '5';
cell f = '6';
cell g = '7';
cell h = '8';
cell i = '9';
output a;
output b;
output c;
output d;
output e;
output f;
output g;
output h;
output i;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_SPIRAL)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(
			run_code(BVM_CONFIG_2D, &code, "", None).unwrap(),
			"123456789"
		);
	}

	// TODO: decipher this
	#[test]
	fn spiral_memory_allocation_3() {
		let program = r#"
cell a @(2, 4) = 1;
cell[4] b @(0, 4);
"#;
		assert_eq!(
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_SPIRAL)).unwrap_err(),
			"Location specifier @(0, 4) conflicts with another allocation"
		);
	}

	#[test]
	fn spiral_memory_allocation_4() {
		let program = r#"
cell a @2 = 1;
cell[4] b;
a = '5';
b[0] = '1';
b[1] = '2';
b[2] = '3';
b[3] = '4';
output b[0];
output b[1];
output b[2];
output b[3];
output a;
"#;
		let code =
			compile_program::<TapeCell2D, Opcode2D>(program, Some(OPT_NONE_2D_SPIRAL)).unwrap();
		println!("{code}");
		assert!(code.contains("v") || code.contains("^"));
		assert_eq!(run_code(BVM_CONFIG_2D, &code, "", None).unwrap(), "12345");
	}
}
