#![allow(dead_code)]
// dead code is allowed because we have two different compile targets (wasm and command-line)

// project dependencies:
mod backend;
mod brainfuck;
mod frontend;
#[macro_use]
mod macros;
mod misc;
mod parser;
mod preprocessor;
mod tests;
use crate::{
	backend::{
		bf::{Opcode, TapeCell},
		bf2d::{Opcode2D, TapeCell2D},
		common::BrainfuckProgram,
	},
	brainfuck::{BrainfuckConfig, BrainfuckContext},
	misc::{MastermindConfig, MastermindContext},
	parser::parser::parse_program,
	preprocessor::{preprocess, strip_comments},
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

	#[arg(short = 'o', long, default_value_t = true)]
	optimize_all: bool,

	#[arg(long, default_value_t = false)]
	optimize_generated_code: bool,

	#[arg(long, default_value_t = false)]
	optimize_generated_all_permutations: bool,

	#[arg(long, default_value_t = false)]
	optimize_cell_clearing: bool,

	#[arg(long, default_value_t = false)]
	optimize_unreachable_loops: bool,

	#[arg(long, default_value_t = false)]
	optimize_constants: bool,

	#[arg(long, default_value_t = false)]
	optimize_empty_blocks: bool,

	#[arg(short, long, default_value_t = false)]
	debug: bool,
}

fn main() -> Result<(), String> {
	// TODO: clean up this crazy file, this was the first ever rust I wrote and it's messy
	std::env::set_var("RUST_BACKTRACE", "1");

	let args = Arguments::parse();

	let ctx = MastermindContext {
		// TODO: change this to not be a bitmask, or at least document it
		config: MastermindConfig {
			optimise_generated_code: args.optimize_generated_code || args.optimize_all,
			optimise_generated_all_permutations: args.optimize_generated_all_permutations
				|| args.optimize_all,
			optimise_cell_clearing: args.optimize_cell_clearing || args.optimize_all,
			optimise_unreachable_loops: args.optimize_unreachable_loops || args.optimize_all,
			optimise_constants: args.optimize_constants || args.optimize_all,
			optimise_empty_blocks: args.optimize_empty_blocks || args.optimize_all,
			enable_2d_grid: false,
			memory_allocation_method: 0,
		},
	};

	let program = match args.file {
		Some(file) => {
			let file_path = std::path::PathBuf::from(file);

			// c-style preprocessor (includes and maybe some simple conditionals to avoid double includes)
			preprocess(file_path)
		}
		None => args.program.unwrap(),
	};

	let bf_program = match args.compile {
		true => {
			let stripped_program = strip_comments(&program);
			// compile the provided file
			if ctx.config.enable_2d_grid {
				let parsed_syntax = parse_program::<TapeCell2D, Opcode2D>(&stripped_program)?;
				if args.debug {
					println!("AST:");
					println!("{parsed_syntax:#?}");
				}
				let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
				if args.debug {
					println!("IR:");
					println!("{instructions:#?}");
				}
				let bf_code = ctx.ir_to_bf(instructions, None)?;
				if args.debug && !args.run {
					println!("BF:");
					println!("{:#?}", bf_code.clone().to_string());
				}
				match ctx.config.optimise_generated_code {
					true => ctx.optimise_bf2d(bf_code),
					false => bf_code,
				}
				.to_string()
			} else {
				let parsed_syntax = parse_program::<TapeCell, Opcode>(&stripped_program)?;
				let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
				let bf_code = ctx.ir_to_bf(instructions, None)?;
				match ctx.config.optimise_generated_code {
					true => ctx.optimise_bf(bf_code),
					false => bf_code,
				}
				.to_string()
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
