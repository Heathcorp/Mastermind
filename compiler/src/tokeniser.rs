// TODO: refactor this tokeniser, needs some fixes and could be made simpler/cleaner

use regex_lite::Regex;

use crate::macros::macros::r_assert;

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
		// ("#debug", Token::Debug),
		("let", Token::Let),
		("=", Token::EqualsSign),
		// ("assert", Token::Assert),
		("while", Token::While),
		("drain", Token::Drain),
		("into", Token::Into),
		// ("clear", Token::Clear),
		// ("loop", Token::Loop),
		("else", Token::Else),
		("copy", Token::Copy),
		("bf", Token::Bf),
		("clobbers", Token::Clobbers),
		("assert", Token::Assert),
		("equals", Token::Equals),
		("unknown", Token::Unknown),
		// ("call", Token::Call),
		// ("bool", Token::Bool),
		// ("free", Token::Free),
		// ("push", Token::Push),
		// ("deal", Token::Deal),
		("def", Token::Def),
		// ("int", Token::Int),
		// ("add", Token::Add),
		// ("sub", Token::Sub),
		// ("pop", Token::Pop),
		("if", Token::If),
		("not", Token::Not),
		("else", Token::Else),
		("{", Token::OpenBrace),
		("}", Token::ClosingBrace),
		("[", Token::OpenSquareBracket),
		("]", Token::ClosingSquareBracket),
		("(", Token::OpenParenthesis),
		(")", Token::ClosingParenthesis),
		("<", Token::OpenAngledBracket),
		(">", Token::ClosingAngledBracket),
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

		let mut found = false;

		/////////
		if let Some(num_capture) = num_re.captures(remaining) {
			found = true;
			let substring = String::from(&num_capture[0]);
			chr_idx += substring.len();
			tokens.push(Token::Digits(substring));
		} else if let Some(name_capture) = name_re.captures(remaining) {
			found = true;
			let substring = String::from(&name_capture[0]);
			if mappings
				.iter()
				// this could be made more efficient if we had a table of keywords vs symbols
				.find(|(keyword, _)| substring == *keyword)
				.is_some()
			{
				found = false;
			} else {
				chr_idx += substring.len();
				tokens.push(Token::Name(substring));
			}
		} else if let Some(str_capture) = str_re.captures(remaining) {
			found = true;
			let substring = String::from(&str_capture[0]);
			// not the most efficient way, this simply removes the quote characters
			// could refactor this
			chr_idx += substring.len();
			let unescaped: String = serde_json::from_str(&substring)
				.or(Err("Could not unescape string literal in tokenisation due to serde error, this should never occur."))?;
			tokens.push(Token::String(unescaped));
		} else if let Some(chr_capture) = chr_re.captures(remaining) {
			found = true;
			let chr_literal = String::from(&chr_capture[0]);
			// see above
			chr_idx += chr_literal.len();
			// this code sucks, TODO: refactor
			// make a new double-quoted string because serde json doesn't like single quotes and I can't be bothered making my own unescaping function
			let escaped_string =
				String::new() + "\"" + &chr_literal[1..(chr_literal.len() - 1)] + "\"";
			let unescaped: String = serde_json::from_str(&escaped_string)
				.or(Err("Could not unescape character literal in tokenisation due to serde error, this should never occur."))?;
			// might need to change this for escaped characters (TODO)
			r_assert!(unescaped.len() == 1, "Character literals must be length 1");
			tokens.push(Token::Character(unescaped.chars().next().unwrap()));
		}
		/////////

		if !found {
			for (text, token) in mappings.iter() {
				if remaining.starts_with(*text) {
					tokens.push(token.clone());
					chr_idx += (*text).len();
					found = true;
					break;
				}
			}
		}
		r_assert!(
			found,
			"Unknown token found while tokenising program: \"{remaining}\""
		);
	}

	Ok(tokens
		.into_iter()
		.filter(|t| match t {
			Token::None => false,
			_ => true,
		})
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
	Def,
	Let,
	// Assert,
	// Free,
	While,
	If,
	Not,
	Else,
	// Loop,
	// Break,
	OpenBrace,
	ClosingBrace,
	OpenSquareBracket,
	ClosingSquareBracket,
	OpenParenthesis,
	ClosingParenthesis,
	OpenAngledBracket,
	ClosingAngledBracket,
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
	// Push,
	// Pop,
	// Deal,
	// Debug,
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
}
