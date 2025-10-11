#![allow(dead_code)]

mod macros;

// allowing dead code because we have two different compile targets (wasm and command-line)
mod backend;
mod brainfuck;
mod brainfuck_optimiser;
mod constants_optimiser;
mod frontend;
mod misc;
mod parser;
mod preprocessor;
mod tokeniser;

mod tests;

use backend::BrainfuckOpcodes;
use brainfuck::{BVMConfig, BVM};
use brainfuck_optimiser::optimise;
use misc::MastermindConfig;
use parser::parse;
use preprocessor::preprocess_from_memory;
use tokeniser::tokenise;

use std::collections::HashMap;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

use crate::misc::MastermindContext;

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
	let config: MastermindConfig = serde_wasm_bindgen::from_value(config).unwrap();
	let ctx = MastermindContext { config: &config };

	let preprocessed_file = preprocess_from_memory(&file_contents, entry_file_name)?;
	let tokens = tokenise(&preprocessed_file)?;
	let parsed_syntax = parse(&tokens)?;
	let instructions = ctx.create_ir_scope(&parsed_syntax, None)?.build_ir(false);
	let bf_code = ctx.ir_to_bf(instructions, false)?;

	Ok(match config.optimise_generated_code {
		true => optimise(bf_code, config.optimise_generated_all_permutations).to_string(),
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

	let config = BVMConfig {
		enable_debug_symbols: false,
		enable_2d_grid: enable_2d_grid,
	};
	let mut bf = BVM::new(config, code.chars().collect());

	// hack, TODO: refactor
	let r = bf.run_async(output_callback, input_callback).await?;

	Ok(r)
}
