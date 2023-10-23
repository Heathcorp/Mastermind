mod brainfuck;
mod compiler;
mod optimiser;
mod parser;
mod tokeniser;

use brainfuck::BVM;
use compiler::MastermindCompiler;
use optimiser::BrainfuckOptimiser;
use parser::MastermindParser;
use tokeniser::MastermindTokeniser;

use std::io::{stdin, stdout, Cursor};

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author = "Heathcorp", version = "0.1", about = "Brainfuck interpreter and compiler", long_about = None)]
struct Arguments {
	#[arg(short, long)]
	file: Option<String>,
	#[arg(short, long)]
	program: Option<String>,
	#[arg(short, long)]
	input: Option<String>,
	#[arg(short, long, default_value_t = false)]
	compile: bool,
	#[arg(short, long, default_value_t = false)]
	run: bool,
}

fn main() {
	std::env::set_var("RUST_BACKTRACE", "1");

	let args = Arguments::parse();

	let program = match args.file {
		Some(file) => std::fs::read_to_string(file).unwrap(),
		None => args.program.unwrap(),
	};

	let bf_program = match args.compile {
		true => {
			// compile the provided file

			// TODO: tokenise properly so we don't need to worry about lines
			let tokeniser = MastermindTokeniser;
			let tokenised_lines = tokeniser.tokenise(&program);
			// parse tokens into syntax tree
			let mut parser = MastermindParser;
			let parsed_program = parser.parse(tokenised_lines);
			// println!("{parsed_program:#?}");
			// compile syntax tree into brainfuck
			let mut compiler = MastermindCompiler::new();
			compiler.compile(parsed_program);

			BrainfuckOptimiser::optimise(compiler.program)
			// compiler.to_string()
		}
		false => program,
	};

	if args.run || !args.compile {
		// run brainfuck
		let mut bvm = BVM::new(bf_program.chars().collect());

		if args.input.is_some() {
			bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout());
		} else {
			bvm.run(&mut stdin(), &mut stdout());
		}
	} else {
		print!("{bf_program}");
	}
}
