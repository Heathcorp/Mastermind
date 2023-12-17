use std::collections::HashMap;

use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
	pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn wasm_compile(file_map: JsValue, entry_file_id: String) -> String {
	let file_map: HashMap<String, String> = serde_wasm_bindgen::from_value(file_map).unwrap();

	file_map.get(&entry_file_id).unwrap().clone()
}
