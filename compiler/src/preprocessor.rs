// take in a file, read includes and simple conditionals and output a file with those includes pasted in
// C-style

use std::{collections::HashMap, path::PathBuf};

use crate::macros::macros::r_assert;

pub fn preprocess(file_path: PathBuf) -> String {
	let file_contents = std::fs::read_to_string(&file_path).unwrap();
	let mut dir_path = file_path.clone();
	dir_path.pop();

	file_contents
		.lines()
		.map(|line| {
			if line.starts_with("#include") {
				// TODO: refactor and deduplicate code, currently doesn't care if "" or <> or jk or any set of two characters
				let split: Vec<&str> = line.split_whitespace().collect();
				assert!(
					split.len() == 2,
					"Malformed #include preprocessor directive {line}"
				);
				let mut substring = split[1];
				assert!(
					substring.len() > 2,
					"Expected path string in #include preprocessor directive {line}"
				);
				substring = &substring[1..(substring.len() - 1)];

				let rel_include_path = PathBuf::from(substring);
				let include_path = dir_path.join(rel_include_path);
				preprocess(include_path)
			} else {
				line.to_owned()
			}
		})
		.fold(String::new(), |acc, e| acc + &e + "\n")
}

// utility function so that files can be compiled from javascript strings in browser

pub fn preprocess_from_memory(
	file_map: &HashMap<String, String>,
	entry_file_name: String,
) -> Result<String, String> {
	let file_contents = file_map
		.get(&entry_file_name)
		.expect(&format!("No such file \"{entry_file_name}\" exists"));

	let mut acc = String::new();
	for line in file_contents.lines() {
		if line.starts_with("#include") {
			// TODO: refactor and deduplicate code, currently doesn't care if "" or <> or jk or any set of two characters
			let split: Vec<&str> = line.split_whitespace().collect();
			r_assert!(
				split.len() == 2,
				"Malformed #include preprocessor directive {line}"
			);
			let mut substring = split[1];
			r_assert!(
				substring.len() > 2,
				"Expected path string in #include preprocessor directive {line}"
			);
			substring = &substring[1..(substring.len() - 1)];

			acc += &preprocess_from_memory(file_map, substring.to_owned())?;
		} else {
			acc += line;
		}
		acc.push('\n');
	}

	Ok(acc)
}

#[cfg(test)]
mod preprocessor_tests {
	// fn def_ifdef_1() {

	// }
}
