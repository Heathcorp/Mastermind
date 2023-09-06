use std::{fmt, io::Read, io::Write, num::Wrapping};

struct Tape {
	positive_array: Vec<Wrapping<u8>>,
	negative_array: Vec<Wrapping<u8>>,

	head_position: i32,
}

impl Tape {
	fn new() -> Self {
		Tape {
			positive_array: Vec::new(),
			negative_array: Vec::new(),
			head_position: 0,
		}
	}
	fn get_cell(&self, index: i32) -> Wrapping<u8> {
		let mut array;
		let i: usize = (match index < 0 {
			true => {
				array = &self.negative_array;
				-index - 1
			}
			false => {
				array = &self.positive_array;
				index
			}
		})
		.try_into()
		.unwrap();

		match i < array.len() {
			true => array[i],
			false => Wrapping(0u8),
		}
	}
	fn move_head_position(&mut self, amount: i32) {
		self.head_position += amount;
	}
	fn modify_current_cell(&mut self, amount: Wrapping<u8>, clear: Option<bool>) {
		let array;
		let i: usize = (match self.head_position < 0 {
			true => {
				array = &mut self.negative_array;
				-self.head_position - 1
			}
			false => {
				array = &mut self.positive_array;
				self.head_position
			}
		})
		.try_into()
		.unwrap();

		if i >= array.len() {
			array.resize(i + 1, Wrapping(0u8));
		}

		array[i] = match clear.unwrap_or(false) {
			true => amount,
			false => array[i] + amount,
		};
	}
	fn set_current_cell(&mut self, value: Wrapping<u8>) {
		self.modify_current_cell(value, Some(true));
	}
	// TODO: simplify duplicated code? probably could use an optional mutable reference thing
	fn get_current_cell(&self) -> Wrapping<u8> {
		self.get_cell(self.head_position)
	}
}

impl fmt::Display for Tape {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut line_1 = String::with_capacity(50);
		let mut line_2 = String::with_capacity(50);
		let mut line_3 = String::with_capacity(50);

		// disgusting
		line_1.push('|');
		line_2.push('|');
		line_3.push('|');

		for (i, pos) in ((self.head_position - 10)..(self.head_position + 10)).enumerate() {
			let val = self.get_cell(pos).0;
			let mut dis = 32u8;
			if val.is_ascii_alphanumeric() || val.is_ascii_punctuation() {
				dis = val;
			}

			// dodgy af, I don't know rust or the best way but I know this isn't
			line_1.push_str(format!("{val:02x}").as_str());
			line_2.push(' ');
			line_2.push(dis as char);
			line_3 += match pos == self.head_position {
				true => "^^",
				false => "--",
			};

			line_1.push('|');
			line_2.push('|');
			line_3.push('|');
		}

		// disgusting but I just want this to work
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_1);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_2);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_3);
		let _ = f.write_str("\n");

		Ok(())
	}
}

pub struct BVM {
	tape: Tape,
	program: Vec<char>,
}

impl BVM {
	pub fn new(program: Vec<char>) -> Self {
		BVM {
			tape: Tape::new(),
			program,
		}
	}
	pub fn run(&mut self, input: &mut impl Read, output: &mut impl Write) {
		let mut pc: usize = 0;
		// this could be more efficient with a pre-computed map
		let mut loop_stack: Vec<usize> = Vec::new();

		while pc < self.program.len() {
			match self.program[pc] {
				'+' => {
					self.tape.modify_current_cell(Wrapping(1), None);
				}
				'-' => {
					self.tape.modify_current_cell(Wrapping(255), None);
				}
				',' => {
					let mut buf = [0; 1];
					input.read_exact(&mut buf);
					self.tape.set_current_cell(Wrapping(buf[0]));
				}
				'.' => {
					let buf = [self.tape.get_current_cell().0];
					output.write(&buf);
				}
				'>' => {
					self.tape.move_head_position(1);
				}
				'<' => {
					self.tape.move_head_position(-1);
				}
				'[' => {
					// entering a loop
					if self.tape.get_current_cell().0 == 0 {
						// skip the loop, (advance to the corresponding closing loop brace)
						// TODO: make this more efficient by pre-computing a loops map
						let mut loop_count = 1;
						while loop_count > 0 {
							pc += 1;
							loop_count += match self.program[pc] {
								'[' => 1,
								']' => -1,
								_ => 0,
							}
						}
					} else {
						// add the open loop to the stack and proceed
						loop_stack.push(pc);
					}
				}
				']' => {
					if self.tape.get_current_cell().0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				'#' => {
					println!("{}", self.tape);
				}
				_ => (),
			};

			pc += 1;
		}
	}
}

#[cfg(test)]
mod tests {
	// TODO: add unit tests for Tape
	use super::*;

	use std::io::Cursor;

	fn run_program(program: String, input: String) -> String {
		let mut bvm = BVM::new(program.chars().collect());

		let input_bytes: Vec<u8> = input.bytes().collect();
		let mut input_stream = Cursor::new(input_bytes);
		let mut output_stream = Cursor::new(Vec::new());

		bvm.run(&mut input_stream, &mut output_stream);

		// TODO: fix this unsafe stuff
		unsafe { String::from_utf8_unchecked(output_stream.into_inner()) }
	}

	#[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		assert_eq!(desired_output, run_program(program, input))
	}

	#[test]
	fn hello_world_1() {
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(desired_output, run_program(program, input))
	}

	#[test]
	fn hello_world_2() {
		let program = String::from(
			"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.",
		);
		let input = String::from("");
		let desired_output = String::from("Hello, World!");
		assert_eq!(desired_output, run_program(program, input))
	}

	#[test]
	fn random_mess() {
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(desired_output, run_program(program, input))
	}
}
