// take in a file, read includes and simple conditionals and output a file with those includes pasted in
// C-style

use std::{collections::HashMap, path::PathBuf};

pub fn preprocess(file_path: PathBuf) -> String {
	let file_contents = std::fs::read_to_string(&file_path).unwrap();
	let mut dir_path = file_path.clone();
	dir_path.pop();

	file_contents
		.lines()
		.map(|line| {
			if line.starts_with("#include ") {
				let rel_include_path = PathBuf::from(&line[9..]);
				let include_path = dir_path.join(rel_include_path);
				preprocess(include_path)
			} else {
				line.to_owned()
			}
		})
		.fold(String::new(), |acc, e| acc + &e + "\n")
}

// utility functions so that files can be compiled from javascript strings in browser
pub fn preprocess_from_memory(
	file_map: &HashMap<String, String>,
	entry_file_name: String,
) -> String {
	let file_contents = file_map
		.get(&entry_file_name)
		.expect(&format!("No such file \"{entry_file_name}\" exists"));

	file_contents
		.lines()
		.map(|line| {
			if line.starts_with("#include ") {
				let other_file_name = String::from(&line[9..]);
				preprocess_from_memory(file_map, other_file_name)
			} else {
				line.to_owned()
			}
		})
		.fold(String::new(), |acc, e| acc + &e + "\n")
}
