mod brainfuck;
use brainfuck::BVM;

mod brainlove;
use brainlove::BrainloveCompiler;

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
  #[arg(short, long, default_value_t = false)]
  compile: bool,
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


  if !args.compile { 
    // run brainfuck
    let mut bvm = BVM::new(program.chars().collect());
    
    if args.input.is_some() {
      bvm.run(&mut Cursor::new(args.input.unwrap()), &mut stdout());
    } else {
      bvm.run(&mut stdin(), &mut stdout());
    }
  } else {
    // run the compiler on the provided file
    let mut mfc = BrainloveCompiler::new();
    mfc.compile(program);
  }
}