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

// TODO: figure out errors for these helper functions
pub fn find_next(chars: &[char], character: char) -> Result<usize, ()> {
	let mut i = 0;
	loop {
		let Some(c) = chars.get(i) else {
			return Err(());
		};

		if c == character {
			break;
		}
		i += 1;
	}
	Ok(i)
}

pub fn find_and_advance<'a>(chars: &'a mut &[char], character: char) -> Result<&'a [char], ()> {
	let substr_len = find_next(chars, character)?;
	let chars_before = chars[..substr_len];
	chars = chars[substr_len..];
	chars_before
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
