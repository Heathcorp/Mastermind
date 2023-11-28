use regex::Regex;

pub fn tokenise(source: &String) -> Vec<Token> {
	let stripped = source
		.lines()
		.map(strip_line)
		.collect::<Vec<String>>()
		.join(" ");

	println!("{stripped}");

	let mappings = [
		(" ", Token::None),
		(";", Token::ClauseDelimiter),
		("output", Token::Output),
		// ("#debug", Token::Debug),
		("let", Token::Let),
		("=", Token::Equals),
		// ("assert", Token::Assert),
		("while", Token::While),
		("drain", Token::Drain),
		("into", Token::Into),
		// ("clear", Token::Clear),
		// ("loop", Token::Loop),
		("else", Token::Else),
		("copy", Token::Copy),
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
		("true", Token::True),
		("false", Token::False),
		(",", Token::Comma),
		("-", Token::Minus),
		("+", Token::Plus),
	];

	let mut tokens: Vec<Token> = Vec::new();

	let mut chr_idx = 0usize;
	while chr_idx < stripped.len() {
		let remaining = &stripped[chr_idx..];

		let mut found = false;
		for (text, token) in mappings.iter() {
			if remaining.starts_with(*text) {
				tokens.push(token.clone());
				chr_idx += (*text).len();
				found = true;
				break;
			}
		}
		if !found {
			// check for numbers and variables
			let num_re = Regex::new(r"^[0-9]+").unwrap();
			let txt_re = Regex::new(r"^[a-zA-Z_]\w*").unwrap();
			let str_re = Regex::new("^\".*\"").unwrap();
			let chr_re = Regex::new("^'.'").unwrap();
			if let Some(num_capture) = num_re.captures(remaining) {
				let substring = String::from(&num_capture[0]);
				chr_idx += substring.len();
				tokens.push(Token::Digits(substring));
			} else if let Some(txt_capture) = txt_re.captures(remaining) {
				let substring = String::from(&txt_capture[0]);
				chr_idx += substring.len();
				tokens.push(Token::Name(substring));
			} else if let Some(str_capture) = str_re.captures(remaining) {
				let mut substring = String::from(&str_capture[0]);
				// not the most efficient way, this simply removes the quote characters
				// could refactor this
				chr_idx += substring.len();
				substring.pop();
				substring.remove(0);
				tokens.push(Token::String(substring));
			} else if let Some(chr_capture) = chr_re.captures(remaining) {
				let mut substring = String::from(&chr_capture[0]);
				// see above
				chr_idx += substring.len();
				substring.pop();
				substring.remove(0);
				// might need to change this for escaped characters (TODO)
				assert_eq!(substring.len(), 1, "Character literals must be length 1");
				tokens.push(Token::Character(substring.chars().next().unwrap()));
			} else {
				panic!("Unknown token found while tokenising program: \"{remaining}\"");
			}
		}
	}

	tokens
		.into_iter()
		.filter(|t| match t {
			Token::None => false,
			_ => true,
		})
		.collect()
}

fn strip_line(line: &str) -> String {
	let mut stripped = line;
	// remove comments
	let split = line.split_once("//");
	if split.is_some() {
		stripped = split.unwrap().0;
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
	Comma,
	Copy,
	Drain,
	Into,
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
	Equals,
	ClauseDelimiter,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn dummy() {
		let program = String::from(
			"
let h[3] = 6;
int[1] e 5;
int[1] l 12;
int[1] o 15;
// comment!
int[1] a_char 96;
loop a_char
{
	add h 1;
	add e 1;
	add l 1;
	add o 1;
};
free a_char;
output h;
output e;
output l;
output l;
output o;
int[1] ten 10;
output ten;
",
		);
		let tokens = tokenise(&program);
		println!("{tokens:#?}");
		// let input = String::from("");
		// let desired_output = String::from("");
		assert_eq!("hello", "hello");
	}
}
