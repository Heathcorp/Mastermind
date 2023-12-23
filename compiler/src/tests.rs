#![allow(dead_code)]
// black box testing
#[cfg(test)]
pub mod tests {
	use crate::{
		brainfuck::tests::run_program,
		builder::Builder,
		compiler::Compiler,
		parser::parse,
		tokeniser::{tokenise, Token},
		MastermindConfig,
	};

	fn compile_and_run(program: String, input: String) -> String {
		println!("{program}");
		let config = MastermindConfig {
			optimise_generated_code: false,
			optimise_cell_clearing: false,
			optimise_variable_usage: false,
			optimise_memory_allocation: false,
			optimise_unreachable_loops: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
		};
		// compile mastermind
		let tokens: Vec<Token> = tokenise(&program);
		println!("{tokens:#?}");
		let clauses = parse(&tokens);
		println!("{clauses:#?}");
		let instructions = Compiler { config: &config }
			.compile(&clauses, None)
			.get_instructions();
		println!("{instructions:#?}");
		let bf_program = Builder { config: &config }.build(instructions);
		println!("{bf_program}");
		// run generated brainfuck with input
		run_program(bf_program, input)
	}

	// #[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		let output = compile_and_run(program, input);
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
		assert_eq!(desired_output, compile_and_run(program, input))
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
		assert_eq!(desired_output, compile_and_run(program, input))
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
		let output = compile_and_run(program, input);
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
		assert_eq!(desired_output, compile_and_run(program, input))
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
		assert_eq!(desired_output, compile_and_run(program, input))
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
		let output = compile_and_run(program, input);
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
	};
	nt_eq = 0;

	drain c {output 'B';};

	b += 1;
	output 10;
};
      ",
		);
		let input = String::from("");
		let desired_output = String::from("0ABB\n1ABB\n2ABB\n3ABBBBBBBBBB\n4ABB\n5ABB\n");
		let output = compile_and_run(program, input);
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_1() {
		let program = String::from(
			"
let global_var = '0';

def func_0(grape) {
	let n = grape + 1;
	output n;
	n = 0;
};;

def func_1(grape) {
	let n = grape + 2;
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
		let output = compile_and_run(program, input);
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_2() {
		let program = String::from(
			"
let global_var = '0';

def func_0(grape) {
	let n = grape + 1;
	output n;

	def func_1(grape) {
		grape += 1;
		output grape;
		grape += 1;
	};

	func_1(n);
	output n;

	grape += 1;
	n = 0;
};

output global_var;
func_0(global_var);
output global_var;

output 10;
		",
		);
		let input = String::from("");
		let desired_output = String::from("01231\n");
		let output = compile_and_run(program, input);
		println!("{output}");
		assert_eq!(desired_output, output)
	}

	#[test]
	fn functions_3() {
		let program = String::from(
			"
let global_var = '0';

def func_0(grape) {
	let n = grape + 1;
	output n;

	def func_1(grape) {
		grape += 1;
		output grape;
		grape += 1;

		let frog[4];
		let zero = '0';
		drain zero into frog[0] frog[1] frog[2] frog[3];
		frog[1] += 2;

		zero = grape + 3;
		func_2(frog, zero);
		output zero;

		frog[0] = 0;
		frog[1] = 0;
		frog[2] = 0;
		frog[3] = 0;
		zero = 0;
	};

	func_1(n);
	output n;

	grape += 1;
	n = 0;
};

output global_var;
func_0(global_var);
output global_var;

output 10;

def func_2(think[4], green) {
	think[2] += 7;
	think[3] += 2;

	output think[0];
	output think[1];
	output think[2];
	output think[3];

	output green;
	let green = '$';
	output green;
	green = 0;
};
		",
		);
		let input = String::from("");
		let desired_output = String::from("01202726$631\n");
		let output = compile_and_run(program, input);
		println!("{output}");
		assert_eq!(desired_output, output)
	}
}
