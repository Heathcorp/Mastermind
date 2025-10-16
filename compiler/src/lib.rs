#![allow(dead_code)]
// dead code is allowed because we have two different compile targets (wasm and command-line)

// crate dependencies:
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
	misc::MastermindContext,
	parser::parse,
	preprocessor::preprocess_from_memory,
	tokeniser::tokenise,
};

// stdlib dependencies:
use std::collections::HashMap;

// external dependencies:
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub fn set_panic_hook() {
	// copied from rustwasm.github.io
	// https://github.com/rustwasm/console_error_panic_hook
	#[cfg(feature = "console_error_panic_hook")]
	console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn wasm_compile(
	file_contents: JsValue,
	entry_file_name: String,
	config: JsValue,
) -> Result<String, String> {
	set_panic_hook();

	let file_contents: HashMap<String, String> =
		serde_wasm_bindgen::from_value(file_contents).unwrap();
	let ctx = MastermindContext {
		config: serde_wasm_bindgen::from_value(config).unwrap(),
	};

	let preprocessed_file = preprocess_from_memory(&file_contents, entry_file_name)?;
	let tokens = tokenise(&preprocessed_file)?;
	let bf_code = if ctx.config.enable_2d_grid {
		let parsed_syntax = parse::<TapeCell2D>(&tokens)?;
		let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
		ctx.ir_to_bf(instructions, None)?
	} else {
		let parsed_syntax = parse::<TapeCell>(&tokens)?;
		let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
		ctx.ir_to_bf(instructions, None)?
	};

	Ok(match ctx.config.optimise_generated_code {
		true => ctx.optimise_bf_code(bf_code).to_string(),
		false => bf_code.to_string(),
	})
}

#[wasm_bindgen]
pub async fn wasm_run_bf(
	code: String,
	enable_2d_grid: bool,
	output_callback: &js_sys::Function,
	input_callback: &js_sys::Function,
) -> Result<String, JsValue> {
	set_panic_hook();

	let ctx = BrainfuckContext {
		config: BrainfuckConfig {
			enable_debug_symbols: false,
			enable_2d_grid: enable_2d_grid,
		},
	};

	let r = ctx
		.run_async(code.chars().collect(), output_callback, input_callback)
		.await?;

	Ok(r)
}
