// take in a file, read includes and simple conditionals and output a file with those includes pasted in
// C-style

// TODO: add tests for this!

use std::{collections::HashMap, path::PathBuf};

use itertools::Itertools;

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

/// strips comments from input program, does not support anything else
pub fn strip_comments(raw_program: &str) -> String {
	let mut stripped = raw_program
		.lines()
		.map(|line| line.split_once("//").map_or_else(|| line, |(left, _)| left))
		.join("\n");
	// join doesn't add a newline to the end, here we re-add it, this is probably unnecessary
	if raw_program.ends_with("\n") {
		stripped.push_str("\n");
	}
	stripped
}

#[cfg(test)]
pub mod preprocessor_tests {
	use crate::preprocessor::strip_comments;

	#[test]
	fn comments_0() {
		assert_eq!(strip_comments(""), "");
		assert_eq!(strip_comments("\n\t\t\n"), "\n\t\t\n");
	}

	#[test]
	fn comments_1() {
		assert_eq!(strip_comments("hi//hello"), "hi");
	}

	#[test]
	fn comments_2() {
		assert_eq!(strip_comments("h//i // hello"), "h");
	}

	#[test]
	fn comments_3() {
		assert_eq!(
			strip_comments(
				r#"
hello // don't talk to me
second line
// third line comment
fourth line
"#
			),
			r#"
hello 
second line

fourth line
"#
		);
	}
}
