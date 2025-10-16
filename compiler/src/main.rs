#![allow(dead_code)]
// dead code is allowed because we have two different compile targets (wasm and command-line)

// project dependencies:
mod backend;
mod brainfuck;
mod brainfuck_optimiser;
mod cells;
mod constants_optimiser;
mod frontend;
mod macros;
mod misc;
mod parser;
mod preprocessor;
mod tests;
mod tokeniser;
use crate::{
	backend::BrainfuckOpcodes,
	brainfuck::{BrainfuckConfig, BrainfuckContext},
	cells::{TapeCell, TapeCell2D},
	misc::{MastermindConfig, MastermindContext},
	parser::parse,
	preprocessor::preprocess,
	tokeniser::tokenise,
};

// stdlib dependencies:
use std::io::{stdin, stdout, Cursor};

// external dependencies:
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

	let ctx = MastermindContext {
		// TODO: change this to not be a bitmask, or at least document it
		config: MastermindConfig::new(args.optimise),
	};

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
			let bf_code = if ctx.config.enable_2d_grid {
				let parsed_syntax = parse::<TapeCell2D>(&tokens)?;
				let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
				ctx.ir_to_bf(instructions, None)?
			} else {
				let parsed_syntax = parse::<TapeCell>(&tokens)?;
				let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
				ctx.ir_to_bf(instructions, None)?
			};

			match ctx.config.optimise_generated_code {
				true => ctx.optimise_bf_code(bf_code).to_string(),
				false => bf_code.to_string(),
			}
		}
		false => program,
	};

	if args.run || !args.compile {
		// run brainfuck
		let ctx = BrainfuckContext {
			config: BrainfuckConfig {
				enable_debug_symbols: false,
				enable_2d_grid: false,
			},
		};

		if args.input.is_some() {
			ctx.run(
				bf_program.chars().collect(),
				&mut Cursor::new(args.input.unwrap()),
				&mut stdout(),
				None,
			)?;
		} else {
			ctx.run(
				bf_program.chars().collect(),
				&mut stdin(),
				&mut stdout(),
				None,
			)?;
		}
	} else {
		print!("{bf_program}");
	}

	Ok(())
}
