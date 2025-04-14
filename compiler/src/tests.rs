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
		optimise_cell_clearing: false,
		optimise_variable_usage: false,
		optimise_memory_allocation: false,
		optimise_unreachable_loops: false,
		optimise_constants: false,
		optimise_empty_blocks: false,
	};

	const OPT_ALL: MastermindConfig = MastermindConfig {
		optimise_generated_code: true,
		optimise_cell_clearing: true,
		optimise_variable_usage: true,
		optimise_memory_allocation: true,
		optimise_unreachable_loops: true,
		optimise_constants: true,
		optimise_empty_blocks: true,
	};

	const BVM_CONFIG_1D: BVMConfig = BVMConfig {
		ENABLE_DEBUG_SYMBOLS: false,
		ENABLE_2D_GRID: false,
	};

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
		Ok(run_code(BVM_CONFIG_1D, bfs, input))
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
		let output = run_code(BVM_CONFIG_1D, code, input);
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

def func_0<grape> {
	cell n = grape + 1;
	output n;
	n = 0;
};;

def func_1<grape> {
	cell n = grape + 2;
	output n;
	n = 0;
}

output global_var;
func_0<global_var>;
output global_var;

global_var += 1;;;
output global_var;
;;func_1<global_var>;
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

def func_0<grape> {
	cell n = grape + 1;
	output n;

	def func_1<grape> {
		grape += 1;
		output grape;
		grape += 1;
	};

	func_1<n>;
	output n;

	grape += 1;
};

output global_var;
func_0<global_var>;
output global_var;

output 10;
		",
		);
		let input = String::from("");
		let desired_output = String::from("01231\n");
		let code = compile_program(program, Some(&OPT_NONE))?.to_string();
		println!("{}", code);
		let output = run_code(BVM_CONFIG_1D, code, input);
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

def func_0<grape> {
	cell n = grape + 1;
	output n;

	def func_1<grape> {
		grape += 1;
		output grape;
		grape += 1;

		cell[4] frog;
		cell zero = '0';
		drain zero into *frog;
		frog[1] += 2;

		zero = grape + 3;
		func_2<frog, zero>;
		output zero;
	};

	func_1<n>;
	output n;

	grape += 1;
};

output global_var;
func_0<global_var>;
output global_var;

output 10;

output global_vars[1];
func_0<global_vars[0]>;
output global_vars[0];

output 10;

def func_2<think[4], green> {
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
	fn functions_4() {
		let program = String::from(
			r#"
def hello<> {
	output "hello";
}

hello<>;
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

def inc<h, g> {
	g += 1;
	if h {h += 1;} else {h = 'Z';}
}

output *b;
inc<b[1], b[2]>;
output *b;

output 10;

cell c = -1;
inc<c, c>;
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

def drain_h<cell h> {
	drain h {
		output 'h';
	}
}

drain_h<b[2]>;
drain_h<b[2]>;
output ' ';
drain_h<b[1]>;
output ' ';

def drain_into<cell a, cell[5] b> {
	drain a into *b;
}

cell u = 'a' - 1;
cell[5] v = [8, 5, 12, 12, 15];
drain_into<u, v>;
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

g[3][0] = 1 + '0';
g[3][1] = 2 + '0';
g[3][2] = 3 + '0';

g[2][0] = 0 + '0';
g[2][1] = 0 + '0';
g[2][2] = 0 + '0';

output g[0][0];
output g[0][1];
output g[0][2];
output g[1][0];
output g[1][1];
output g[1][2];
output g[2][0];
output g[2][1];
output g[2][2];
output g[3][0];
output g[3][1];
output g[3][2];
"#,
		);
		let input = String::from("");
		let desired_output = String::from("543123000123");
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
		let output = run_code(BVM_CONFIG_1D, code.clone(), input);
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

		let output = run_code(BVM_CONFIG_1D, code, String::from("~"));
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

		let output = run_code(BVM_CONFIG_1D, code, String::from("~"));
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

		let output = run_code(BVM_CONFIG_1D, code, String::from("HEY"));
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

		let output = run_code(BVM_CONFIG_1D, code, String::from("line of input\n"));
		assert_eq!(output, "lmijnoef !opfg !ijnopquvtu");
		Ok(())
	}

	#[test]
	fn inline_brainfuck_5() -> Result<(), String> {
		let program = String::from(
			r#"
// external function within the same file, could be tricky to implement
def quote<n> {
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
			quote<chr>;
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

		let output = run_code(BVM_CONFIG_1D, code, String::from("hello\n"));
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
			run_code(BVM_CONFIG_1D, code.to_string(), input)
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
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, code, input));

		Ok(())
	}
}
