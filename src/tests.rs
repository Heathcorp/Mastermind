// black box testing
#[cfg(test)]
pub mod tests {
	use crate::{
		brainfuck::tests::run_program, compiler::MastermindCompiler, optimiser::BrainfuckOptimiser,
		parser::MastermindParser, tokeniser::MastermindTokeniser,
	};

	fn compile_and_run(program: String, input: String) -> String {
		// compile mastermind
		let mut compiler = MastermindCompiler::new();
		compiler.compile(MastermindParser.parse(MastermindTokeniser.tokenise(&program)));
		let compiled_code = BrainfuckOptimiser::optimise(compiler.program);
		println!("{compiled_code}");
		// run generated brainfuck with input
		run_program(compiled_code, input)
	}

	#[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		assert_eq!(desired_output, compile_and_run(program, input))
	}

	#[test]
	fn hello_1() {
		let program = String::from(
			"
int[1] h 8
int[1] e 5
int[1] l 12
int[1] o 15
// comment!
int[1] a_char 96
loop a_char
{
  add h 1
  add e 1
  add l 1
  add o 1
}
free a_char
output h
output e
output l
output l
output o
int[1] ten 10
output ten
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
int[1] h 8
int[1] e 5
int[1] l 12
int[1] o 15
// comment!
int[1] a_char 96
copy h a_char
copy e a_char
copy l a_char
drain o a_char
free a_char
output h
output e
output l
output l
output o
int[1] ten 10
output ten
      ",
		);
		let input = String::from("");
		let desired_output = String::from("hello\n");
		assert_eq!(desired_output, compile_and_run(program, input))
	}
}
