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

use std::collections::HashMap;

pub struct BrainloveCompiler {}

impl BrainloveCompiler {
	pub fn new() -> Self {
		BrainloveCompiler {}
	}

	pub fn compile<'a>(&mut self, source: String) {
		// basic steps:
		// 1. tokenise the source code into commands, blocks, variables

		let lines: Vec<&str> = source.lines().collect();

		// step one strip and get the type of each line for ease of processing
		// I think this could be combined into the parse block function, seems a bit redundant at the moment
		// TODO: simplify
		let line_pairs: Vec<LinePair> = lines
			.into_iter()
			.map(|line| {
				let trimmed = Self::strip_line(line);
				(Self::get_line_type(trimmed), trimmed)
			})
			.filter(|pair| pair.0 != LineType::None)
			.collect();

		println!("{:#?}", line_pairs);

		let root_block = self.parse_block(&line_pairs);

		println!("{:#?}", root_block);

		let output: String = self.transpile_block(root_block).into_iter().collect();

		println!("{:#?}", output);
	}

	// recursive function to create a tree representation of the program
	fn parse_block<'a>(&'a self, line_pairs: &'a [LinePair]) -> Block {
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
					}
					let mut depth = 1;
					// skip to the corresponding end block
					while depth > 0 {
						i += 1;
						depth += match line_pairs[i].0 {
							LineType::BlockStart => 1,
							LineType::BlockEnd => -1,
							_ => 0,
						};
					}
				}
				LineType::LoopDefinition => {
					// find the start block
					while line_pairs[i].0 != LineType::BlockStart {
						i += 1;
					}
					let start_line = i;
					let mut depth = 1;
					// skip to the corresponding end block
					while depth > 0 {
						i += 1;
						depth += match line_pairs[i].0 {
							LineType::BlockStart => 1,
							LineType::BlockEnd => -1,
							_ => 0,
						};
					}
					let end_line = i;

					// recursively parse the loop's block
					let range = &line_pairs[start_line..=end_line];
					let block = self.parse_block(range);
					let var_name = words[1];
					parsed_block
						.commands
						.push(Command::Loop { var_name, block });
				}
				LineType::VariableDeclaration => {
					let var_name = words[1];
					parsed_block.variables.push(Variable {
						name: var_name,
						var_type: VariableType::Int,
						length: 1,
						argument: false,
						consumed: words.contains(&"consume"),
					});
					if words.len() > 2 {
						// initialisation immediate value
						let imm: i32 = words[2].parse().unwrap();
						parsed_block
							.commands
							.push(Command::AddImmediate { var_name, imm });
					}
				}
				LineType::AddOperation => {
					// surely I can use references to point to the actual variable object
					let var_name: &str = words[1];
					let imm: i32 = words[2].parse().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::SubOperation => {
					// surely I can use references to point to the actual variable object
					let var_name: &str = words[1];
					let imm: i32 = -words[2].parse::<i32>().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
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
			"loop" => LineType::LoopDefinition,
			"add" => LineType::AddOperation,
			"sub" => LineType::SubOperation,
			"call" => LineType::FunctionCall,
			_ => LineType::None,
		}
	}

	fn transpile_block<'a>(&self, block: Block) -> Vec<char> {
		// the real meat and potatoes
		let mut output: Vec<char> = Vec::new();

		// start with the variables, probably the special gravy of this whole thing
		// will be structuring multi-cell variables so that they can be operated on
		let mut var_map: HashMap<&str, usize> = HashMap::new();

		{
			for (i, var) in block.variables.iter().enumerate() {
				var_map.insert(var.name, i);
				// this will need to change for multi-cell variables
			}
		}

		// go through commands, part 2 of the special sauce
		{
			let move_to_pos =
				|program: &mut Vec<char>, current_tape_pos: &mut usize, target_tape_pos| {
					let direction: i32 = match target_tape_pos > *current_tape_pos {
						true => 1,
						false => -1,
					};
					let arrow = match target_tape_pos > *current_tape_pos {
						true => '>',
						false => '<',
					};
					for i in (0..current_tape_pos.abs_diff(target_tape_pos)) {
						program.push(arrow);
						match direction > 0 {
							true => {
								*current_tape_pos += 1;
							}
							false => {
								*current_tape_pos -= 1;
							}
						}
					}
				};
			let move_to_var =
				|program: &mut Vec<char>, current_tape_pos: &mut usize, var_name: &str| {
					move_to_pos(program, current_tape_pos, *var_map.get(var_name).unwrap())
				};
			let add_imm_to_cell = |program: &mut Vec<char>, imm: i32| {
				let operator = match imm < 0 {
					true => '-',
					false => '+',
				};
				for i in 0..imm.abs() {
					program.push(operator);
				}
			};

			let mut tape_pos: usize = 0;

			// let move_to_var = |var_name| move_to_pos(var_map.get(var_name).unwrap().clone());
			for cmd in block.commands.iter() {
				match *cmd {
					Command::AddImmediate { var_name, imm } => {
						// move to the variable's tape pos
						move_to_var(&mut output, &mut tape_pos, var_name);

						// add the immediate number
						add_imm_to_cell(&mut output, imm);
					}
					// Command::Loop(block) => {}
					_ => (),
				}
			}
		}

		output
	}
}

// TODO: use enums

#[derive(Debug)]
struct Block<'a> {
	functions: Vec<Function>,
	variables: Vec<Variable<'a>>,
	commands: Vec<Command<'a>>,
}

#[derive(Debug)]
struct Function {}

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
enum Command<'a> {
	AddImmediate { var_name: &'a str, imm: i32 },
	Loop { var_name: &'a str, block: Block<'a> },
}

#[derive(Debug, PartialEq)]
enum LineType {
	None,
	FunctionDefinition,
	FunctionCall,
	VariableDeclaration,
	LoopDefinition,
	BlockStart,
	BlockEnd,
	AddOperation,
	SubOperation,
}

type LinePair<'a> = (LineType, &'a str);
