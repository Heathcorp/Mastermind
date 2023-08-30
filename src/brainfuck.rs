use std::{io::Read, io::Write, num::Wrapping};

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
  fn move_head_position(&mut self, amount: i32) {
    self.head_position += amount;
  }
  // TODO: simplify all this duplicated code
  fn modify_current_cell(&mut self, amount: Wrapping<u8>) {
    if self.head_position < 0 {
      let i: usize = (-1 - self.head_position).try_into().unwrap();
      if i >= self.negative_array.len() {
        self.negative_array.resize(i + 1, Wrapping(0u8));
      }
      self.negative_array[i] += amount;
    } else {
      let i: usize = self.head_position.try_into().unwrap();
      if i >= self.positive_array.len() {
        self.positive_array.resize(i + 1, Wrapping(0u8));
      }
      self.positive_array[i] += amount;
    }
  }
  fn set_current_cell(&mut self, value: Wrapping<u8>) {
    if self.head_position < 0 {
      let i: usize = (-1 - self.head_position).try_into().unwrap();
      if i >= self.negative_array.len() {
        self.negative_array.resize(i + 1, Wrapping(0u8));
      }
      self.negative_array[i] = value;
    } else {
      let i: usize = self.head_position.try_into().unwrap();
      if i >= self.positive_array.len() {
        self.positive_array.resize(i + 1, Wrapping(0u8));
      }
      self.positive_array[i] = value;
    }
  }
  fn get_current_cell(&self) -> Wrapping<u8> {
    if self.head_position < 0 {
      let i: usize = (-1 - self.head_position).try_into().unwrap();
      if i >= self.negative_array.len() {
        return Wrapping(0u8);
      } else {
        return self.negative_array[i];
      }
    } else {
      let i: usize = self.head_position.try_into().unwrap();
      if i >= self.positive_array.len() {
        return Wrapping(0u8);
      } else {
        return self.positive_array[i];
      }
    }
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
        '+' => {self.tape.modify_current_cell(Wrapping(1));},
        '-' => {self.tape.modify_current_cell(Wrapping(255u8));},
        ',' => {
          let mut buf = [0; 1];
          input.read_exact(&mut buf);
          self.tape.set_current_cell(Wrapping(buf[0]));
        },
        '.' => {
          let buf = [self.tape.get_current_cell().0];
          output.write(&buf);
        },
        '>' => {self.tape.move_head_position(1);},
        '<' => {self.tape.move_head_position(-1);},
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
        },
        ']' => {
          if self.tape.get_current_cell().0 == 0 {
            // exit the loop
            loop_stack.pop();
          } else {
            // cell isn't 0 so jump back to corresponding opening loop brace
            // not sure what rust will do if the stack is empty
            pc = loop_stack[loop_stack.len() - 1];
          }
        },
        _ => (),
      };

      pc += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::io::Cursor;

  fn run_program(program: String, input: String) -> String {
    let mut bvm = BVM::new(program.chars().collect());

    let input_bytes: Vec<u8> = input.bytes().collect();
    let mut input_stream = Cursor::new(input_bytes);
    let mut output_stream = Cursor::new(Vec::new());

    bvm.run(&mut input_stream, &mut output_stream);
    
    unsafe {
      String::from_utf8_unchecked(output_stream.into_inner())
    }
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
    let program = String::from("+[-->-[>>+>-----<<]<--<---]>-.>>>+.>>..+++[.>]<<<<.+++.------.<<-.>>>>+.");
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
