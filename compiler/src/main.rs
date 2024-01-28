#![allow(dead_code)]

mod macros;

// Stages: (rust format has jumbled these)
mod brainfuck; // 6. Run
mod builder; // 4. Build (and pre-optimise)
mod compiler; // 3. Compile
mod optimiser; // 5. Post-Optimise
mod parser; // 2. Parse
mod preprocessor; // 0. Preprocess includes and macro-type stuff
mod tokeniser; // 1. Tokenise

mod misc;
mod tests;

use brainfuck::BVM;
use builder::{BrainfuckProgram, Builder};
use compiler::Compiler;
use misc::MastermindConfig;
use optimiser::optimise;
use parser::parse;
use preprocessor::preprocess;
use tokeniser::tokenise;

use std::io::{stdin, stdout, Cursor};

use clap::Parser;

#[derive(Parser, Default, Debug)]
#[command(author = "Heathcorp", version = "0.1", about = "Mastermind: the Brainfuck interpreter and compilation tool", long_about = None)]
struct Arguments {
	#[arg(short, long, help = "provide a file to read a program from")]
	file: Option<String>,

	#[arg(short, long, help = "provide a program via command line arguments")]
	program: Option<String>,

	#[arg(
		short,
		long,
		help = "provide input to the Brainfuck VM if running, stdin will be used if not provided"
	)]
	input: Option<String>,

	#[arg(
		short,
		long,
		default_value_t = false,
		help = "compile the provided program to Brainfuck"
	)]
	compile: bool,

	#[arg(
		short,
		long,
		default_value_t = false,
		help = "run the compiled or provided Brainfuck code"
	)]
	run: bool,

	#[arg(
		short,
		long,
		default_value_t = 0,
		help = "specify the level of optimisation, this is a bitmask value"
	)]
	optimise: usize,
}

fn main() -> Result<(), String> {
	std::env::set_var("RUST_BACKTRACE", "1");

	let args = Arguments::parse();

	let config = MastermindConfig::new(args.optimise);

	let program;
	match args.file {
		Some(file) => {
			let file_path = std::path::PathBuf::from(file);

			// c-style preprocessor (includes and maybe some simple conditionals to avoid double includes)
			program = preprocess(file_path);
		}
		None => {
			program = args.program.unwrap();
		}
	};

	let bf_program = match args.compile {
		true => {
			// compile the provided file

			let tokens = tokenise(&program)?;
			// parse tokens into syntax tree
			let clauses = parse(&tokens)?;
			// compile syntax tree into brainfuck

			// 2 stage compilation step, first stage compiles syntax tree into low-level instructions
			// 	second stage actually writes out the low-level instructions into brainfuck

			let compiler = Compiler { config: &config };
			let instructions = compiler.compile(&clauses, None)?.get_instructions();

			let builder = Builder { config: &config };
			let bf_program = builder.build(instructions)?;

			match config.optimise_generated_code {
				true => optimise(bf_program).to_string(),
				false => bf_program.to_string(),
			}
		}
		false => program,
	};

	if args.run || !args.compile {
		// run brainfuck
		let mut bvm = BVM::new(bf_program.chars().collect());

		if args.input.is_some() {
			bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout())?;
		} else {
			bvm.run(&mut stdin(), &mut stdout())?;
		}
	} else {
		print!("{bf_program}");
	}

	Ok(())
}
