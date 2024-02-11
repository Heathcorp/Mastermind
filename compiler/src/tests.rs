#![allow(dead_code)]

// black box testing
#[cfg(test)]
pub mod tests {
	use crate::{
		brainfuck::tests::run_program,
		builder::{BrainfuckProgram, Builder, Opcode},
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

	fn compile_and_run(program: String, input: String) -> Result<String, String> {
		// println!("{program}");
		// compile mastermind
		let tokens: Vec<Token> = tokenise(&program)?;
		// println!("{tokens:#?}");
		let clauses = parse(&tokens)?;
		// println!("{clauses:#?}");
		let instructions = Compiler { config: &OPT_NONE }
			.compile(&clauses, None)?
			.get_instructions();
		// println!("{instructions:#?}");
		let bf_program = Builder { config: &OPT_NONE }.build(instructions)?;
		let bfs = bf_program.to_string();
		// println!("{}", bfs);
		// run generated brainfuck with input
		Ok(run_program(bfs, input))
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
		.get_instructions();
		// println!("{instructions:#?}");
		let bf_code = Builder {
			config: config.unwrap_or(&OPT_NONE),
		}
		.build(instructions)?;
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
		let output = run_program(code, input);
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn hello_1() {
		let program = String::from(
			"
let h = 8;
let e = 5;
let l = 12;
let o = 15;
// comment!
let a_char = 96;
drain a_char into h e l o;
output h;
output e;
output l;
output l;
output o;
let ten = 10;
output ten;
      ",
		);
		let input = String::from("");
		let desired_output = String::from("hello\n");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
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
let EEL[5] =    "ello\n";
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
let str[4] = [5, 12, 12, 15];
let a = 'a' - 1;
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
let p = 9 - (true + true -(-7));
if not p {
	output "Hi friend!\n";
}

let q = 8 + p - (4 + p);
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

let not_a = 'a' + (-1) - (0 - 1);
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
let x = 5;
let A = 'A';

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
let n = '0';
let a = 10;
let b = 1;
drain a {
	output n;
	++n;
	output 'A';
	let c = b;
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
let a = 4;
let b[6] = [65, 65, 65, 65, 65, 1];
copy a into b[0] b[1] b[4] b[5] {
	copy b[5] into b[2];
	
	output b[0];
	output b[1];
	output b[2];
	output b[3];
	output b[4];
	output 10;
}a+='a';output a;

let g = 5;
drain g into a {output a;}
      ",
		);
		let input = String::from("");
		let desired_output = String::from("AABAA\nBBDAB\nCCGAC\nDDKAD\neefghi");
		assert_eq!(desired_output, compile_and_run(program, input).expect(""))
	}

	#[test]
	fn ifs_1() {
		let program = String::from(
			"
let x = 7;
let y = 9;

let z = x - y;
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
let x = 7;
let y = 9;

let z = x - y;
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
let a = 5;
if a {
	let b = a + '0';
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
let n = '0';
let a = 6;
let b;
drain a {
	output n;++n;
;;;;;;
	output 'A';

	let c;
	let nt_eq = a - b;

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
let global_var = '0';

def func_0<grape> {
	let n = grape + 1;
	output n;
	n = 0;
};;

def func_1<grape> {
	let n = grape + 2;
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
	fn functions_2() {
		let program = String::from(
			"
let global_var = '0';

def func_0<grape> {
	let n = grape + 1;
	output n;

	def func_1<grape> {
		grape += 1;
		output grape;
		grape += 1;
	};

	func_1<n>;
	output n;

	grape += 1;
	n = 0;
};

output global_var;
func_0<global_var>;
output global_var;

output 10;
		",
		);
		let input = String::from("");
		let desired_output = String::from("01231\n");
		let output = compile_and_run(program, input).expect("");
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3() {
		let program = String::from(
			"
let global_var = '0';

let global_vars[2] = ['0', 64];

def func_0<grape> {
	let n = grape + 1;
	output n;

	def func_1<grape> {
		grape += 1;
		output grape;
		grape += 1;

		let frog[4];
		let zero = '0';
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
	// let green = '$';
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
	fn input_1() {
		let program = String::from(
			"
let b;
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
let b[3];
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
let b[3] = "Foo";

def inc<h, g> {
	g += 1;
	if h {h += 1;} else {h = 'Z';}
}

output *b;
inc<b[1], b[2]>;
output *b;

output 10;

let c = -1;
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
let b[3] = [1, 2, 3];

def drain_h<h> {
	drain h {
		output 'h';
	}
}

drain_h<b[2]>;
drain_h<b[2]>;
output ' ';
drain_h<b[1]>;
output ' ';

def drain_into<a, b[5]> {
	drain a into *b;
}

let u = 'a' - 1;
let v[5] = [8, 5, 12, 12, 15];
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
	let g = 0 + 5 + (-(-5));
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
let f = 'f';
output f;
{
	let f = 'F';
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
	fn memory_specifiers_1() -> Result<(), String> {
		let program = String::from(
			r#"
let foo @3 = 2;
{
	let n = 12;
	while n {
		n -= 1;
		foo += 10;
	}
}
output foo;
"#,
		);
		let desired_code = String::from(">>>++<<<++++++++++++[->>>++++++++++<<<][-]>>>.[-]");
		let code = compile_program(program, None)?.to_string();
		println!("{code}");

		let input = String::from("");
		let desired_output = String::from("z");
		let output = run_program(code.clone(), input);
		println!("{output}");
		assert_eq!(desired_code, code);
		assert_eq!(desired_output, output);
		Ok(())
	}

	#[test]
	fn memory_specifiers_2() -> Result<(), String> {
		let program = String::from(
			r#"
let a @5 = 4;
let foo @0 = 2;
let b = 10;
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
let a @1 = 1;
let foo @0 = 2;
let b = 3;
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
let a @0 = 5;
output a;
assert a = 2;
a = 0;
output a;
"#,
		);
		let code = compile_program(program, Some(&OPT_ALL))?.to_string();
		println!("{code}");

		assert!(code.starts_with("+++++.--."));
		Ok(())
	}
}
