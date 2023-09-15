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

use std::{clone, collections::HashMap};

use crate::construction::BrainfuckBuilder;

pub struct MastermindCompiler {}

impl MastermindCompiler {
	pub fn new() -> Self {
		MastermindCompiler {}
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

		let mut builder = BrainfuckBuilder::new();
		self.compile_block(root_block, &mut builder);

		println!("{:#?}", builder);
		let output = builder.to_string();

		println!("{:#?}", output);
	}

	// recursive function to create a tree representation of the program
	fn parse_block<'a>(&'a self, line_pairs: &'a [LinePair]) -> Block {
		let mut parsed_block = Block {
			variables: Vec::new(),
			commands: Vec::new(),
		};
		// functions are inlined as there are no functions in brainfuck and it is easier to inline them at the syntax tree step than at the next step
		let mut functions: Vec<Function> = Vec::new();

		let mut i = 0;
		while i < line_pairs.len() {
			let line_words: Vec<&str> = line_pairs[i].1.split_whitespace().collect();
			match line_pairs[i].0 {
				LineType::FunctionDefinition => {
					i += 1;
					// start at the arguments to include them in the block
					let args_start_line = i;
					// find the start block
					while line_pairs[i].0 != LineType::BlockStart {
						i += 1;
					}
					let block_start_line = i;
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

					// recursively parse the function's block
					let range = &line_pairs[args_start_line..=end_line];
					let mut function_block = self.parse_block(range);
					for i in 0..(block_start_line - args_start_line) {
						function_block.variables[i].argument = true;
					}
					functions.push(Function {
						name: line_words[1],
						block: function_block,
					})
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
					let var_name = line_words[1];
					parsed_block
						.commands
						.push(Command::ConsumeLoop { var_name, block });
				}
				LineType::VariableDeclaration => {
					let var_name = line_words[1];
					parsed_block.variables.push(Variable {
						name: var_name,
						val_type: ValueType::Int,
						length: 1,
						argument: false,
					});
					if line_words.len() > 2 {
						// initialisation immediate value
						let imm: i32 = line_words[2].parse().unwrap();
						parsed_block
							.commands
							.push(Command::AddImmediate { var_name, imm });
					}
				}
				LineType::AddOperation => {
					// surely I can use references to point to the actual variable object
					let var_name: &str = line_words[1];
					let imm: i32 = line_words[2].parse().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::SubOperation => {
					// surely I can use references to point to the actual variable object
					let var_name: &str = line_words[1];
					let imm: i32 = -line_words[2].parse::<i32>().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::FunctionCall => {
					// tricky part, expanding function calls

					// get the function being called
					let function_name = line_words[1];
					let mut function_instance: Function = functions
						.iter()
						.find(|f| f.name == function_name)
						.unwrap()
						.clone();
					let function_arg_names: Vec<&str> = function_instance
						.block
						.variables
						.iter()
						.filter_map(|var| match var.argument {
							true => Some(var.name),
							false => None,
						})
						.collect();

					// get the variables being called as arguments
					let mut var_translations: HashMap<&str, &str> = HashMap::new();
					for (i, outer_var_name) in line_words[2..].iter().enumerate() {
						let function_var = function_arg_names[i];
						var_translations.insert(function_var, *outer_var_name);
					}

					// stick the commands and variables into the block, final step

					parsed_block.commands.push(Command::ScopedBlock {
						var_translations,
						block: function_instance.block,
					});
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

	// I don't think this construction is a good one, not very functional
	// kind of dependency injection but not really, it's not too too bad but I'm sure there is a better way
	fn compile_block(&self, block: Block, builder: &mut BrainfuckBuilder) {
		// the real meat and potatoes

		// start a new variable name scope

		for var in block.variables.iter() {
			if !var.argument {
				builder.allocate_var(var.name);
			}
		}

		for cmd in block.commands {
			match cmd {
				Command::AddImmediate { var_name, imm } => {
					builder.move_to_var(var_name);
					builder.add_to_current_cell(imm);
				}
				Command::ConsumeLoop {
					var_name,
					block: loop_block,
				} => {
					// to start the loop move to the variable you want to consume
					builder.move_to_var(var_name);
					builder.open_loop();
					// do what you want to do in the loop
					self.compile_block(loop_block, builder);
					// decrement the variable
					builder.move_to_var(var_name);
					builder.add_to_current_cell(-1);

					builder.close_loop();
				}
				Command::ScopedBlock {
					var_translations,
					block,
				} => {
					// tricky stuff, this is only used for functions atm
					// basically we need to recursively compile the contained block,
					// I think this will have to be done in the builder

					// prime the builder with the variable translations
					builder.open_scope(&var_translations);

					self.compile_block(block, builder);

					// remove the variable translations from the builder
					builder.close_scope();
				}
				_ => (),
			}
		}

		for var in block.variables.iter() {
			builder.free_var(var.name);
		}
	}
}

// TODO: use enums

#[derive(Debug, Clone)]
struct Block<'a> {
	variables: Vec<Variable<'a>>,
	commands: Vec<Command<'a>>,
}

#[derive(Debug, Clone)]
struct Function<'a> {
	name: &'a str,
	block: Block<'a>,
}

#[derive(Debug, Clone)]
enum ValueType {
	Str,
	Int,
}

#[derive(Debug, Clone)]
struct Variable<'a> {
	name: &'a str,
	val_type: ValueType,
	length: usize,
	argument: bool,
}

#[derive(Debug, Clone)]
enum Command<'a> {
	AddImmediate {
		var_name: &'a str,
		imm: i32,
	},
	ConsumeLoop {
		var_name: &'a str,
		block: Block<'a>,
	},
	ScopedBlock {
		var_translations: HashMap<&'a str, &'a str>,
		block: Block<'a>,
	},
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
