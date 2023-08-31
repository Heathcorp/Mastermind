mod brainfuck;
use brainfuck::BVM;

use std::io::{stdin, stdout};

fn main() {
  let prog = "++++>+++<# >[-<+>]<#";
  let mut bvm = BVM::new(prog.chars().collect());

  bvm.run(&mut stdin(), &mut stdout());

}