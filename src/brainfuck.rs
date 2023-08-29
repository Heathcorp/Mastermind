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
  fn set_head_position(&mut self, position: i32) {
    self.head_position = position;
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

pub struct BrainfuckVirtualMachine {
  tape: Tape,
  program: Vec<char>,
}

impl BrainfuckVirtualMachine {
  pub fn new(program: Vec<char>) -> Self {
    BrainfuckVirtualMachine {
      tape: Tape::new(),
      program,
    }
  }
  pub fn run(&mut self, input: &mut impl Read, output: &mut impl Write) {
    let mut pc = 0;
    // let loops = Vec::new();

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
        _ => break
      };

      pc += 1;
    }
  }
}
