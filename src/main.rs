mod brainfuck;
mod construction;
mod mastermind;

use brainfuck::BVM;
use mastermind::MastermindCompiler;

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

	let program = if args.file.is_some() {
		std::fs::read_to_string(args.file.unwrap()).unwrap()
	} else if args.program.is_some() {
		args.program.unwrap()
	} else {
		String::new()
	};

	let bf_program = match args.compile {
		true => {
			// run the compiler on the provided file
			let mut mfc = MastermindCompiler::new();
			mfc.compile(program)
		}
		false => program,
	};

	if args.run {
		// run brainfuck
		let mut bvm = BVM::new(bf_program.chars().collect());

		if args.input.is_some() {
			bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout());
		} else {
			bvm.run(&mut stdin(), &mut stdout());
		}
	}
}
