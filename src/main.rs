mod brainfuck;
use brainfuck::BVM;

use std::io::{stdin, stdout};


fn main() {
  let prog = "";
  let mut vm = BVM::new(prog.chars().collect());

  vm.run(&mut stdin(), &mut stdout());

}