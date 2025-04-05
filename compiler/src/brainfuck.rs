// A lot of this file could be refactored with the knowledge I have now about rust
// this was the first thing added in this project and it has barely changed

use std::{
	collections::HashMap,
	fmt,
	io::{Read, Write},
	num::Wrapping,
};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

struct Tape {
	memory_map: HashMap<(i32, i32), Wrapping<u8>>,
	head_position: (i32, i32),
}

impl Tape {
	fn new() -> Self {
		Tape {
			memory_map: HashMap::new(),
			head_position: (0, 0),
		}
	}
	fn get_cell(&self, position: (i32, i32)) -> Wrapping<u8> {
		match self.memory_map.get(&position) {
			Some(val) => *val,
			None => Wrapping(0),
		}
	}
	fn move_head_position(&mut self, amount: (i32, i32)) {
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

		for pos in (self.head_position.1 - 10)..(self.head_position.1 + 10) {
			let val = self.get_cell((pos, 0)).0;
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

			line_4 += match pos == self.head_position.1 {
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

pub struct BVMConfig {
	pub ENABLE_DEBUG_SYMBOLS: bool,
	pub ENABLE_2D_GRID: bool,
}

pub struct BVM {
	config: BVMConfig,
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
	pub fn new(config: BVMConfig, program: Vec<char>) -> Self {
		BVM {
			config,
			tape: Tape::new(),
			program,
		}
	}

	// TODO: refactor/rewrite this, can definitely be improved with async read/write traits or similar
	// I don't love that I duplicated this to make it work with js
	// TODO: this isn't covered by unit tests
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
			match (
				self.program[pc],
				self.config.ENABLE_DEBUG_SYMBOLS,
				self.config.ENABLE_2D_GRID,
			) {
				('+', _, _) => self.tape.increment_current_cell(Wrapping(1)),
				('-', _, _) => self.tape.increment_current_cell(Wrapping(-1i8 as u8)),
				(',', _, _) => {
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
					self.tape.set_current_cell(Wrapping(byte));
				}
				('.', _, _) => {
					// TODO: handle errors
					let byte = self.tape.get_current_cell().0;
					let fnum: f64 = byte as f64; // I have no idea if this works (TODO: test again)
					output_callback
						.call1(&JsValue::null(), &JsValue::from_f64(fnum))
						.or(Err("failed calling output callback"))?;

					output_bytes.push(byte);
				}
				('>', _, _) => {
					self.tape.move_head_position((1, 0));
				}
				('<', _, _) => {
					self.tape.move_head_position((-1, 0));
				}
				('[', _, _) => {
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
				(']', _, _) => {
					if self.tape.get_current_cell().0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				('^', _, true) => self.tape.move_head_position((0, 1)),
				('v', _, true) => self.tape.move_head_position((0, -1)),
				// ('#', true, ) => {
				// 	println!("{}", self.tape);
				// }
				// ('@', true, _) => {
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
			match (
				self.program[pc],
				self.config.ENABLE_DEBUG_SYMBOLS,
				self.config.ENABLE_2D_GRID,
			) {
				('+', _, _) => self.tape.increment_current_cell(Wrapping(1)),
				('-', _, _) => self.tape.increment_current_cell(Wrapping(-1i8 as u8)),
				(',', _, _) => {
					let mut buf = [0; 1];
					let _ = input.read_exact(&mut buf);
					self.tape.set_current_cell(Wrapping(buf[0]));
				}
				('.', _, _) => {
					let buf = [self.tape.get_current_cell().0];
					let _ = output.write(&buf);
				}
				('>', _, _) => {
					self.tape.move_head_position((1, 0));
				}
				('<', _, _) => {
					self.tape.move_head_position((-1, 0));
				}
				('[', _, _) => {
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
				(']', _, _) => {
					if self.tape.get_current_cell().0 == 0 {
						// exit the loop
						loop_stack.pop();
					} else {
						// cell isn't 0 so jump back to corresponding opening loop brace
						// not sure what rust will do if the stack is empty
						pc = loop_stack[loop_stack.len() - 1];
					}
				}
				('^', _, true) => self.tape.move_head_position((0, 1)),
				('v', _, true) => self.tape.move_head_position((0, -1)),
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

	pub fn run_code(config: BVMConfig, code: String, input: String) -> String {
		let mut bvm = BVM::new(config, code.chars().collect());

		let input_bytes: Vec<u8> = input.bytes().collect();
		let mut input_stream = Cursor::new(input_bytes);
		let mut output_stream = Cursor::new(Vec::new());

		bvm.run(&mut input_stream, &mut output_stream).unwrap();

		// TODO: fix this unsafe stuff
		unsafe { String::from_utf8_unchecked(output_stream.into_inner()) }
	}
	const BVM_CONFIG_1D: BVMConfig = BVMConfig {
		ENABLE_DEBUG_SYMBOLS: false,
		ENABLE_2D_GRID: false,
	};
	const BVM_CONFIG_2D: BVMConfig = BVMConfig {
		ENABLE_DEBUG_SYMBOLS: false,
		ENABLE_2D_GRID: true,
	};

	#[test]
	fn dummy_test() {
		let program = String::from("");
		let input = String::from("");
		let desired_output = String::from("");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	#[test]
	fn hello_world_1() {
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	#[test]
	fn hello_world_2() {
		let program = String::from(
			"+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.",
		);
		let input = String::from("");
		let desired_output = String::from("Hello, World!");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	#[test]
	fn random_mess() {
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	#[test]
	fn grid_disabled_1() {
		let program = String::from("++++++++[->++++++[->+>+<<]<]>>.>^+++.");
		let input = String::from("");
		let desired_output = String::from("03");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	#[test]
	fn grid_disabled_2() {
		let program =
			String::from("++++++++[->^^^+++vvvv+++[->^^^^+>+<vvvv<]<]>^^^^^^^^>.>vvvv+++.");
		let input = String::from("");
		let desired_output = String::from("03");
		assert_eq!(desired_output, run_code(BVM_CONFIG_1D, program, input))
	}

	// 2D tests:
	#[test]
	fn grid_regression_1() {
		// hello world
		let program = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
		let input = String::from("");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}

	#[test]
	fn grid_regression_2() {
		// random mess
		let program = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.");
		let input = String::from("");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}

	#[test]
	fn grid_basic_1() {
		let program = String::from("++++++++[-^++++++[->+v+<^]v]>+++++^.v.");
		let input = String::from("");
		let desired_output = String::from("05");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}

	#[test]
	fn grid_mover_1() {
		let program = String::from(
			"-<<<<<<<<<<<<^^^^^^^^^^^^-<^++++++++[->>vv+[->v+]->v++++++<^<^+[-<^+]-<^]>>vv+[->v+]->v...",
		);
		let input = String::from("");
		let desired_output = String::from("000");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}

	#[test]
	fn grid_bfception_1() {
		// run a hello world program within a 1d brainfuck interpreter implemented in 2d brainfuck
		let program = String::from("-v>,[>,]^-<+[-<+]->+[-v------------------------------------------^>+]-<+[-<+]->+[-v[-^+^+vv]^[-v+^]^->+<[>-<->+<[>-<->+<[>-<->+<[>-<-------------->+<[>-<-->+<[>-<----------------------------->+<[>-<-->+<[>-<vv[-]^^[-]]>[[-]<[-]vv[-]++++++v++^^^>]<[-]]>[[-]<[-]vv[-]+++++v+^^^>]<[-]]>[[-]<[-]vv[-]+++^^>]<[-]]>[[-]<[-]vv[-]++++^^>]<[-]]>[[-]<[-]vv[-]+++++++^^>]<[-]]>[[-]<[-]vv[-]++^^>]<[-]]>[[-]<[-]vv[-]++++++++^^>]<[-]]>[[-]<[-]vv[-]+^^>]<vv^>+]-v-v-v-v-^^^^<+[-<+]<->v-v-v<-v->^^^^>vvv+^^^<+>+[-<->+v[-^^+^+vvv]^^[-vv+^^]^>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<[-]]>[-<vvvvv+[-<+]->-[+>-]+v,^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v.^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v[[-]^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]<+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<]>[-<vv+[-<+]-<->>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv]^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v>+<[>-<[-]]>[-<^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]>+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<->>-[+>-]+^^>]<]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv>]<^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+<<-v-^>+v+^[<+v+^>-v-^]+>-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+>>-v-^<+v+^[>+v+^<-v-^]+<-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v-^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v+^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<vv>+]-");
		let input = String::from("++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.\n");
		let desired_output = String::from("Hello World!\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}

	#[test]
	fn grid_bfception_2() {
		// random mess
		let program = String::from("-v>,[>,]^-<+[-<+]->+[-v------------------------------------------^>+]-<+[-<+]->+[-v[-^+^+vv]^[-v+^]^->+<[>-<->+<[>-<->+<[>-<->+<[>-<-------------->+<[>-<-->+<[>-<----------------------------->+<[>-<-->+<[>-<vv[-]^^[-]]>[[-]<[-]vv[-]++++++v++^^^>]<[-]]>[[-]<[-]vv[-]+++++v+^^^>]<[-]]>[[-]<[-]vv[-]+++^^>]<[-]]>[[-]<[-]vv[-]++++^^>]<[-]]>[[-]<[-]vv[-]+++++++^^>]<[-]]>[[-]<[-]vv[-]++^^>]<[-]]>[[-]<[-]vv[-]++++++++^^>]<[-]]>[[-]<[-]vv[-]+^^>]<vv^>+]-v-v-v-v-^^^^<+[-<+]<->v-v-v<-v->^^^^>vvv+^^^<+>+[-<->+v[-^^+^+vvv]^^[-vv+^^]^>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<>+<-[>[-]<[-]]>[-<vvvvv+[-<+]->-[+>-]+v,^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v.^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v[[-]^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]<+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<]>[-<vv+[-<+]-<->>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv]^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v[-v+v+^^]v[-^+v]v>+<[>-<[-]]>[-<^^^+[-<+]-^^^+[->+]-<+[>>-[+>-]>+vv[-^^^+^+vvvv]^^^[-vvv+^^^]^->+<[>-<->+<[>-<[-]]>[-<vv+[-<+]-<->>-[+>-]+^^>]<]>[-<vv+[-<+]-<+>>-[+>-]+^^>]<vv+[-<+]-<][-]>vvv+[-<+]->-[+>-]+vvv>]<^^^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+<<-v-^>+v+^[<+v+^>-v-^]+>-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+>>-v-^<+v+^[>+v+^<-v-^]+<-+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v-^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<]>[-<vvvvv+[-<+]->-[+>-]+v+^+[-<+]-<^^^+[->+]->-[+>-]+^^>]<vv>+]-");
		let input = String::from("+++++[>+++++[>++>++>+++>+++>++++>++++<<<<<<-]<-]+++++[>>[>]<[+.<<]>[++.>>>]<[+.<]>[-.>>]<[-.<<<]>[.>]<[+.<]<-]++++++++++.\n");
		let desired_output = String::from("eL34NfeOL454KdeJ44JOdefePK55gQ67ShfTL787KegJ77JTeghfUK88iV9:XjgYL:;:KfiJ::JYfijgZK;;k[<=]lh^L=>=KgkJ==J^gklh_K>>m`?@bnicL@A@KhmJ@@JchmnidKAA\n");
		assert_eq!(desired_output, run_code(BVM_CONFIG_2D, program, input))
	}
}
