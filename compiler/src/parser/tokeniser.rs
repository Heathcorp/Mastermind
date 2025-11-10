// TODO: make an impl for a tokeniser, inverse-builder pattern?
// have a function to peek, then accept changes, so we don't double hangle tokens

use crate::macros::macros::r_panic;

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
	Copy,
	Drain,
	Into,
	Bf,
	Clobbers,
	Assert,
	Equals,
	Unknown,
	True,
	False,
	LeftBrace,
	RightBrace,
	LeftSquareBracket,
	RightSquareBracket,
	LeftParenthesis,
	RightParenthesis,
	Comma,
	Dot,
	Asterisk,
	At,
	Plus,
	Minus,
	EqualsSign,
	Semicolon,
	PlusPlus,
	MinusMinus,
	PlusEquals,
	MinusEquals,
	Name(String),
	Number(usize),
	String(String),
	Character(char),
}

/// Get the next token from chars, advance the passed in pointer
pub fn next_token(chars: &mut &[char]) -> Result<Token, String> {
	// skip any whitespace
	loop {
		match chars.get(0) {
			Some(c) => {
				if !c.is_whitespace() {
					break;
				}
			}
			None => break,
		}
		*chars = &chars[1..];
	}

	// read the first character and branch from there
	let Some(c) = chars.get(0) else {
		return Ok(Token::None);
	};
	Ok(match *c {
		c @ (';' | '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | '*' | '@' | '+' | '-') => {
			*chars = &chars[1..];
			match c {
				';' => Token::Semicolon,
				'{' => Token::LeftBrace,
				'}' => Token::RightBrace,
				'(' => Token::LeftParenthesis,
				')' => Token::RightParenthesis,
				'[' => Token::LeftSquareBracket,
				']' => Token::RightSquareBracket,
				'.' => Token::Dot,
				',' => Token::Comma,
				'*' => Token::Asterisk,
				'@' => Token::At,
				'+' => match chars.get(0) {
					Some('+') => {
						*chars = &chars[1..];
						Token::PlusPlus
					}
					Some('=') => {
						*chars = &chars[1..];
						Token::PlusEquals
					}
					_ => Token::Plus,
				},
				'-' => match chars.get(0) {
					Some('-') => {
						*chars = &chars[1..];
						Token::MinusMinus
					}
					Some('=') => {
						*chars = &chars[1..];
						Token::MinusEquals
					}
					_ => Token::Minus,
				},
				_ => unreachable!(),
			}
		}
		'0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
			Token::Number(parse_number(chars)?)
		}
		'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o'
		| 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C'
		| 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q'
		| 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '_' => {
			let word = parse_word(chars)?;
			match word.as_str() {
				"output" => Token::Output,
				"input" => Token::Input,
				"fn" => Token::Fn,
				"cell" => Token::Cell,
				"struct" => Token::Struct,
				"while" => Token::While,
				"if" => Token::If,
				"not" => Token::Not,
				"else" => Token::Else,
				"copy" => Token::Copy,
				"drain" => Token::Drain,
				"into" => Token::Into,
				"bf" => Token::Bf,
				"clobbers" => Token::Clobbers,
				"assert" => Token::Assert,
				"equals" => Token::Equals,
				"unknown" => Token::Unknown,
				"true" => Token::True,
				"false" => Token::False,
				_ => Token::Name(word),
			}
		}
		'\'' => Token::Character(parse_character_literal(chars)?),
		'"' => Token::String(parse_string_literal(chars)?),
		_ => r_panic!("Invalid token found: `{c}`."),
	})
}

fn parse_number(chars: &mut &[char]) -> Result<usize, String> {
	// parse hexadecimal and binary
	// if let Some('0') = chars.get(0) {
	// 	match chars.get(1) {
	// 		// Some('x') => {
	// 		// 	let mut i = 2;
	// 		// }
	// 		// Some('b') => {
	// 		// 	let mut i = 2;
	// 		// }
	// 		_ => (),
	// 	}
	// }

	// parse decimal natural number
	let mut i = 0;
	let mut n = 0;
	loop {
		let Some(digit) = chars.get(i) else {
			break;
		};
		match digit {
			c @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => {
				n *= 10;
				n += (*c as usize) - ('0' as usize);
			}
			// '_' => {
			// 	// TODO: support underscores in number literals?
			// }
			';' | '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | '*' | '@' | '+' | '-' => break,
			c if c.is_whitespace() => break,
			'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n'
			| 'o' | 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | 'A' | 'B'
			| 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P'
			| 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '_' => {
				// TODO: add source snippet
				r_panic!("Unexpected word character in number token.")
			}
			// TODO: add source snippet
			_ => r_panic!("Unknown character found while parsing number token."),
		}
		i += 1;
	}

	Ok(n)
}

fn parse_word(chars: &mut &[char]) -> Result<String, String> {
	todo!()
}

/// handle character escape sequences, supports Rust ASCII escapes
fn parse_character_literal(chars: &mut &[char]) -> Result<char, String> {
	let mut i = 0;
	let Some('\'') = chars.get(i) else {
		r_panic!("Expected `'` at start of character literal.");
	};
	i += 1;
	let c = match chars.get(i) {
		Some('\\') => {
			i += 1;
			let c = match chars.get(i) {
				Some(c) => match c {
					'\'' => '\'',
					'n' => '\n',
					'r' => '\r',
					't' => '\t',
					'\\' => '\\',
					'0' => '\0',
					// TODO: add source snippet
					_ => r_panic!("Invalid escape sequence in character literal."),
				},
				None => r_panic!("Expected escape sequence in character literal."),
			};
			c
		}
		Some(c) => *c,
		None => r_panic!("Character literal must be length 1."),
	};
	i += 1;
	let Some('\'') = chars.get(i) else {
		r_panic!("Expected `'` at end of character literal.");
	};
	i += 1;

	// update used characters
	assert!(i <= chars.len());
	*chars = &chars[i..];

	Ok(c)
}

/// handle string escape sequences, supports Rust ASCII escapes
fn parse_string_literal(chars: &mut &[char]) -> Result<String, String> {
	let mut built_string = String::new();
	let mut i = 0;
	let Some('"') = chars.get(i) else {
		r_panic!("Expected `\"` at start of string literal.");
	};
	i += 1;
	loop {
		match chars.get(i) {
			None => r_panic!("Unexpected end of input in string literal."),
			Some('\\') => {
				i += 1;
				built_string.push(match chars.get(i) {
					Some('\"') => '"',
					Some('n') => '\n',
					Some('r') => '\r',
					Some('t') => '\t',
					Some('\\') => '\\',
					Some('0') => '\0',
					// TODO: add source snippet
					_ => r_panic!("Invalid escape sequence in string literal."),
				});
			}
			Some('"') => break,
			Some(c) => built_string.push(*c),
		}
	}
	i += 1;
	let Some('"') = chars.get(i) else {
		r_panic!("Expected `\"` at end of string literal.");
	};
	i += 1;

	// update used characters
	assert!(i <= chars.len());
	*chars = &chars[i..];

	Ok(built_string)
}

#[cfg(test)]
mod tokeniser_tests {
	use crate::macros::macros::r_panic;

	use super::*;

	fn tokenise(input_str: &str) -> Result<Vec<Token>, String> {
		let chars_vec: Vec<char> = input_str.chars().collect();
		let mut chars_slice = &chars_vec[..];
		let mut tokens = vec![];
		loop {
			let Ok(token) = next_token(&mut chars_slice) else {
				r_panic!("Invalid token in input.");
			};
			if let Token::None = token {
				break;
			}
			tokens.push(token);
		}
		Ok(tokens)
	}

	fn _tokenisation_test(input_str: &str, desired_output: &[Token]) {
		let actual_output = tokenise(input_str).unwrap();
		println!("desired: {desired_output:#?}");
		println!("actual: {actual_output:#?}");
		assert!(actual_output.iter().eq(desired_output));
	}

	#[test]
	fn empty_1() {
		_tokenisation_test("", &[]);
	}

	#[test]
	fn empty_1a() {
		_tokenisation_test("  \n  \t  ", &[]);
	}

	#[test]
	fn empty_2() {
		let chars_vec: Vec<char> = "".chars().collect();
		let mut chars_slice = &chars_vec[..];
		assert_eq!(next_token(&mut chars_slice).unwrap(), Token::None);
	}

	#[test]
	fn empty_2a() {
		let chars_vec: Vec<char> = "\n    \t \n  ".chars().collect();
		let mut chars_slice = &chars_vec[..];
		assert_eq!(next_token(&mut chars_slice).unwrap(), Token::None);
	}

	#[test]
	fn single() {
		let desired_output = [
			Token::EqualsSign,
			Token::EqualsSign,
			Token::Semicolon,
			Token::Semicolon,
			Token::Asterisk,
			Token::Asterisk,
			Token::At,
			Token::At,
			Token::LeftSquareBracket,
			Token::LeftSquareBracket,
			Token::LeftBrace,
			Token::LeftBrace,
			Token::LeftParenthesis,
			Token::LeftParenthesis,
			Token::RightSquareBracket,
			Token::RightSquareBracket,
			Token::RightBrace,
			Token::RightBrace,
			Token::RightParenthesis,
			Token::RightParenthesis,
			Token::Dot,
			Token::Dot,
			Token::Comma,
			Token::Comma,
		];
		_tokenisation_test("==;;**@@[[{{((]]}}))..,,", &desired_output);
		_tokenisation_test(" == ; ;**@ @[[ {{ ( (] ]}} )). ., ,", &desired_output);
	}

	#[test]
	fn double_1() {
		_tokenisation_test(
			"+=+=-=-=++++----",
			&[
				Token::PlusEquals,
				Token::PlusEquals,
				Token::MinusEquals,
				Token::MinusEquals,
				Token::PlusPlus,
				Token::PlusPlus,
				Token::MinusMinus,
				Token::MinusMinus,
			],
		);
	}

	#[test]
	fn double_1a() {
		_tokenisation_test(
			"+ =+ = -= -=+ +++ - - --",
			&[
				Token::Plus,
				Token::EqualsSign,
				Token::Plus,
				Token::EqualsSign,
				Token::MinusEquals,
				Token::MinusEquals,
				Token::Plus,
				Token::PlusPlus,
				Token::Plus,
				Token::Minus,
				Token::Minus,
				Token::MinusMinus,
			],
		);
	}

	#[test]
	fn double_2() {
		_tokenisation_test(
			"-++=+++=+-=--=---=-+++++-+-----",
			&[
				Token::Minus,
				Token::PlusPlus,
				Token::EqualsSign,
				Token::PlusPlus,
				Token::PlusEquals,
				Token::Plus,
				Token::MinusEquals,
				Token::MinusMinus,
				Token::EqualsSign,
				Token::MinusMinus,
				Token::MinusEquals,
				Token::Minus,
				Token::PlusPlus,
				Token::PlusPlus,
				Token::Plus,
				Token::Minus,
				Token::Plus,
				Token::MinusMinus,
				Token::MinusMinus,
				Token::Minus,
			],
		);
	}

	#[test]
	fn double_2a() {
		_tokenisation_test(
			"-+ +=+ ++=+-=-- =-- - =-+ +++ +-+-- - --",
			&[
				Token::Minus,
				Token::Plus,
				Token::PlusEquals,
				Token::Plus,
				Token::Plus,
				Token::PlusEquals,
				Token::Plus,
				Token::MinusEquals,
				Token::MinusMinus,
				Token::EqualsSign,
				Token::MinusMinus,
				Token::Minus,
				Token::EqualsSign,
				Token::Minus,
				Token::Plus,
				Token::PlusPlus,
				Token::Plus,
				Token::Plus,
				Token::Minus,
				Token::Plus,
				Token::MinusMinus,
				Token::Minus,
				Token::MinusMinus,
			],
		);
	}

	#[test]
	fn single_and_double() {
		_tokenisation_test(
			"=+==;+=- =;*---=++*@@[[{{+ +((]--]}+-+})).---.-,,",
			&[
				Token::EqualsSign,
				Token::PlusEquals,
				Token::EqualsSign,
				Token::Semicolon,
				Token::PlusEquals,
				Token::Minus,
				Token::EqualsSign,
				Token::Semicolon,
				Token::Asterisk,
				Token::MinusMinus,
				Token::MinusEquals,
				Token::Plus,
				Token::Plus,
				Token::Asterisk,
				Token::At,
				Token::At,
				Token::LeftSquareBracket,
				Token::LeftSquareBracket,
				Token::LeftBrace,
				Token::LeftBrace,
				Token::PlusPlus,
				Token::LeftParenthesis,
				Token::LeftParenthesis,
				Token::RightSquareBracket,
				Token::MinusMinus,
				Token::RightSquareBracket,
				Token::RightBrace,
				Token::Plus,
				Token::Minus,
				Token::Plus,
				Token::RightBrace,
				Token::RightParenthesis,
				Token::RightParenthesis,
				Token::Dot,
				Token::MinusMinus,
				Token::Minus,
				Token::Dot,
				Token::Minus,
				Token::Comma,
				Token::Comma,
			],
		);
	}

	#[test]
	fn keywords() {
		_tokenisation_test(
			r#"
output output input input fn fn cell cell 	struct struct while while if
if not not else else copy copy 	drain drain into into bf bf clobbers clobbers
 	assert assert equals equals unknown unknown true true false false
"#,
			&[
				Token::Output,
				Token::Output,
				Token::Input,
				Token::Input,
				Token::Fn,
				Token::Fn,
				Token::Cell,
				Token::Cell,
				Token::Struct,
				Token::Struct,
				Token::While,
				Token::While,
				Token::If,
				Token::If,
				Token::Not,
				Token::Not,
				Token::Else,
				Token::Else,
				Token::Copy,
				Token::Copy,
				Token::Drain,
				Token::Drain,
				Token::Into,
				Token::Into,
				Token::Bf,
				Token::Bf,
				Token::Clobbers,
				Token::Clobbers,
				Token::Assert,
				Token::Assert,
				Token::Equals,
				Token::Equals,
				Token::Unknown,
				Token::Unknown,
				Token::True,
				Token::True,
				Token::False,
				Token::False,
			],
		);
	}

	#[test]
	fn keywords_and_simples() {
		_tokenisation_test(
			r#"unknown,assert,equals.into;struct)clobbers-- -+input+++not(else{
if fn{output)true)false -while*  @copy@+=@drain-=into=][bf.cell"#,
			&[
				Token::Unknown,
				Token::Comma,
				Token::Assert,
				Token::Comma,
				Token::Equals,
				Token::Dot,
				Token::Into,
				Token::Semicolon,
				Token::Struct,
				Token::RightParenthesis,
				Token::Clobbers,
				Token::MinusMinus,
				Token::Minus,
				Token::Plus,
				Token::Input,
				Token::Plus,
				Token::PlusPlus,
				Token::Not,
				Token::LeftParenthesis,
				Token::Else,
				Token::LeftBrace,
				Token::If,
				Token::Fn,
				Token::LeftBrace,
				Token::Output,
				Token::RightParenthesis,
				Token::True,
				Token::RightParenthesis,
				Token::False,
				Token::Minus,
				Token::While,
				Token::Asterisk,
				Token::At,
				Token::Copy,
				Token::At,
				Token::PlusEquals,
				Token::At,
				Token::Drain,
				Token::MinusEquals,
				Token::Into,
				Token::EqualsSign,
				Token::RightSquareBracket,
				Token::LeftSquareBracket,
				Token::Bf,
				Token::Dot,
				Token::Cell,
			],
		);
	}

	#[test]
	fn names_1() {
		_tokenisation_test("i", &[Token::Name(String::from("i"))]);
	}

	#[test]
	fn names_1a() {
		_tokenisation_test("_", &[Token::Name(String::from("_"))]);
	}

	#[test]
	fn names_2() {
		_tokenisation_test(
			"while hello",
			&[Token::While, Token::Name(String::from("hello"))],
		);
	}

	#[test]
	fn names_2a() {
		_tokenisation_test(
			"while_",
			&[Token::While, Token::Name(String::from("while_"))],
		);
	}

	#[test]
	fn names_2b() {
		_tokenisation_test(
			"if_else_while_hello;welcome\ninto the if club",
			&[
				Token::Name(String::from("if_else_while_hello")),
				Token::Semicolon,
				Token::Name(String::from("welcome")),
				Token::Into,
				Token::Name(String::from("the")),
				Token::If,
				Token::Name(String::from("club")),
			],
		);
	}

	#[test]
	fn names_2c() {
		_tokenisation_test(
			"hello{If;elSe ___if}\n\n\nclobberss",
			&[
				Token::Name(String::from("hello")),
				Token::LeftBrace,
				Token::Name(String::from("If")),
				Token::Semicolon,
				Token::Name(String::from("elSe")),
				Token::Name(String::from("___if")),
				Token::RightBrace,
				Token::Name(String::from("clobberss")),
			],
		);
	}

	#[test]
	fn names_2d() {
		_tokenisation_test(
			"hello while you were gone I",
			&[
				Token::Name(String::from("hello")),
				Token::While,
				Token::Name(String::from("you")),
				Token::Name(String::from("were")),
				Token::Name(String::from("gone")),
				Token::Name(String::from("I")),
			],
		);
	}

	#[test]
	fn character_literals_1() {
		_tokenisation_test(
			r#"'a' 'b' 'c' ' '"#,
			&[
				Token::Character('a'),
				Token::Character('b'),
				Token::Character('c'),
				Token::Character(' '),
			],
		);
	}

	#[test]
	fn character_literals_2() {
		_tokenisation_test(r#"'\n'"#, &[Token::Character('\n')]);
	}

	#[test]
	fn character_literals_3() {
		_tokenisation_test(r#"'"'"#, &[Token::Character('"')]);
	}

	#[test]
	fn character_literals_4() {
		_tokenisation_test(r#"'\''"#, &[Token::Character('\'')]);
	}

	#[test]
	#[should_panic]
	fn character_literals_5() {
		_tokenisation_test(r#"'\'"#, &[Token::Character('\\')]);
	}

	#[test]
	#[should_panic]
	fn character_literals_6() {
		_tokenisation_test(r#"'aa'"#, &[Token::String(String::from("aa"))]);
	}

	#[test]
	fn string_literals_1() {
		_tokenisation_test("\"hello\"", &[Token::String(String::from("hello"))]);
	}

	#[test]
	fn string_literals_2() {
		_tokenisation_test(r#""""#, &[Token::String(String::from(""))]);
	}

	#[test]
	fn string_literals_2a() {
		_tokenisation_test(
			r#""""""#,
			&[
				Token::String(String::from("")),
				Token::String(String::from("")),
			],
		);
	}

	#[test]
	fn string_literals_3() {
		_tokenisation_test(
			r#""\"" " ""#,
			&[
				Token::String(String::from("\"")),
				Token::String(String::from(" ")),
			],
		);
	}

	#[test]
	fn numbers_dec_1() {
		_tokenisation_test(
			"1 123 000098763",
			&[
				Token::String(String::from("\"")),
				Token::String(String::from(" ")),
			],
		);
	}

	#[test]
	fn numbers_dec_2() {
		_tokenisation_test(
			".0654 567.32",
			&[
				Token::Dot,
				Token::Number(654),
				Token::Number(567),
				Token::Dot,
				Token::Number(32),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_1() {
		_tokenisation_test(
			"0x56 0x00 0x00ff1 0x4ff2",
			&[
				Token::Number(0x56),
				Token::Number(0x00),
				Token::Number(0xff1),
				Token::Number(0x4ff2),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_1a() {
		_tokenisation_test(
			"0x 56 0x00 0x00f f1 0 x4ff2",
			&[
				Token::Number(0),
				Token::Name(String::from("x")),
				Token::Number(56),
				Token::Number(0x00),
				Token::Number(0x00f),
				Token::Name(String::from("f1")),
				Token::Number(0),
				Token::Name(String::from("x4ff2")),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_2() {
		_tokenisation_test(
			"0x56 0x00 0x00ff1 0x4ff2",
			&[
				Token::Number(0x56),
				Token::Number(0x00),
				Token::Number(0xff1),
				Token::Number(0x4ff2),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_2a() {
		_tokenisation_test(
			"0x 56 0x00 0x00f f1 0 x4ff2",
			&[
				Token::Number(0),
				Token::Name(String::from("x")),
				Token::Number(56),
				Token::Number(0x00),
				Token::Number(0x00f),
				Token::Name(String::from("f1")),
				Token::Number(0),
				Token::Name(String::from("x4ff2")),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_bin_1() {
		_tokenisation_test(
			"0b1111 0b000 0b0 0b1 0b1010100 0b001101",
			&[
				Token::Number(0b1111),
				Token::Number(0b000),
				Token::Number(0b0),
				Token::Number(0b1),
				Token::Number(0b1010100),
				Token::Number(0b001101),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_bin_1a() {
		_tokenisation_test(
			"0b1 111 0 b000 0 b 0 0b1 0b101 0100 0b001101",
			&[
				Token::Number(0b1),
				Token::Number(111),
				Token::Number(0),
				Token::Name(String::from("b000")),
				Token::Number(0),
				Token::Name(String::from("b")),
				Token::Number(0),
				Token::Number(0b1),
				Token::Number(0b101),
				Token::Number(100),
				Token::Number(0b1101),
			],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_bin_1() {
		_tokenisation_test(
			"0x11001 0b11001",
			&[Token::Number(0x11001), Token::Number(0b11001)],
		);
	}

	#[test]
	#[ignore]
	fn numbers_hex_bin_2() {
		for s in [
			"0b00102", "0b013000", "0b010040", "0b050000", "0b66000", "0b017", "0b8", "0b90",
			"0b01a0", "0b4b", "0b01c0", "0b0d", "0b01e0", "0b01f",
		] {
			assert_eq!(tokenise(s).unwrap_err(), "");
		}
	}

	#[test]
	fn numbers_and_words_dec() {
		assert_eq!(tokenise("456hello").unwrap_err(), "");
	}

	#[test]
	#[ignore]
	fn numbers_and_words_hex() {
		assert_eq!(tokenise("0x00free me").unwrap_err(), "");
	}

	#[test]
	#[ignore]
	fn numbers_and_words_bin() {
		assert_eq!(tokenise("0b00ebrave").unwrap_err(), "");
	}
}
