#![allow(dead_code)]

mod macros;

mod backend;
mod brainfuck;
mod brainfuck_optimiser;
mod constants_optimiser;
mod frontend;
mod parser;
mod preprocessor;
mod tokeniser;

mod misc;
mod tests;

use backend::BrainfuckOpcodes;
use brainfuck::{BVMConfig, BVM};
use misc::MastermindConfig;
use parser::parse;
use preprocessor::preprocess;
use tokeniser::tokenise;

use std::io::{stdin, stdout, Cursor};

use clap::Parser;

use crate::misc::MastermindContext;

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
			// parse tokens into syntax tree
			let clauses = parse(&tokens)?;
			// compile syntax tree into brainfuck

			// 2 stage compilation step, first stage compiles syntax tree into low-level instructions
			// 	second stage translates the low-level instructions into brainfuck

			let instructions = ctx.create_ir_scope(&clauses, None)?.build_ir(false);
			let bf_code = ctx.ir_to_bf(instructions, None)?;

			match ctx.config.optimise_generated_code {
				true => ctx.optimise_bf_code(bf_code).to_string(),
				false => bf_code.to_string(),
			}
		}
		false => program,
	};

	if args.run || !args.compile {
		// run brainfuck
		let config = BVMConfig {
			enable_debug_symbols: false,
			enable_2d_grid: false,
		};
		let mut bvm = BVM::new(config, bf_program.chars().collect());

		if args.input.is_some() {
			bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout(), None)?;
		} else {
			bvm.run(&mut stdin(), &mut stdout(), None)?;
		}
	} else {
		print!("{bf_program}");
	}

	Ok(())
}
