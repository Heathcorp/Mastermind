// TODO: make an impl for a tokeniser, inverse-builder pattern?
// have a function to peek, then accept changes, so we don't double hangle tokens

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
	Name(String),
	Digits(String),
	String(String),
	Character(char),
	Plus,
	Minus,
	PlusPlus,
	MinusMinus,
	PlusEquals,
	MinusEquals,
	EqualsSign,
	Semicolon,
}

/// Get the next token from chars, advance the passed in pointer
pub fn next_token(chars: &mut &[char]) -> Result<Token, ()> {
	// skip any whitespace
	skip_whitespace(chars);

	// read the first character and branch from there
	let Some(c) = chars.get(0) else {
		return Ok(Token::None);
	};
	Ok(match *c {
		';' => {
			*chars = &chars[1..];
			Token::Semicolon
		}
		'{' => {
			*chars = &chars[1..];
			Token::LeftBrace
		}
		'}' => {
			*chars = &chars[1..];
			Token::RightBrace
		}
		'(' => {
			*chars = &chars[1..];
			Token::LeftParenthesis
		}
		')' => {
			*chars = &chars[1..];
			Token::RightParenthesis
		}
		'[' => {
			*chars = &chars[1..];
			Token::LeftSquareBracket
		}
		']' => {
			*chars = &chars[1..];
			Token::RightSquareBracket
		}
		'.' => {
			*chars = &chars[1..];
			Token::Dot
		}
		',' => {
			*chars = &chars[1..];
			Token::Comma
		}
		'*' => {
			*chars = &chars[1..];
			Token::Asterisk
		}
		'@' => {
			*chars = &chars[1..];
			Token::At
		}
		_ => todo!(),
	})
}

// TODO: fix this, make this based on token, currently it has no nuance for strings for example
// TODO: figure out errors for these helper functions
pub fn find_next(chars: &[char], character: char) -> Result<usize, ()> {
	let mut i = 0;
	loop {
		let Some(c) = chars.get(i) else {
			return Err(());
		};

		if *c == character {
			break;
		}
		i += 1;
	}
	Ok(i)
}

// TODO: fix this, make this based on token, currently it has no nuance for strings for example
pub fn find_and_advance<'a>(chars: &'a mut &[char], character: char) -> Result<&'a [char], ()> {
	let substr_len = find_next(chars, character)?;
	let chars_before = &chars[..substr_len];
	*chars = &chars[substr_len..];
	Ok(chars_before)
}

pub fn skip_whitespace(chars: &mut &[char]) {
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
}

pub fn find_next_whitespace(chars: &[char]) -> Result<usize, ()> {
	let mut i = 0;
	loop {
		let Some(c) = chars.get(i) else {
			return Err(());
		};

		if c.is_whitespace() {
			break;
		}
		i += 1;
	}
	Ok(i)
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
}
