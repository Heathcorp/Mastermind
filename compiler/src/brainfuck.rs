// A lot of this file could be refactored with the knowledge I have now about rust
// this was the first thing added in this project and it has barely changed

use std::{
	collections::HashMap,
	io::{Read, Write},
	num::Wrapping,
};

use crate::{
	backend::{bf2d::TapeCell2D, common::TapeCellVariant},
	macros::macros::r_panic,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

struct Tape<TC: TapeCellVariant> {
	memory_map: HashMap<TC, Wrapping<u8>>,
	head_position: TC,
}

impl Tape<TapeCell2D> {
	fn new() -> Self {
		Tape {
			memory_map: HashMap::new(),
			head_position: TapeCell2D(0, 0),
		}
	}
	fn get_cell(&self, position: TapeCell2D) -> Wrapping<u8> {
		match self.memory_map.get(&position) {
			Some(val) => *val,
			None => Wrapping(0),
		}
	}
	fn move_head_position(&mut self, amount: TapeCell2D) {
		self.head_position.0 += amount.0;
		self.head_position.1 += amount.1;
	}
	fn increment_current_cell(&mut self, amount: Wrapping<u8>) {
		let val = self.memory_map.get_mut(&self.head_position);
		match val {
			Some(val) => {
				*val += amount;
			}
			None => {
				self.memory_map.insert(self.head_position, amount);
			}
		}
	}
	fn set_current_cell(&mut self, value: Wrapping<u8>) {
		self.memory_map.insert(self.head_position, value);
	}
	// TODO: simplify duplicated code? probably could use an optional mutable reference thing
	fn get_current_cell(&self) -> Wrapping<u8> {
		match self.memory_map.get(&self.head_position) {
			Some(val) => *val,
			None => Wrapping(0),
		}
	}
}

pub struct BrainfuckConfig {
	pub enable_debug_symbols: bool,
	pub enable_2d_grid: bool,
}

pub struct BrainfuckContext {
	pub config: BrainfuckConfig,
}

pub trait AsyncByteReader {
	async fn read_byte(&mut self) -> u8;
}

pub trait ByteWriter {
	fn write_byte(&mut self, byte: u8);
}

impl BrainfuckContext {
	const MAX_STEPS_DEFAULT: usize = (2 << 30) - 2;

	// TODO: refactor/rewrite this, can definitely be improved with async read/write traits or similar
	// I don't love that I duplicated this to make it work with js
	// TODO: this isn't covered by unit tests
	pub async fn run_async(
		&self,
		program: Vec<char>,
		output_callback: &js_sys::Function,
		input_callback: &js_sys::Function,
	) -> Result<String, String> {
		let mut tape = Tape::new();
		let mut pc: usize = 0;
		// this could be more efficient with a pre-computed map
		let mut loop_stack: Vec<usize> = Vec::new();

		let mut output_bytes: Vec<u8> = Vec::new();

		while pc < program.len() {
			match (
				program[pc],
				self.config.enable_debug_symbols,
				self.config.enable_2d_grid,
			) {
				('+', _, _) => tape.increment_current_cell(Wrapping(1)),
				('-', _, _) => tape.increment_current_cell(Wrapping(-1i8 as u8)),
				(',', _, _) => {
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
					tape.set_current_cell(Wrapping(byte));
				}
				('.', _, _) => {
					// TODO: handle errors
					let byte = tape.get_current_cell().0;
					let fnum: f64 = byte as f64; // I have no idea if this works (TODO: test again)
					output_callback
						.call1(&JsValue::null(), &JsValue::from_f64(fnum))
						.or(Err("failed calling output callback"))?;

					output_bytes.push(byte);
				}
				('>', _, _) => {
					tape.move_head_position(TapeCell2D(1, 0));
				}
				('<', _, _) => {
					tape.move_head_position(TapeCell2D(-1, 0));
				}
				('[', _, _) => {
					// entering a loop
					if tape.get_current_cell().0 == 0 {
						// skip the loop, (advance to the corresponding closing loop brace)
						// TODO: make this more efficient by pre-computing a loops map
						let mut loop_count = 1;
						while loop_count > 0 {
							pc += 1;
							loop_count += match program[pc] {
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
				(']', _, _) => {
					if tape.get_current_cell().0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				('^', _, true) => tape.move_head_position(TapeCell2D(0, 1)),
				('v', _, true) => tape.move_head_position(TapeCell2D(0, -1)),
				('^', _, false) => {
					r_panic!("2D Brainfuck currently disabled");
				}
				('v', _, false) => {
					r_panic!("2D Brainfuck currently disabled");
				}
				// ('#', true, ) => {
				// 	println!("{}", tape);
				// }
				// ('@', true, _) => {
				// 	print!("{}", tape.get_current_cell().0 as i32);
				// }
				_ => (),
			};

			// let s: String = self.program[(cmp::max(0i32, (pc as i32) - 10i32) as usize)
			// 	..(cmp::min(self.program.len() as i32, (pc as i32) + 10i32) as usize)]
			// 	.iter()
			// 	.collect();
			// println!("{s}");
			// println!("{}", tape);
			pc += 1;
		}

		Ok(unsafe { String::from_utf8_unchecked(output_bytes) })
	}

	pub fn run(
		&self,
		program: Vec<char>,
		input: &mut impl Read,
		output: &mut impl Write,
		max_steps: Option<usize>,
	) -> Result<(), String> {
		let mut tape = Tape::new();
		let mut steps = 0usize;
		let mut pc: usize = 0;
		// this could be more efficient with a pre-computed map
		let mut loop_stack: Vec<usize> = Vec::new();

		while pc < program.len() {
			match (
				program[pc],
				self.config.enable_debug_symbols,
				self.config.enable_2d_grid,
			) {
				('+', _, _) => tape.increment_current_cell(Wrapping(1)),
				('-', _, _) => tape.increment_current_cell(Wrapping(-1i8 as u8)),
				(',', _, _) => {
					let mut buf = [0; 1];
					let _ = input.read_exact(&mut buf);
					tape.set_current_cell(Wrapping(buf[0]));
				}
				('.', _, _) => {
					let buf = [tape.get_current_cell().0];
					let _ = output.write(&buf);
				}
				('>', _, _) => {
					tape.move_head_position(TapeCell2D(1, 0));
				}
				('<', _, _) => {
					tape.move_head_position(TapeCell2D(-1, 0));
				}
				('[', _, _) => {
					// entering a loop
					if tape.get_current_cell().0 == 0 {
						// skip the loop, (advance to the corresponding closing loop brace)
						// TODO: make this more efficient by pre-computing a loops map
						let mut loop_count = 1;
						while loop_count > 0 {
							pc += 1;
							loop_count += match program[pc] {
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
				(']', _, _) => {
					if tape.get_current_cell().0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				('^', _, true) => tape.move_head_position(TapeCell2D(0, 1)),
				('v', _, true) => tape.move_head_position(TapeCell2D(0, -1)),
				('^', _, false) => {
					r_panic!("2D Brainfuck currently disabled");
				}
				('v', _, false) => {
					r_panic!("2D Brainfuck currently disabled");
				}
				// '#' => {
				// 	println!("{}", tape);
				// }
				// '@' => {
				// 	print!("{}", tape.get_current_cell().0 as i32);
				// }
				_ => (),
			};

			// let s: String = self.program[(cmp::max(0i32, (pc as i32) - 10i32) as usize)
			// 	..(cmp::min(self.program.len() as i32, (pc as i32) + 10i32) as usize)]
			// 	.iter()
			// 	.collect();
			// println!("{s}");
			// println!("{}", tape);
			pc += 1;

			// cut the program short if it runs forever
			steps += 1;
			if steps > max_steps.unwrap_or(Self::MAX_STEPS_DEFAULT) {
				// not sure if this should error out or just quit silently
				return Err(String::from(
					"Max steps reached in BVM, possibly an infinite loop.",
				));
			}
		}

		Ok(())
	}
}

#[cfg(test)]
pub mod bvm_tests {
	// TODO: add unit tests for Tape
	use super::*;

	use std::io::Cursor;

	pub fn run_code(
		config: BrainfuckConfig,
		code: String,
		input: String,
		max_steps_cutoff: Option<usize>,
	) -> String {
		let ctx = BrainfuckContext { config };

		let input_bytes: Vec<u8> = input.bytes().collect();
		let mut input_stream = Cursor::new(input_bytes);
		let mut output_stream = Cursor::new(Vec::new());

		ctx.run(
			code.chars().collect(),
			&mut input_stream,
			&mut output_stream,
			max_steps_cutoff,
		)
		.unwrap();

		// TODO: fix this unsafe stuff
		unsafe { String::from_utf8_unchecked(output_stream.into_inner()) }
	}
	const BVM_CONFIG_1D: BrainfuckConfig = BrainfuckConfig {
		enable_debug_symbols: false,
		enable_2d_grid: false,
	};
	const BVM_CONFIG_2D: BrainfuckConfig = BrainfuckConfig {
		enable_debug_symbols: false,
		enable_2d_grid: true,
	};

	#[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_1D, program, input, None)
		)
	}

	#[test]
	fn hello_world_1() {
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_1D, program, input, None)
		)
	}

	#[test]
	fn hello_world_2() {
		let program = String::from(
			"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.",
		);
		let input = String::from("");
		let desired_output = String::from("Hello, World!");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_1D, program, input, None)
		)
	}

	#[test]
	fn random_mess() {
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_1D, program, input, None)
		)
	}

	#[test]
	#[should_panic(expected = "2D Brainfuck currently disabled")]
	fn grid_disabled_1() {
		let program = String::from("++++++++[->++++++[->+>+<<]<]>>.>^+++.");
		let input = String::from("");
		run_code(BVM_CONFIG_1D, program, input, None);
	}

	#[test]
	#[should_panic(expected = "2D Brainfuck currently disabled")]
	fn grid_disabled_2() {
		let program =
			String::from("++++++++[->^^^+++vvvv+++[->^^^^+>+<vvvv<]<]>^^^^^^^^>.>vvvv+++.");
		let input = String::from("");
		run_code(BVM_CONFIG_1D, program, input, None);
	}

	// 2D tests:
	#[test]
	fn grid_regression_1() {
		// hello world
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn grid_regression_2() {
		// random mess
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn grid_basic_1() {
		let program = String::from("++++++++[-^++++++[->+v+<^]v]>+++++^.v.");
		let input = String::from("");
		let desired_output = String::from("05");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn grid_mover_1() {
		let program = String::from(
			"-<<<<<<<<<<<<^^^^^^^^^^^^-<^++++++++[->>vv+[->v+]->v++++++<^<^+[-<^+]-<^]>>vv+[->v+]->v...",
		);
		let input = String::from("");
		let desired_output = String::from("000");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn grid_bfception_1() {
		// run a hello world program within a 1d brainfuck interpreter implemented in 2d brainfuck
		let program = String::from("-v>,[>,]^-<+[-<+]->+[-v------------------------------------------^>+]-<+[-<+]->+[-v[-^+^+vv]^[-v+^]^->+<[>-<->+<[>-<->+<[>-<->+<[>-<-------------->+<[>-<-->+<[>-<----------------------------->+<[>-<-->+<[>-<vv[-]^^[-]]>[[-]<[-]vv[-]++++++v++^^^>]<[-]]>[[-]<[-]vv[-]+++++v+^^^>]<[-]]>[[-]<[-]vv[-]+++^^>]<[-]]>[[-]<[-]vv[-]++++^^>]<[-]]>[[-]<[-]vv[-]+++++++^^>]<[-]]>[[-]<[-]vv[-]++^^>]<[-]]>[[-]<[-]vv[-]++++++++^^>]<[-]]>[[-]<[-]vv[-]+^^>]<vv^>+]-v-v-v-v-^^^^<+[-<+]<->v-v-v<-v->^^^^>vvv+^^^<+>+[-<->+v[-^^+^+vvv]^^[-vv+^^]^>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<[-]]>[-<vvvvv+[-<+]->-[+>-]+v,^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v.^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v[[-]^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]<+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<]>[-<vv+[-<+]-<->>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv]^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v>+<[>-<[-]]>[-<^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]>+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<->>-[+>-]+^^>]<]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv>]<^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+<<-v-^>+v+^[<+v+^>-v-^]+>-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+>>-v-^<+v+^[>+v+^<-v-^]+<-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v-^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v+^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<vv>+]-");
		let input = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.\n");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn grid_bfception_2() {
		// random mess
		let program = String::from("-v>,[>,]^-<+[-<+]->+[-v------------------------------------------^>+]-<+[-<+]->+[-v[-^+^+vv]^[-v+^]^->+<[>-<->+<[>-<->+<[>-<->+<[>-<-------------->+<[>-<-->+<[>-<----------------------------->+<[>-<-->+<[>-<vv[-]^^[-]]>[[-]<[-]vv[-]++++++v++^^^>]<[-]]>[[-]<[-]vv[-]+++++v+^^^>]<[-]]>[[-]<[-]vv[-]+++^^>]<[-]]>[[-]<[-]vv[-]++++^^>]<[-]]>[[-]<[-]vv[-]+++++++^^>]<[-]]>[[-]<[-]vv[-]++^^>]<[-]]>[[-]<[-]vv[-]++++++++^^>]<[-]]>[[-]<[-]vv[-]+^^>]<vv^>+]-v-v-v-v-^^^^<+[-<+]<->v-v-v<-v->^^^^>vvv+^^^<+>+[-<->+v[-^^+^+vvv]^^[-vv+^^]^>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<[-]]>[-<vvvvv+[-<+]->-[+>-]+v,^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v.^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v[[-]^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]<+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<]>[-<vv+[-<+]-<->>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv]^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v>+<[>-<[-]]>[-<^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]>+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<->>-[+>-]+^^>]<]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv>]<^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+<<-v-^>+v+^[<+v+^>-v-^]+>-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+>>-v-^<+v+^[>+v+^<-v-^]+<-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v-^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v+^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<vv>+]-");
		let input = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.\n");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}

	#[test]
	fn test_bf2d_code() {
		let program = String::from(
			",.[-]+[--^-[^^+^-----vv]v--v---]^-.^^^+.^^..+++[.^]vvvv.+++.------.vv-.^^^^+.",
		);
		let input = String::from("");
		let desired_output = String::from("\0Hello, World!");
		assert_eq!(
			desired_output,
			run_code(BVM_CONFIG_2D, program, input, None)
		)
	}
}
