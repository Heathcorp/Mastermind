// allowing dead code because we have two different compile targets (wasm and command-line)
#![allow(dead_code)]
mod brainfuck;
mod builder;
mod compiler;
mod misc;
mod optimiser;
mod parser;
mod preprocessor;
mod tokeniser;

use brainfuck::BVM;
use builder::Builder;
use compiler::Compiler;
use misc::MastermindConfig;
use optimiser::optimise;
use parser::parse;
use preprocessor::preprocess_from_memory;
use tokeniser::tokenise;

use std::{collections::HashMap, io::Cursor};

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
	pub fn alert(s: &str);
}

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
pub fn wasm_compile(file_contents: JsValue, entry_file_name: String, config: JsValue) -> String {
	set_panic_hook();

	let file_contents: HashMap<String, String> =
		serde_wasm_bindgen::from_value(file_contents).unwrap();
	let config: MastermindConfig = serde_wasm_bindgen::from_value(config).unwrap();
	let compiler = Compiler { config: &config };
	let builder = Builder { config: &config };

	let preprocessed_file = preprocess_from_memory(&file_contents, entry_file_name);
	let tokens = tokenise(&preprocessed_file);
	let parsed = parse(&tokens);
	let instructions = compiler.compile(&parsed, None);
	let bf_code = builder.build(instructions.get_instructions());

	match config.optimise_generated_code {
		true => optimise(bf_code.chars().collect()),
		false => bf_code,
	}
}

#[wasm_bindgen]
pub fn wasm_run_bf(code: String) -> String {
	set_panic_hook();

	let mut bf = BVM::new(code.chars().collect());

	let mut output = Cursor::new(Vec::new());
	let mut input = Cursor::new(vec![]);
	bf.run(&mut input, &mut output);

	unsafe { String::from_utf8_unchecked(output.into_inner()) }
}
