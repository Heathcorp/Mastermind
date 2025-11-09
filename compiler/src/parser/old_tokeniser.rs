// project dependencies:
use crate::macros::macros::{r_assert, r_panic};

// external dependencies:
use regex_lite::Regex;

// TODO: refactor: combine tokeniser and parser into one
//  make the inline brainfuck tokens contextual
pub fn tokenise(source: &str) -> Result<Vec<Token>, String> {
	let stripped = source
		.lines()
		.map(strip_line)
		.collect::<Vec<String>>()
		.join(" ");

	// mappings are a list of key * value tuples because we are doing "starts with" searches,
	//  meaning we can't look up in a hashtable
	let mappings = [
		// (" ", Token::None),
		(";", Token::Semicolon),
		("output", Token::Output),
		("input", Token::Input),
		("cell", Token::Cell),
		("struct", Token::Struct),
		("=", Token::EqualsSign),
		("while", Token::While),
		("drain", Token::Drain),
		("into", Token::Into),
		("else", Token::Else),
		("copy", Token::Copy),
		("bf", Token::Bf),
		("clobbers", Token::Clobbers),
		("assert", Token::Assert),
		("equals", Token::Equals),
		("unknown", Token::Unknown),
		("fn", Token::Fn),
		("if", Token::If),
		("not", Token::Not),
		("else", Token::Else),
		("{", Token::OpenBrace),
		("}", Token::ClosingBrace),
		("[", Token::OpenSquareBracket),
		("]", Token::ClosingSquareBracket),
		("(", Token::OpenParenthesis),
		(")", Token::ClosingParenthesis),
		("<", Token::LessThan),
		(">", Token::MoreThan),
		("^", Token::Caret),
		("true", Token::True),
		("false", Token::False),
		(",", Token::Comma),
		(".", Token::Dot),
		("*", Token::Asterisk),
		("@", Token::At),
		("-", Token::Minus),
		("+", Token::Plus),
	];
	// check for numbers and variables
	let number_regex = Regex::new(r#"^[0-9]+"#).unwrap();
	let name_regex = Regex::new(r#"^[a-zA-Z_]\w*"#).unwrap();
	let string_regex = Regex::new(r#"^"(?:[^"\\]|\\.)*""#).unwrap();
	let character_regex = Regex::new(r#"^'(?:[^'\\]|\\.)'"#).unwrap();

	let mut tokens: Vec<Token> = Vec::new();

	let mut chr_idx = 0usize;
	while chr_idx < stripped.len() {
		let remaining = &stripped[chr_idx..];

		if let Some(raw) = number_regex
			.captures(remaining)
			.map(|num_capture| String::from(&num_capture[0]))
		{
			chr_idx += raw.len();
			tokens.push(Token::Digits(raw));
		} else if let Some(raw) = name_regex
			.captures(remaining)
			.map(|name_capture| String::from(&name_capture[0]))
			.take_if(|raw| {
				mappings
					.iter()
					.find(|(keyword, _)| raw == *keyword)
					.is_none()
			}) {
			chr_idx += raw.len();
			tokens.push(Token::Name(raw));
		} else if let Some(raw) = string_regex
			.captures(remaining)
			.map(|str_capture| String::from(&str_capture[0]))
		{
			chr_idx += raw.len();
			r_assert!(
				raw.len() >= 2,
				"Not enough characters in string literal token, \
this should never occur. {raw:#?}"
			);
			tokens.push(Token::String(tokenise_raw_string_literal(
				&raw[1..(raw.len() - 1)],
			)?));
		} else if let Some(raw) = character_regex
			.captures(remaining)
			.map(|chr_capture| String::from(&chr_capture[0]))
		{
			chr_idx += raw.len();
			r_assert!(
				raw.len() >= 2,
				"Not enough characters in character literal token, \
this should never occur. {raw:#?}"
			);
			tokens.push(Token::Character(tokenise_raw_character_literal(
				&raw[1..(raw.len() - 1)],
			)?));
		} else if let Some((text, token)) = mappings
			.iter()
			.find(|(text, _)| remaining.starts_with(text))
		{
			tokens.push(token.clone());
			chr_idx += (*text).len();
		} else {
			r_panic!("Unknown token found while tokenising program: \"{remaining}\"");
		}
	}

	Ok(tokens
		.into_iter()
		.filter(|t| !matches!(t, Token::None))
		.collect())
}

fn strip_line(line: &str) -> String {
	let mut stripped = line;
	// remove comments
	let split = line.split_once("//");
	if let Some((one, _comment)) = split {
		stripped = one;
	}

	// remove excess whitespace
	stripped
		.trim()
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" ")
}

/// handle string escape sequences
// supports Rust ASCII escapes
fn tokenise_raw_string_literal(raw: &str) -> Result<String, String> {
	let mut s_iter = raw.chars();
	let mut built_string = String::with_capacity(raw.len());
	while let Some(raw_char) = s_iter.next() {
		built_string.push(match raw_char {
			'\\' => match s_iter.next() {
				Some(c) => match c {
					'\"' => '"',
					'n' => '\n',
					'r' => '\r',
					't' => '\t',
					'\\' => '\\',
					'0' => '\0',
					_ => r_panic!("Invalid escape sequence in string literal: {raw}"),
				},
				None => r_panic!("Expected escape sequence in string literal: {raw}"),
			},
			c => c,
		});
	}
	Ok(built_string)
}
