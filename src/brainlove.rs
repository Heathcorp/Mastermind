// need to somehow make a language that compiles to brainfuck

// example:
// def add
//   int[1] a
//   int[1] b consume
// start
//   loop consume b
//   start
//     add a 1
//   end
// end a

// // program start
// int[1] a 4
// int[1] b 3
// #debug
// call add a b
// #debug

// // desired output:
// // ++++>+++<# >[-<+>]<#

// to start with we will only support integer operations
// strings will make things tricky but will be necessary as a second stage
// all functions will be top level, no closures with context of their parent scope
// all variables are the first thing in their scope

// rough idea of minimum barely proof of concept prototype:
// 1. split program by lines
// 2. variables initialized as specific tape positions (malloc)
// 3. to start with we only support simple increment decrement ops
// 4. loop with consume means it will loop over a variable and delete the variable (free)
// 5. at the start of each loop iteration, keep track of the tape pos and return back at the end of the block (this will be different for strings possibly)
// 6. this will have to change for other things as well but that wil come in stage 2

pub struct BrainloveCompiler {

}

impl BrainloveCompiler {
  pub fn new() -> Self {
    BrainloveCompiler {}
  }

  pub fn compile<'a>(&mut self, source: String) {
    // basic steps:
    // 1. tokenise the source code into commands, blocks, variables

    let lines: Vec<&str> = source.lines().collect();

    // step one strip and get the type of each line for ease of processing
    let line_pairs: Vec<(LineType, &str)> = lines
      .into_iter()
      .map(|line| {
        let trimmed = Self::strip_line(line);
        (Self::get_line_type(trimmed), trimmed)
      })
      .filter(|pair| pair.0 != LineType::None)
      .collect();

    let root_block = self.parse_block(&line_pairs);

    println!("{:#?}", root_block);
  }

  // recursive function to create a tree representation of the program
  fn parse_block<'a>(&'a self, line_pairs: &Vec<(LineType, &'a str)>) -> Block {
    let mut parsed_block = Block {
      functions: Vec::new(),
      variables: Vec::new(),
      commands: Vec::new(),
    };

    let mut i = 0;
    while i < line_pairs.len() {
      let words: Vec<&str> = line_pairs[i].1.split_whitespace().collect();
      match line_pairs[i].0 {
        LineType::FunctionDefinition => {
          // skip the whole definition for now (TODO)
          // find the start block
          while line_pairs[i].0 != LineType::BlockStart {
            i += 1;
          };
          let mut depth = 1;
          // skip to the corresponding end block
          while depth > 0 {
            i += 1;
            depth += match line_pairs[i].0 {
              LineType::BlockStart => 1,
              LineType::BlockEnd => -1,
              _ => 0,
            };
          };
        },
        LineType::VariableDeclaration => {
          parsed_block.variables.push(
            Variable {
              name: words[1],
              var_type: VariableType::Int,
              length: 1,
              argument: false,
              consumed: words.contains(&"consume")
            }
          );
        },
        _ => (),
      }
      i += 1;
    }

    parsed_block
  }

  fn strip_line(line: &str) -> &str {
    let mut stripped = line;
    // remove comments
    let split = line.split_once("//");
    if split.is_some() {
      stripped = split.unwrap().0;
    }

    // remove whitespace
    stripped.trim()
  }
  
  fn get_line_type(line: &str) -> LineType {
    let mut split = line.split_whitespace();
    let word = split.next().unwrap_or(line);

    match word {
      "def" => LineType::FunctionDefinition,
      // TODO: change this?
      "int[1]" => LineType::VariableDeclaration,
      "start" => LineType::BlockStart,
      "end" => LineType::BlockEnd,
      "loop" => LineType::LoopStart,
      "add" => LineType::AddOperation,
      "call" => LineType::FunctionCall,
      _ => LineType::None,
    }
  }
}

#[derive(Debug)]
struct Block<'a> {
  functions: Vec<Function>,
  variables: Vec<Variable<'a>>,
  commands: Vec<Command<'a>>,
}

#[derive(Debug)]
struct Function {

}

#[derive(Debug)]
enum VariableType {
  Str,
  Int,
}

#[derive(Debug)]
struct Variable<'a> {
  name: &'a str,
  var_type: VariableType,
  length: usize,
  argument: bool,
  consumed: bool,
}

#[derive(Debug)]
enum CommandType {
  Add,
  Loop,
}

#[derive(Debug)]
struct Command<'a> {
  cmd_type: CommandType,
  source_line: usize,
  attached_block: Block<'a>,
}

#[derive(Debug, PartialEq)]
enum LineType {
  None,
  FunctionDefinition,
  FunctionCall,
  VariableDeclaration,
  LoopStart,
  BlockStart,
  BlockEnd,
  AddOperation,
}
