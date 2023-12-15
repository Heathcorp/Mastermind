// take in a file, read includes and simple conditionals and output a file with those includes pasted in
// C-style

use std::path::PathBuf;

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

// utility functions split out so that files can be compiled from javascript strings in browser
