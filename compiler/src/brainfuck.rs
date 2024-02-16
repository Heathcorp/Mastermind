// A lot of this file could be refactored with the knowledge I have now about rust
// this was the first thing added in this project and it has barely changed

use std::{
	fmt,
	io::{Read, Write},
	num::Wrapping,
};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

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
	fn get_cell(&self, index: i32) -> Result<Wrapping<u8>, String> {
		let array;
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
		.or(Err(
			"Could not read current cell due to integer error, this should never occur.",
		))?;

		Ok(match i < array.len() {
			true => array[i],
			false => Wrapping(0u8),
		})
	}
	fn move_head_position(&mut self, amount: i32) {
		self.head_position += amount;
	}
	fn modify_current_cell(
		&mut self,
		amount: Wrapping<u8>,
		clear: Option<bool>,
	) -> Result<(), String> {
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
		.or(Err(
			"Could not modify current cell due to integer error, this should never occur.",
		))?;

		if i >= array.len() {
			array.resize(i + 1, Wrapping(0u8));
		}

		array[i] = match clear.unwrap_or(false) {
			true => amount,
			false => array[i] + amount,
		};

		Ok(())
	}
	fn set_current_cell(&mut self, value: Wrapping<u8>) -> Result<(), String> {
		Ok(self.modify_current_cell(value, Some(true))?)
	}
	// TODO: simplify duplicated code? probably could use an optional mutable reference thing
	fn get_current_cell(&self) -> Result<Wrapping<u8>, String> {
		Ok(self.get_cell(self.head_position)?)
	}
}

impl fmt::Display for Tape {
	// absolutely horrible code here, not even used ever so should just get rid of it
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut line_0 = String::with_capacity(50);
		let mut line_1 = String::with_capacity(50);
		let mut line_2 = String::with_capacity(50);
		let mut line_3 = String::with_capacity(50);
		let mut line_4 = String::with_capacity(50);

		// disgusting
		line_0.push('|');
		line_1.push('|');
		line_2.push('|');
		line_3.push('|');
		line_4.push('|');

		for pos in (self.head_position - 10)..(self.head_position + 10) {
			let val = self.get_cell(pos).unwrap().0;
			let mut dis = 32u8;
			if val.is_ascii_alphanumeric() || val.is_ascii_punctuation() {
				dis = val;
			}

			// dodgy af, I don't know rust or the best way but I know this isn't
			line_0.push_str(format!("{val:03}").as_str());

			line_1.push_str(format!("{:3}", (val as i8)).as_str());

			line_2.push_str(format!(" {val:02x}").as_str());

			line_3.push(' ');
			line_3.push(' ');
			line_3.push(dis as char);

			line_4 += match pos == self.head_position {
				true => "^^^",
				false => "---",
			};

			line_0.push('|');
			line_1.push('|');
			line_2.push('|');
			line_3.push('|');
			line_4.push('|');
		}

		// disgusting but I just want this to work
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_0);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_1);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_2);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_3);
		let _ = f.write_str("\n");
		let _ = f.write_str(&line_4);
		let _ = f.write_str("\n");

		Ok(())
	}
}

pub struct BVM {
	tape: Tape,
	program: Vec<char>,
}

pub trait AsyncByteReader {
	async fn read_byte(&mut self) -> u8;
}

pub trait ByteWriter {
	fn write_byte(&mut self, byte: u8);
}

impl BVM {
	pub fn new(program: Vec<char>) -> Self {
		BVM {
			tape: Tape::new(),
			program,
		}
	}

	// TODO: refactor/rewrite this, can definitely be improved with async read/write traits or similar
	// I don't love that I duplicated this to make it work with js
	pub async fn run_async(
		&mut self,
		output_callback: &js_sys::Function,
		input_callback: &js_sys::Function,
	) -> Result<String, String> {
		let mut pc: usize = 0;
		// this could be more efficient with a pre-computed map
		let mut loop_stack: Vec<usize> = Vec::new();

		let mut output_bytes: Vec<u8> = Vec::new();

		while pc < self.program.len() {
			match self.program[pc] {
				'+' => {
					self.tape.modify_current_cell(Wrapping(1), None)?;
				}
				'-' => {
					self.tape.modify_current_cell(Wrapping(-1i8 as u8), None)?;
				}
				',' => {
					// https://github.com/rustwasm/wasm-bindgen/issues/2195
					// let password_jsval: JsValue = func.call1(&this, &JsValue::from_bool(true))?;
					// let password_promise_res: Result<js_sys::Promise, JsValue> =
					// 	password_jsval.dyn_into();
					// let password_promise = password_promise_res
					// 	.map_err(|_| "Function askUnlockPassword does not return a Promise")
					// 	.map_err(err_to_js)?;
					// let password_jsstring = JsFuture::from(password_promise).await?;
					// let password = password_jsstring
					// 	.as_string()
					// 	.ok_or("Promise didn't return a String")
					// 	.map_err(err_to_js)?;

					// TODO: handle errors
					let jsval = input_callback
						.call0(&JsValue::null())
						.or(Err("failed calling input callback"))?;
					let promise_res: Result<js_sys::Promise, JsValue> = jsval.dyn_into();
					let promise = promise_res.or(Err(
						"failed getting promise from return value of input callback",
					))?;
					let js_num = JsFuture::from(promise)
						.await
						.or(Err("failed getting number from returned promise"))?;
					let num = js_num
						.as_f64()
						.expect("Could not convert js number into f64 type");
					let byte: u8 = num as u8; // I have no idea if this works (TODO: test)
					self.tape.set_current_cell(Wrapping(byte))?;
				}
				'.' => {
					// TODO: handle errors
					let byte = self.tape.get_current_cell()?.0;
					let fnum: f64 = byte as f64; // I have no idea if this works (TODO: test again)
					output_callback
						.call1(&JsValue::null(), &JsValue::from_f64(fnum))
						.or(Err("failed calling output callback"))?;

					output_bytes.push(byte);
				}
				'>' => {
					self.tape.move_head_position(1);
				}
				'<' => {
					self.tape.move_head_position(-1);
				}
				'[' => {
					// entering a loop
					if self.tape.get_current_cell()?.0 == 0 {
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
					if self.tape.get_current_cell()?.0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				// '#' => {
				// 	println!("{}", self.tape);
				// }
				// '@' => {
				// 	print!("{}", self.tape.get_current_cell().0 as i32);
				// }
				_ => (),
			};

			// let s: String = self.program[(cmp::max(0i32, (pc as i32) - 10i32) as usize)
			// 	..(cmp::min(self.program.len() as i32, (pc as i32) + 10i32) as usize)]
			// 	.iter()
			// 	.collect();
			// println!("{s}");
			// println!("{}", self.tape);
			pc += 1;
		}

		Ok(unsafe { String::from_utf8_unchecked(output_bytes) })
	}

	pub fn run(&mut self, input: &mut impl Read, output: &mut impl Write) -> Result<(), String> {
		let mut pc: usize = 0;
		// this could be more efficient with a pre-computed map
		let mut loop_stack: Vec<usize> = Vec::new();

		while pc < self.program.len() {
			match self.program[pc] {
				'+' => {
					self.tape.modify_current_cell(Wrapping(1), None)?;
				}
				'-' => {
					self.tape.modify_current_cell(Wrapping(-1i8 as u8), None)?;
				}
				',' => {
					let mut buf = [0; 1];
					let _ = input.read_exact(&mut buf);
					self.tape.set_current_cell(Wrapping(buf[0]))?;
				}
				'.' => {
					let buf = [self.tape.get_current_cell()?.0];
					let _ = output.write(&buf);
				}
				'>' => {
					self.tape.move_head_position(1);
				}
				'<' => {
					self.tape.move_head_position(-1);
				}
				'[' => {
					// entering a loop
					if self.tape.get_current_cell()?.0 == 0 {
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
					if self.tape.get_current_cell()?.0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				// '#' => {
				// 	println!("{}", self.tape);
				// }
				// '@' => {
				// 	print!("{}", self.tape.get_current_cell().0 as i32);
				// }
				_ => (),
			};

			// let s: String = self.program[(cmp::max(0i32, (pc as i32) - 10i32) as usize)
			// 	..(cmp::min(self.program.len() as i32, (pc as i32) + 10i32) as usize)]
			// 	.iter()
			// 	.collect();
			// println!("{s}");
			// println!("{}", self.tape);
			pc += 1;
		}

		Ok(())
	}
}

#[cfg(test)]
pub mod tests {
	// TODO: add unit tests for Tape
	use super::*;

	use std::io::Cursor;

	pub fn run_code(code: String, input: String) -> String {
		let mut bvm = BVM::new(code.chars().collect());

		let input_bytes: Vec<u8> = input.bytes().collect();
		let mut input_stream = Cursor::new(input_bytes);
		let mut output_stream = Cursor::new(Vec::new());

		bvm.run(&mut input_stream, &mut output_stream).unwrap();

		// TODO: fix this unsafe stuff
		unsafe { String::from_utf8_unchecked(output_stream.into_inner()) }
	}

	#[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		assert_eq!(desired_output, run_code(program, input))
	}

	#[test]
	fn hello_world_1() {
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(desired_output, run_code(program, input))
	}

	#[test]
	fn hello_world_2() {
		let program = String::from(
			"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.",
		);
		let input = String::from("");
		let desired_output = String::from("Hello, World!");
		assert_eq!(desired_output, run_code(program, input))
	}

	#[test]
	fn random_mess() {
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(desired_output, run_code(program, input))
	}
}
