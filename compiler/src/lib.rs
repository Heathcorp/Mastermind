#![allow(dead_code)]

mod macros;

// allowing dead code because we have two different compile targets (wasm and command-line)
mod brainfuck;
mod brainfuck_optimiser;
mod builder;
mod compiler;
mod misc;
mod parser;
mod preprocessor;
mod tokeniser;

use brainfuck::BVM;
use brainfuck_optimiser::optimise;
use builder::{BrainfuckProgram, Builder};
use compiler::Compiler;
use misc::MastermindConfig;
use parser::parse;
use preprocessor::preprocess_from_memory;
use tokeniser::tokenise;

use std::collections::HashMap;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

// copied from rustwasm.github.io
pub fn set_panic_hook() {
	// When the `console_error_panic_hook` feature is enabled, we can call the
	// `set_panic_hook` function at least once during initialization, and then
	// we will get better error messages if our code ever panics.
	//
	// For more details see
	// https://github.com/rustwasm/console_error_panic_hook#readme
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
	let compiler = Compiler { config: &config };
	let builder = Builder { config: &config };

	let preprocessed_file = preprocess_from_memory(&file_contents, entry_file_name)?;
	let tokens = tokenise(&preprocessed_file)?;
	let parsed = parse(&tokens)?;
	let instructions = compiler.compile(&parsed, None)?;
	let bf_code = builder.build(instructions.finalise_instructions(false), false)?;

	Ok(match config.optimise_generated_code {
		true => optimise(bf_code).to_string(),
		false => bf_code.to_string(),
	})
}

#[wasm_bindgen]
pub async fn wasm_run_bf(
	code: String,
	output_callback: &js_sys::Function,
	input_callback: &js_sys::Function,
) -> Result<String, JsValue> {
	set_panic_hook();

	let mut bf = BVM::new(code.chars().collect());

	// hack, TODO: refactor
	let r = bf.run_async(output_callback, input_callback).await?;

	Ok(r)
}
