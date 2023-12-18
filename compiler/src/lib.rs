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

use std::collections::HashMap;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
	pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn wasm_compile(file_contents: JsValue, entry_file_name: String, config: JsValue) -> String {
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
