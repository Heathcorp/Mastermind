use regex_lite::Regex;

use crate::macros::macros::{r_assert, r_panic};

pub fn tokenise(source: &String) -> Result<Vec<Token>, String> {
	let stripped = source
		.lines()
		.map(strip_line)
		.collect::<Vec<String>>()
		.join(" ");

	let mappings = [
		(" ", Token::None),
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
		("^", Token::UpToken),
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
	let num_re = Regex::new(r#"^[0-9]+"#).unwrap();
	let name_re = Regex::new(r#"^[a-zA-Z_]\w*"#).unwrap();
	// string regex taken from chatgpt
	let str_re = Regex::new(r#"^"(?:[^"\\]|\\.)+""#).unwrap();
	// char regex taken from chatgpt again
	let chr_re = Regex::new(r#"^'(?:[^'\\]|\\.)'"#).unwrap();

	let mut tokens: Vec<Token> = Vec::new();

	let mut chr_idx = 0usize;
	while chr_idx < stripped.len() {
		let remaining = &stripped[chr_idx..];

		if let Some(substring) = num_re
			.captures(remaining)
			.map(|num_capture| String::from(&num_capture[0]))
		{
			chr_idx += substring.len();
			tokens.push(Token::Digits(substring));
		} else if let Some(substring) = name_re
			.captures(remaining)
			.map(|name_capture| String::from(&name_capture[0]))
			.take_if(|substring| {
				mappings
					.iter()
					.find(|(keyword, _)| substring == *keyword)
					.is_none()
			}) {
			chr_idx += substring.len();
			tokens.push(Token::Name(substring));
		} else if let Some(substring) = str_re
			.captures(remaining)
			.map(|str_capture| String::from(&str_capture[0]))
		{
			chr_idx += substring.len();
			let unescaped = serde_json::from_str(&substring)
				.or(Err("Could not unescape string literal in tokenisation \
due to serde error, this should never occur."))?;
			tokens.push(Token::String(unescaped));
		} else if let Some(substring) = chr_re
			.captures(remaining)
			.map(|chr_capture| String::from(&chr_capture[0]))
		{
			chr_idx += substring.len();
			// hack: replace single quotes with double quotes, then use serde to unescape all the characters
			let escaped_string = String::new() + "\"" + &substring[1..(substring.len() - 1)] + "\"";
			let unescaped: String = serde_json::from_str(&escaped_string)
				.or(Err("Could not unescape character literal in tokenisation \
due to serde error, this should never occur."))?;

			r_assert!(unescaped.len() == 1, "Character literals must be length 1");
			tokens.push(Token::Character(unescaped.chars().next().unwrap()));
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
		// stick a None token on the end to fix some weird parsing errors (seems silly but why not?)
		.chain([Token::None])
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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	None,
	Output,
	Input,
	Fn,
	Cell,
	Struct,
	While,
	If,
	Not,
	Else,
	OpenBrace,
	ClosingBrace,
	OpenSquareBracket,
	ClosingSquareBracket,
	OpenParenthesis,
	ClosingParenthesis,
	LessThan,
	MoreThan,
	Comma,
	Dot,
	Asterisk,
	At,
	Copy,
	Drain,
	Into,
	Bf,
	Clobbers,
	Assert,
	Equals,
	Unknown,
	Name(String),
	Digits(String),
	String(String),
	Character(char),
	True,
	False,
	Minus,
	Plus,
	EqualsSign,
	Semicolon,
	UpToken,
}

#[cfg(test)]
mod tokeniser_tests {
	use crate::tokeniser::{tokenise, Token};

	fn _character_literal_test(input_str: &str, desired_output: &[Token]) {
		let input_string = String::from(input_str);
		let actual_output = tokenise(&input_string).unwrap();
		println!("desired: {desired_output:#?}");
		println!("actual: {actual_output:#?}");
		assert!(actual_output.iter().eq(desired_output));
	}

	#[test]
	fn character_literals_1() {
		_character_literal_test(
			r#"'a' 'b' 'c' ' '"#,
			&[
				Token::Character('a'),
				Token::Character('b'),
				Token::Character('c'),
				Token::Character(' '),
				// TODO: remove this None, fix the code that needs it
				Token::None,
			],
		);
	}

	#[test]
	fn character_literals_2() {
		_character_literal_test(
			r#"'\n'"#,
			&[
				Token::Character('\n'),
				// TODO: remove this None, fix the code that needs it
				Token::None,
			],
		);
	}

	#[test]
	fn character_literals_3() {
		_character_literal_test(
			r#"'"'"#,
			&[
				Token::Character('"'),
				// TODO: remove this None, fix the code that needs it
				Token::None,
			],
		);
	}

	#[test]
	fn character_literals_4() {
		_character_literal_test(
			r#"'\''"#,
			&[
				Token::Character('\''),
				// TODO: remove this None, fix the code that needs it
				Token::None,
			],
		);
	}
}
