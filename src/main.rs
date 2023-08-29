mod brainfuck;
use brainfuck::BrainfuckVirtualMachine;

use std::io::{stdin, stdout};


fn main() {
  let prog = ",.+.+.>----++++,.+.<.<,.+.+.>.";
  let mut vm = BrainfuckVirtualMachine::new(prog.chars().collect());

  vm.run(&mut stdin(), &mut stdout());

}