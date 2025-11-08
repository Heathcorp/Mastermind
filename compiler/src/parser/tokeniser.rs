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
	OpenBrace,
	ClosingBrace,
	OpenSquareBracket,
	ClosingSquareBracket,
	OpenParenthesis,
	ClosingParenthesis,
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
	PlusEquals,
	MinusEquals,
	EqualsSign,
	Semicolon,
}

/// Get the next token from chars, advance the passed in pointer
pub fn next_token(chars: &mut &[char]) -> Result<Token, ()> {
	// skip any whitespace
	skip_whitespace(chars)?;

	// TODO: this is flawed, what about cell g=5;?
	let token_len = find_next_whitespace(*chars)?;

	Ok(match token_len {
		0 => return Err(()),
		1 => match chars[0] {
			'{' => Token::OpenBrace,
			'}' => Token::ClosingBrace,
			_ => todo!(),
		},
		2 => match chars[0..2] {
			['b', 'f'] => Token::Bf,
			['i', 'f'] => Token::If,
			_ => todo!(),
		},
		3 => match chars[0..3] {
			['n', 'o', 't'] => Token::Not,
			_ => todo!(),
		},
		4 => match chars[0..4] {
			['c', 'e', 'l', 'l'] => Token::Cell,
			['e', 'l', 's', 'e'] => Token::Else,
			['t', 'r', 'u', 'e'] => Token::True,
			_ => todo!(),
		},
		5 => match chars[0..5] {
			['w', 'h', 'i', 'l', 'e'] => Token::While,
			_ => todo!(),
		},
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

pub fn skip_whitespace(chars: &mut &[char]) -> Result<(), ()> {
	loop {
		let Some(c) = chars.get(0) else {
			return Err(());
		};

		if !c.is_whitespace() {
			break;
		}
		*chars = &chars[1..];
	}
	Ok(())
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
				r_panic!("Invlid token in input.");
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
	fn keywords_1() {
		_tokenisation_test(
			"while output input if",
			&[Token::While, Token::Output, Token::Input, Token::If],
		);
	}

	#[test]
	fn keywords_2() {
		_tokenisation_test(
			"into clobbers assert bf else;;;;",
			&[
				Token::Into,
				Token::Clobbers,
				Token::Assert,
				Token::Bf,
				Token::Else,
				Token::Semicolon,
				Token::Semicolon,
				Token::Semicolon,
				Token::Semicolon,
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
				Token::OpenBrace,
				Token::Name(String::from("If")),
				Token::Semicolon,
				Token::Name(String::from("elSe")),
				Token::Name(String::from("___if")),
				Token::ClosingBrace,
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
