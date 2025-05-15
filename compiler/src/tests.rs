#![allow(dead_code)]

// black box testing
#[cfg(test)]
pub mod tests {
	use crate::{
		brainfuck::{tests::run_code, BVMConfig},
		builder::{BrainfuckOpcodes, Builder, Opcode},
		compiler::Compiler,
		parser::parse,
		tokeniser::{tokenise, Token},
		MastermindConfig,
	};
	// TODO: run test suite with different optimisations turned on
	const OPT_NONE: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		optimise_variable_usage: false,
		optimise_memory_allocation: false,
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
		optimise_variable_usage: true,
		optimise_memory_allocation: true,
		optimise_unreachable_loops: true,
		optimise_constants: true,
		optimise_empty_blocks: true,
		memory_allocation_method: 0,
		enable_2d_grid: false,
	};

	const OPT_NONE_TILES: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		optimise_variable_usage: false,
		optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 3,
		enable_2d_grid: false,
	};

	const OPT_NONE_SPIRAL: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		optimise_variable_usage: false,
		optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 2,
		enable_2d_grid: false,
	};

	const OPT_NONE_ZIG_ZAG: MastermindConfig = MastermindConfig {
		optimise_generated_code: false,
		optimise_generated_all_permutations: false,
		optimise_cell_clearing: false,
		optimise_variable_usage: false,
		optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
		memory_allocation_method: 1,
		enable_2d_grid: false,
	};

	const BVM_CONFIG_1D: BVMConfig = BVMConfig {
		ENABLE_DEBUG_SYMBOLS: false,
		ENABLE_2D_GRID: false,
	};

	const BVM_CONFIG_2D: BVMConfig = BVMConfig {
		ENABLE_DEBUG_SYMBOLS: false,
		ENABLE_2D_GRID: true,
	};

	const TESTING_BVM_MAX_STEPS: usize = 100_000_000;

	fn compile_and_run(program: String, input: String) -> Result<String, String> {
		// println!("{program}");
		// compile mastermind
		let tokens: Vec<Token> = tokenise(&program)?;
		// println!("{tokens:#?}");
		let clauses = parse(&tokens)?;
		// println!("{clauses:#?}");
		let instructions = Compiler { config: &OPT_NONE }
			.compile(&clauses, None)?
			.finalise_instructions(false);
		// println!("{instructions:#?}");
		let bf_program = Builder { config: &OPT_NONE }.build(instructions, false)?;
		let bfs = bf_program.to_string();
		// println!("{}", bfs);
		// run generated brainfuck with input
		Ok(run_code(
			BVM_CONFIG_1D,
			bfs,
			input,
			Some(TESTING_BVM_MAX_STEPS),
		))
	}

	fn compile_program(
		program: String,
		config: Option<&MastermindConfig>,
	) -> Result<Vec<Opcode>, String> {
		// println!("{program}");
		// compile mastermind
		let tokens: Vec<Token> = tokenise(&program)?;
		// println!("{tokens:#?}");
		let clauses = parse(&tokens)?;
		// println!("{clauses:#?}");
		let instructions = Compiler {
			config: config.unwrap_or(&OPT_NONE),
		}
		.compile(&clauses, None)?
		.finalise_instructions(false);
		// println!("{instructions:#?}");
		let bf_code = Builder {
			config: config.unwrap_or(&OPT_NONE),
		}
		.build(instructions, false)?;
		// println!("{}", bfs);

		Ok(bf_code)
	}

	// #[test]
	fn dummy_success_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	// #[test]
	fn dummy_compile_fail_test() {
		let program = String::from("");
		let result = compile_program(program, None);
		assert!(result.is_err());
	}

	// #[test]
	fn dummy_code_test() {
		let program = String::from("");
		let desired_code = String::from("");
		let code = compile_program(program, None).expect("").to_string();
		println!("{code}");
		assert_eq!(desired_code, code);

		let input = String::from("");
		let desired_output = String::from("");
		let output = run_code(BVM_CONFIG_1D, code, input, None);
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn hello_1() {
		let program = String::from(
			"
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
      ",
		);
		let input = String::from("");
		let desired_output = String::from("hello\n");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""));
	}

	#[test]
	fn hello_2() {
		let program = String::from(
			"
output 'h';
output 'e';
output 'l';
output 'l';
output 'o';
output 10;
      ",
		);
		let input = String::from("");
		let desired_output = String::from("hello\n");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
	}

	#[test]
	fn hello_3() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("hello\n\n\0F");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn hello_4() {
		let program = String::from(
			r#"
cell[4] str = [5, 12, 12, 15];
cell a = 'a' - 1;
drain a into *str;
output 'H';
output *str;
output 46;
output 10;
output "What?";
"#,
		);
		let input = String::from("");
		let desired_output = String::from("Hello.\nWhat?");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn hello_5() {
		let program = String::from(
			r#"
output "Hell";
output ['o', '.',  '\n'];
"#,
		);
		let input = String::from("");
		let desired_output = String::from("Hello.\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn expressions_1() {
		let program = String::from(
			r#";
output '@' + 256 + 1 + false + true + 'e' - '@';
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("g");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn expressions_2() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("Hi friend!\npath b");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn expressions_3() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("ACb");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn expressions_4() {
		let program = String::from(
			r#";
cell x = 5;
cell A = 'A';

drain 0 + x + 1 into A {
	output '6';
}

output ' ';
output A;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("666666 G");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_1() {
		let program = String::from(
			r#";
cell x = 5;
output '0' + x;
x += 1;
output '0' + x;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("56");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_2() {
		let program = String::from(
			r#";
cell x = 5;
output '0' + x;
x = x + 1;
output '0' + x;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("56");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}
	#[test]
	fn assignments_3() {
		let program = String::from(
			r#";
cell x = 5;
output '0' + x;
x += 1 + x;
output '0' + x;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("5;");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_4() {
		let program = String::from(
			r#";
cell x = 2;
output '0' + x;
x = x + x + x;
output '0' + x;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("26");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_5() {
		let program = String::from(
			r#";
cell x = 2;
x = (2 + 3) - ((x + 4) + 1) + 4 - (12) + (3 + 10);
output '0' + x;
		"#,
		);
		let input = String::from("");
		let desired_output = String::from("3");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_6() {
		let program = String::from(
			r#";
cell[2] x = [4, 5];
x[0] = x[0] + 4;
x[1] = x[1] - 3;

x[0] += '0';
x[1] += '0';
output *x;
        "#,
		);
		let input = String::from("");
		let desired_output = String::from("82");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn assignments_7() {
		let program = String::from(
			r#";
cell[2] x = [1, 2];
x[0] = x[1] + 5; // 7
x[1] = x[0] + x[1]; // 9

x[0] += '0';
x[1] += '0';
output *x;
        "#,
		);
		let input = String::from("");
		let desired_output = String::from("79");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn loops_1() {
		let program = String::from(
			"
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
      ",
		);
		let input = String::from("");
		let desired_output = String::from("0AB\n1ABB\n2ABBB\n3ABBBB\n4ABBBBB\n5ABBBBBB\n6ABBBBBBB\n7ABBBBBBBB\n8ABBBBBBBBB\n9ABBBBBBBBBB\n");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
	}

	#[test]
	fn loops_2() {
		let program = String::from(
			"
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
      ",
		);
		let input = String::from("");
		let desired_output = String::from("AABAA\nBBDAB\nCCGAC\nDDKAD\neefghi");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
	}

	#[test]
	fn loops_3() {
		let program = String::from(
			"
drain 40;
output 'h';
      ",
		);
		let input = String::from("");
		let desired_output = String::from("h");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
	}

	#[test]
	fn ifs_1() {
		let program = String::from(
			"
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
		",
		);
		let input = String::from("");
		let desired_output = String::from("ACE\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn ifs_2() {
		let program = String::from(
			"
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
		",
		);
		let input = String::from("");
		let desired_output = String::from("ACE\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn ifs_3() {
		let program = String::from(
			"
cell a = 5;
if a {
	cell b = a + '0';
	output b;
}
output 10;
		",
		);
		let input = String::from("");
		let desired_output = String::from("5\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn loops_and_ifs_1() {
		let program = String::from(
			"
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
      ",
		);
		let input = String::from("");
		let desired_output = String::from("0ABB\n1ABB\n2ABB\n3ABBBBBBBBBB\n4ABB\n5ABB\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_1() {
		let program = String::from(
			"
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
		",
		);
		let input = String::from("");
		let desired_output = String::from("010131\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_2() -> Result<(), String> {
		let program = String::from(
			"
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
		",
		);
		let input = String::from("");
		let desired_output = String::from("01231\n");
		let code = compile_program(program, Some(&OPT_NONE))?.to_string();
		println!("{}", code);
		let output = run_code(BVM_CONFIG_1D, code, input, None);
		println!("{output}");
		assert_eq!(desired_output, output);

		Ok(())
	}

	#[test]
	fn functions_3() {
		let program = String::from(
			"
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
		",
		);
		let input = String::from("");
		let desired_output = String::from("01202726631\n@1202726631\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3a() {
		let program = String::from(
			r#"
cell[4] a = "AACD";
add_one(a[1]);
output *a;

fn add_one(cell cel) {
  ++cel;
}
"#,
		);
		let input = String::from("");
		let desired_output = String::from("ABCD");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3b() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("111");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3c() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("12");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3d() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("12\n23");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3e() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("12\n33");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	#[should_panic]
	fn functions_3f() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("12\n33");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_4() {
		let program = String::from(
			r#"
fn hello() {
	output "hello";
}

hello();
output 10;
		"#,
		);
		let input = String::from("");
		let desired_output = String::from("hello\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn input_1() {
		let program = String::from(
			"
cell b;
input b;
++b;
output b;
",
		);
		let input = String::from("A");
		let desired_output = String::from("B");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn input_2() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("ABC");
		let desired_output = String::from("ABC\nDDD");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn memory_1() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("FooFpp\nZ");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn memory_2() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("hhh hh hello");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn blocks_1() {
		let program = String::from(
			r#"
{{{{{{{
	cell g = 0 + 5 + (-(-5));
	output "Freidns";
	{
		output g;
	}
}}}}}}}
"#,
		);
		let input = String::from("");
		let desired_output = String::from("Freidns\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn blocks_2() {
		let program = String::from(
			r#"
cell f = 'f';
output f;
{
	cell f = 'F';
	output f;
}
output f;
	"#,
		);
		let input = String::from("");
		let desired_output = String::from("fFf");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn dimensional_arrays_1() {
		let program = String::from(
			r#"
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
"#,
		);
		let input = String::from("");
		let desired_output = String::from("543112320003");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_1() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("0064");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_2() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("3452");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_3() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("gh");
		let desired_output = String::from("hg");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_3a() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("gh");
		let desired_output = String::from("hg");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_3b() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("ghpalindrome");
		let desired_output = String::from("nhg");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_4a() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("hellow");
		let desired_output = String::from("helowl\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_4b() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("gy0123");
		let desired_output = String::from("0123yg\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_4c() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("gy-+t");
		let desired_output = String::from("t-+yg\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	#[should_panic]
	fn structs_4d() {
		let program = String::from(
			r#";
struct AA a;
input *a.reds;

struct AA {
  cell[4] reds;
  cell green;
}

output a.reds[4];
output '\n';
			"#,
		);
		let input = String::from("0123a");
		let desired_output = String::from("a\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_5() {
		let program = String::from(
			r#";
struct AA {
  cell green;
}

struct AA[2] as;
as[0].green = 5;
as[1].green = 3;

output '0' + as[0].green;
output '0' + as[1].green;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("53");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_5a() {
		let program = String::from(
			r#"
struct AAA[2] as;
as[0].green = 5;
as[1].green = 3;

output '0' + as[0].green;
output '0' + as[1].green;

struct AAA {
  cell green;
}
"#,
		);
		let input = String::from("");
		let desired_output = String::from("53");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_6() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("tr");
		let desired_output = String::from("HI\n6tr");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_7() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("abcdefgh");
		let desired_output = String::from("HI\ngabchdef");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_7a() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("abcdefgh");
		let desired_output = String::from("HI\ngabchdef");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_bf_1() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("jkl");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	// TODO: fix the r_panic macro that makes this error have unescaped quotes in it (weird)
	// #[should_panic(expected = r#"Subfields "marker" and "temp_cells" overlap in struct."#)]
	#[should_panic]
	fn structs_bf_1a() {
		let program = String::from(
			r#";
struct Frame {
	cell    marker     @2;
	cell    value      @0;
	cell[2] temp_cells @1;
}

struct Frame f;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	// TODO: fix the r_panic macro that makes this error have unescaped quotes in it (weird)
	// #[should_panic(expected = r#"Subfields "marker" and "temp_cells" overlap in struct."#)]
	#[should_panic]
	fn structs_bf_1b() {
		let program = String::from(
			r#";
struct Frame {
	cell    marker     @-2;
	cell    value      @0;
	cell[2] temp_cells @1;
}

struct Frame f;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	#[should_panic]
	fn structs_bf_1c() {
		let program = String::from(
			r#";
struct G {
	cell a @1;
	cell b @1;
}

struct G g;
g.a = 'a';
g.b = 'b';

output g.a;
output g.b;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("ab");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn structs_bf_2() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("55");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_0() {
		let program = String::from(
			r#";
output '0' + sizeof(cell);
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("1");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_0a() {
		let program = String::from(
			r#";
output '0' + sizeof(cell[5]);
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("5");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_0b() {
		let program = String::from(
			r#";
cell a;
cell b[4];
output '0' + sizeof(a);
output '0' + sizeof(b);
output '0' + sizeof(b[2]);
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("141");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_1() {
		let program = String::from(
			r#";
struct Green {
  cell blue;
}
let s = sizeof(struct Green);
output '0' + s;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("1");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_1a() {
		let program = String::from(
			r#";
struct Green {
  cell blue;
}
let s = sizeof(struct Green[3]);
output '0' + s;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("3");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_1b() {
		let program = String::from(
			r#";
struct Green {
  cell blue;
}
let s = sizeof(struct Green[3][2]);
output '0' + s;
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("6");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_2() {
		let program = String::from(
			r#";
struct Green {
  cell blue;
	cell red;
}
struct Green g;
output '0' + sizeof(g);
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("2");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_3() {
		let program = String::from(
			r#";
struct Green {
  cell blue;
	cell[5] red;
	cell yellow;
}
struct Green[2] g;
output '0' + sizeof(g) - 13;

output '0' + sizeof(g[0].blue);
output '0' + sizeof(g[0].red);
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("115");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_4() {
		let program = String::from(
			r#";
struct Green {
  cell blue @2;
}
struct Green[3] g;
output '0' + sizeof(struct Green);
output '0' + sizeof(g);
output '0' + sizeof(g[2].blue)
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("391");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[ignore]
	#[test]
	fn sizeof_5() {
		let program = String::from(
			r#";
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
			"#,
		);
		let input = String::from("");
		let desired_output = String::from("23612");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn memory_specifiers_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell foo @3 = 2;
{
	cell n = 12;
	while n {
		n -= 1;
		foo += 10;
	}
}
output foo;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(code, ">>>++<<<++++++++++++[->>>++++++++++<<<][-]>>>.");
		assert_eq!(output, "z");
		Ok(())
	}

	#[test]
	fn memory_specifiers_2() -> Result<(), String> {
		let program = String::from(
			r#"
cell a @5 = 4;
cell foo @0 = 2;
cell b = 10;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(">>>>>++++<<<<<++>++++++++++"));
		Ok(())
	}

	#[test]
	fn memory_specifiers_3() -> Result<(), String> {
		let program = String::from(
			r#"
cell a @1 = 1;
cell foo @0 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(">+<++>>+++"));
		Ok(())
	}

	#[test]
	fn memory_specifiers_4() -> Result<(), String> {
		let program = String::from(
			r#"
cell a @1,2 = 1;
cell foo @0 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(">^^+<vv++>+++"));
		Ok(())
	}

	#[test]
	fn memory_specifiers_5() -> Result<(), String> {
		let program = String::from(
			r#"
cell[4][3] g @1,2;
g[0][0] = 1;
g[1][1] = 2;
g[2][2] = 3;
cell foo @0 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(">^^[-]+>>>>>[-]++>>>>>[-]+++<<<<<<<<<<<vv++>+++"));
		Ok(())
	}

	#[test]
	fn memory_specifiers_6() {
		let program = String::from(
			r#"
cell a @1 = 1;
cell foo @1 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None);
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @1,0 conflicts with another allocation"));
	}

	#[test]
	fn memory_specifiers_7() {
		let program = String::from(
			r#"
cell a @1,3 = 1;
cell foo @1,3 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None);
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @1,3 conflicts with another allocation"));
	}

	#[test]
	fn memory_specifiers_8() {
		let program = String::from(
			r#"
cell a @2 = 1;
cell foo @2,0 = 2;
cell b = 3;
"#,
		);
		let code = compile_program(program, None);
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @2,0 conflicts with another allocation"));
	}

	#[test]
	fn memory_specifiers_9() {
		let program = String::from(
			r#"
cell a @2,4 = 1;
cell[4] b @0,4;
"#,
		);
		let code = compile_program(program, None);
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @0,4 conflicts with another allocation"));
	}

	#[test]
	fn variable_location_specifiers_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell a = 'h';
bf @a {.}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("wxy");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "h");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_1a() -> Result<(), String> {
		let program = String::from(
			r#"
cell[100] _;
cell a = 'h';
cell[4] b;
bf @a {.}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "h");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_2() -> Result<(), String> {
		let program = String::from(
			r#"
struct Test {cell[3] a @0; cell b;}
struct Test t;
input *t.a;
bf @t.a {
[+.>]
}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("wxy");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(code, ",>,>,<<[+.>]");
		assert_eq!(output, "xyz");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_2a() -> Result<(), String> {
		let program = String::from(
			r#"
struct Test {cell[3] a @0; cell b;}
struct Test t;
input *t.a;
bf @t {
[+.>]
}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("wxy");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(code, ",>,>,<<[+.>]");
		assert_eq!(output, "xyz");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_3() -> Result<(), String> {
		let program = String::from(
			r#"
cell[5] f @6 = "abcde";
bf @f[2] clobbers *f {.+++.}
output 10;
output *f;
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "cf\nabfde");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_3a() -> Result<(), String> {
		let program = String::from(
			r#"
cell[4] f @8 = "xyz ";
bf @f {[.>]}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "xyz ");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_4() -> Result<(), String> {
		let program = String::from(
			r#"
fn func(cell g) {
  bf @g {+.-}
}

cell a = '5';
func(a);
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "6");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_4a() -> Result<(), String> {
		let program = String::from(
			r#"
fn func(cell g) {
  bf @g {+.-}
}

cell[3] a = "456";
func(a[1]);
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "6");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_4b() -> Result<(), String> {
		let program = String::from(
			r#"
fn func(cell g) {
  bf @g {+.-}
}

struct H {cell[3] r;}
struct H a;
a.r[0] = '4';
a.r[1] = '5';
a.r[2] = '6';
func(a.r[1]);
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "6");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_4c() -> Result<(), String> {
		let program = String::from(
			r#"
fn func(struct H h) {
  bf @h {+.-}
}

struct H {cell[3] r @0;}
struct H a;
a.r[0] = '4';
a.r[1] = '5';
a.r[2] = '6';
func(a);
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "5");
		Ok(())
	}

	#[test]
	fn variable_location_specifiers_4d() -> Result<(), String> {
		let program = String::from(
			r#"
fn func(cell[2] g) {
  bf @g {+.-}
}

struct J {cell[2] j;}
struct H {cell[20] a; struct J jj @1;}
struct H a;
a.jj.j[0] = '3';
a.jj.j[1] = '4';
func(a.jj.j);
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let output = run_code(BVM_CONFIG_1D, code.clone(), input, None);
		println!("{output}");
		assert_eq!(output, "4");
		Ok(())
	}

	#[test]
	fn assertions_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell a @0 = 5;
output a;
assert a equals 2;
a = 0;
output a;
"#,
		);
		let code = compile_program(program, Some(&OPT_ALL))?.to_string();
		println!("{code}");

		assert!(code.starts_with("+++++.--."));
		Ok(())
	}

	#[test]
	fn assertions_2() -> Result<(), String> {
		let program = String::from(
			r#"
cell a @0 = 2;
output a;
assert a unknown;
a = 0;
output a;
"#,
		);
		let code = compile_program(program, Some(&OPT_ALL))?.to_string();
		println!("{code}");

		assert!(code.starts_with("++.[-]."));
		Ok(())
	}

	#[test]
	fn inline_brainfuck_1() -> Result<(), String> {
		let program = String::from(
			r#"
bf {
	,.[-]
	+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.
}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert_eq!(
			code,
			",.[-]+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
		);

		let output = run_code(BVM_CONFIG_1D, code, String::from("~"), None);
		assert_eq!(output, "~Hello, World!");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_2() -> Result<(), String> {
		let program = String::from(
			r#"
// cell a @0;
// cell b @1;
bf @3 {
	,.[-]
	+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.
}
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(
			">>>,.[-]+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+."
		));

		let output = run_code(BVM_CONFIG_1D, code, String::from("~"), None);
		assert_eq!(output, "~Hello, World!");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_3() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert!(code.starts_with(",>,>,<<[+>]<<<[.[-]>]<<<"));

		let output = run_code(BVM_CONFIG_1D, code, String::from("HEY"), None);
		assert_eq!(output, "IFZ");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_4() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let output = run_code(BVM_CONFIG_1D, code, String::from("line of input\n"), None);
		assert_eq!(output, "lmijnoef !opfg !ijnopquvtu");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_5() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let output = run_code(BVM_CONFIG_1D, code, String::from("hello\n"), None);
		assert_eq!(output, "'h'\n'e'\n'l'\n'l'\n'o'\n");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_6() -> Result<(), String> {
		let program = String::from(
			r#"
cell b = 4;

bf {
	++--
	{
		output b;
	}
	++--
}
"#,
		);
		let result = compile_program(program, None);
		assert!(result.is_err());

		Ok(())
	}

	#[test]
	fn inline_brainfuck_7() -> Result<(), String> {
		let program = String::from(
			r#"
	bf {
		,>,>,
		<<
		{{{{{{cell g @5 = 1;}}}}}}
	}
	"#,
		);
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		assert_eq!(code, ",>,>,<<>>>>>+[-]<<<<<");
		Ok(())
	}
	#[test]
	fn inline_2d_brainfuck() -> Result<(), String> {
		let program = String::from(
			r#"
			bf {,.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+.}
		"#,
		);
		let code = compile_program(program, None)?.to_string();

		assert_eq!(
			code,
			",.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+."
		);

		let output = run_code(BVM_CONFIG_2D, code, String::from("~"), None);
		assert_eq!(output, "~Hello, World!");
		Ok(())
	}
	#[test]
	#[should_panic(expected = "Invalid Inline Brainfuck Characters in vvstvv")]
	fn invalid_inline_2d_brainfuck() {
		let program = String::from(
			r#"
			bf {,.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvstvv.+++.------.vv-.^^^^+.}
		"#,
		);
		let result = compile_program(program, None);
	}

	#[test]
	#[should_panic(expected = "2D Brainfuck currently disabled")]
	fn inline_2d_brainfuck_disabled() {
		run_code(
			BVM_CONFIG_1D,
			String::from(
				",.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+.",
			),
			String::from("~"),
			None,
		);
	}
	#[test]
	fn constant_optimisations_1() -> Result<(), String> {
		let program = String::from(
			"
output 'h';
      ",
		);
		let input = String::from("");
		let desired_output = String::from("h");

		let code = compile_program(program, Some(&OPT_ALL))?;
		println!("{}", code.clone().to_string());
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_1D, code.to_string(), input, None)
		);

		Ok(())
	}

	#[test]
	fn constant_optimisations_2() -> Result<(), String> {
		let program = String::from(
			r#"
cell[15] arr @1;
cell a = 'G';
cell b = a + 45;
output b;
b -= 43;
output b;
output a + 3;
      "#,
		);
		let input = String::from("");
		let desired_output = String::from("tIJ");

		let code = compile_program(program, Some(&OPT_ALL))?.to_string();
		println!("{}", code);
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, code, input, None));

		Ok(())
	}
	#[test]
	#[should_panic(expected = "Memory Allocation Method not implemented")]
	fn unimplemented_memory_allocation() {
		let program = String::from(
			r#"
			cell[15] arr @1;
			cell a = 'G';
			"#,
		);
		let cfg = MastermindConfig {
			optimise_generated_code: false,
			optimise_generated_all_permutations: false,
			optimise_cell_clearing: false,
			optimise_variable_usage: false,
			optimise_memory_allocation: false,
			optimise_unreachable_loops: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
			memory_allocation_method: 128,
			enable_2d_grid: false,
		};
		let code = compile_program(program, Some(&cfg));
	}
	#[test]
	fn tiles_memory_allocation_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
      "#,
		);
		let desired_output = String::from("+<v+^+^+>vv+^^+>vv+^+^+");

		let code = compile_program(program, Some(&OPT_NONE_TILES))?.to_string();
		assert_eq!(desired_output, code);

		Ok(())
	}
	#[test]
	fn tiles_memory_allocation_2() -> Result<(), String> {
		let program = String::from(
			r#"
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
      "#,
		);
		let input = String::from("");
		let desired_output = String::from("123456789");

		let code = compile_program(program, Some(&OPT_NONE_TILES))?.to_string();
		println!("{}", code);
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));

		Ok(())
	}

	#[test]
	fn tiles_memory_allocation_3() {
		let program = String::from(
			r#"
cell a @2,4 = 1;
cell[4] b @0,4;
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_TILES));
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @0,4 conflicts with another allocation"));
	}

	#[test]
	fn tiles_memory_allocation_4() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_TILES))?.to_string();
		println!("{}", code);
		let input = String::from("");
		let desired_output = String::from("12345");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));
		Ok(())
	}

	#[test]
	fn zig_zag_memory_allocation_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
      "#,
		);
		let desired_output = String::from("+>+<^+>>v+<^+<^+>>>vv+<^+<^+");

		let code = compile_program(program, Some(&OPT_NONE_ZIG_ZAG))?.to_string();
		assert_eq!(desired_output, code);

		Ok(())
	}
	#[test]
	fn zig_zag_memory_allocation_2() -> Result<(), String> {
		let program = String::from(
			r#"
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
      "#,
		);
		let input = String::from("");
		let desired_output = String::from("123456789");

		let code = compile_program(program, Some(&OPT_NONE_ZIG_ZAG))?.to_string();
		println!("{}", code);
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));

		Ok(())
	}

	#[test]
	fn zig_zag_memory_allocation_3() {
		let program = String::from(
			r#"
cell a @2,4 = 1;
cell[4] b @0,4;
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_ZIG_ZAG));
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @0,4 conflicts with another allocation"));
	}

	#[test]
	fn zig_zag_memory_allocation_4() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_ZIG_ZAG))?.to_string();
		println!("{}", code);
		let input = String::from("");
		let desired_output = String::from("12345");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));
		Ok(())
	}

	#[test]
	fn spiral_memory_allocation_1() -> Result<(), String> {
		let program = String::from(
			r#"
cell a = 1;
cell b = 1;
cell c = 1;
cell d = 1;
cell e = 1;
cell f = 1;
cell h = 1;
cell i = 1;
cell j = 1;
      "#,
		);
		let desired_output = String::from("^+>+v+<+<+^+^+>+>+");

		let code = compile_program(program, Some(&OPT_NONE_SPIRAL))?.to_string();
		assert_eq!(desired_output, code);

		Ok(())
	}
	#[test]
	fn spiral_memory_allocation_2() -> Result<(), String> {
		let program = String::from(
			r#"
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
      "#,
		);
		let input = String::from("");
		let desired_output = String::from("123456789");

		let code = compile_program(program, Some(&OPT_NONE_SPIRAL))?.to_string();
		println!("{}", code);
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));

		Ok(())
	}

	#[test]
	fn spiral_memory_allocation_3() {
		let program = String::from(
			r#"
cell a @2,4 = 1;
cell[4] b @0,4;
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_SPIRAL));
		assert!(code.is_err());
		assert!(code
			.unwrap_err()
			.to_string()
			.contains("Location specifier @0,4 conflicts with another allocation"));
	}

	#[test]
	fn spiral_memory_allocation_4() -> Result<(), String> {
		let program = String::from(
			r#"
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
"#,
		);
		let code = compile_program(program, Some(&OPT_NONE_SPIRAL))?.to_string();
		println!("{}", code);
		let input = String::from("");
		let desired_output = String::from("12345");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, code, input, None));
		Ok(())
	}
}
