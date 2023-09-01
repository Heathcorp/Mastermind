mod brainfuck;
use brainfuck::BVM;

use std::{io::{stdin, stdout, Cursor}, fs::File};

use clap::Parser;

#[derive(Parser,Default,Debug)]
#[command(author = "Heathcorp", version = "0.1", about = "Brainfuck interpreter and compiler", long_about = None)]
struct Arguments {
  #[arg(short, long)]
  file: Option<String>,
  #[arg(short, long)]
  program: Option<String>,
  #[arg(short, long)]
  input: Option<String>,
}

fn main() {
  let args = Arguments::parse();
  
  let mut program = if args.file.is_some() {
    std::fs::read_to_string(args.file.unwrap()).unwrap()
  } else if args.program.is_some() {
    args.program.unwrap()
  } else {
    String::new()
  };

  let mut bvm = BVM::new(program.chars().collect());
  
  if args.input.is_some() {
    bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout());
  } else {
    bvm.run(&mut stdin(), &mut stdout());
  }
}