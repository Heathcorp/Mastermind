use std::collections::HashMap;

use crate::tokeniser::{LinePair, LineType};

pub struct MastermindParser;

impl MastermindParser {
	pub fn parse<'a>(&mut self, line_pairs: Vec<LinePair>) -> Block {
		// basic steps:
		// 1. tokenise the source code into commands, blocks, variables

		let functions = Vec::new();
		self.parse_block(&line_pairs, &functions)
	}

	// recursive function to create a tree representation of the program
	fn parse_block<'a>(
		&'a self,
		line_pairs: &'a [LinePair],
		outer_functions: &'a Vec<Function>,
	) -> Block {
		// functions are inlined as there are no functions in brainfuck and it is easier to inline them at the syntax tree step than at the next step
		// might have to make them global??? hmm

		let mut inner_functions: Vec<Function> = Vec::new();

		let mut parsed_block = Block {
			variables: Vec::new(),
			commands: Vec::new(),
		};

		// parse only functions first
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
					let mut function_block = self.parse_block(range, outer_functions);
					for i in 0..(block_start_line - args_start_line) {
						function_block.variables[i].argument = true;
					}
					inner_functions.push(Function {
						name: String::from(line_words[1]),
						block: function_block,
					});
				}
				_ => (),
			}
			i += 1;
		}

		// create a new collection of all available functions
		let mut combined_functions = Vec::new();
		combined_functions.extend(inner_functions);
		combined_functions.extend(outer_functions.iter().map(|of| of.clone()));

		// parse all the other stuff
		// should probably scope this i var instead of using it twice
		i = 0;
		while i < line_pairs.len() {
			let line_words: Vec<&str> = line_pairs[i].1.split_whitespace().collect();
			let line_type = line_pairs[i].0.clone();
			match line_type {
				LineType::FunctionDefinition => {
					// find the function block start
					while line_pairs[i].0 != LineType::BlockStart {
						i += 1;
					}
					// skip to the corresponding block end so that we don't look at functions
					let mut depth = 1;
					while depth > 0 {
						i += 1;
						depth += match line_pairs[i].0 {
							LineType::BlockStart => 1,
							LineType::BlockEnd => -1,
							_ => 0,
						};
					}
				}
				LineType::ConsumeLoopDefinition | LineType::WhileLoopDefinition => {
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
					// see above, duplicate code, could be improved
					let block = self.parse_block(range, &combined_functions);
					let var_name = String::from(line_words[1]);

					match line_type {
						LineType::ConsumeLoopDefinition => {
							parsed_block
								.commands
								.push(Command::ConsumeLoop { var_name, block });
						}
						LineType::WhileLoopDefinition => {
							parsed_block
								.commands
								.push(Command::WhileLoop { var_name, block });
						}
						_ => (),
					};
				}
				LineType::IfDefinition => {
					let var_name = String::from(line_words[1]);
					// TODO: simplify some of these match branches
					// find the start line
					while line_pairs[i].0 != LineType::BlockStart {
						i += 1;
					}
					let if_start_line = i;
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
					let if_end_line = i;
					// same for else block
					// I don't like how I've done this, TODO: redo with a better program design
					i += 1;
					let mut is_else = false;
					if line_pairs.len() > i && line_pairs[i].0 == LineType::ElseDefinition {
						is_else = true;
					} else {
						i -= 1;
					}
					let mut else_start_line = i;
					if is_else {
						while line_pairs[i].0 != LineType::BlockStart {
							i += 1;
						}
						else_start_line = i;
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
					let else_end_line = i;

					let if_block = self.parse_block(
						&line_pairs[if_start_line..=if_end_line],
						&combined_functions,
					);
					let else_block = match is_else {
						true => Some(self.parse_block(
							&line_pairs[else_start_line..=else_end_line],
							&combined_functions,
						)),
						false => None,
					};

					parsed_block.commands.push(Command::IfElse {
						var_name,
						consume: line_words.len() > 2 && line_words[2] == "consume",
						if_block,
						else_block,
					});
				}
				LineType::IntegerDeclaration => {
					let var_name = String::from(line_words[1]);
					let mut imm: i8 = 0;
					if line_words.len() > 2 {
						// initialise immediate value
						imm = line_words[2].parse().unwrap();
						parsed_block.commands.push(Command::AddImmediate {
							var_name: var_name.clone(),
							imm,
						});
					}
					parsed_block.variables.push(Variable {
						name: var_name,
						argument: false,
						var_type: VariableType::ByteInteger,
						initial: imm,
					});
				}
				LineType::BooleanDeclaration => {
					let var_name = String::from(line_words[1]);
					let mut imm: i8 = 0;
					if line_words.len() > 2 {
						// initialise immediate value
						imm = line_words[2].parse().unwrap();
						parsed_block.commands.push(Command::AddImmediate {
							var_name: var_name.clone(),
							imm,
						});
					}
					parsed_block.variables.push(Variable {
						name: var_name,
						var_type: VariableType::Boolean,
						argument: false,
						initial: imm,
					});
				}
				LineType::AddOperation => {
					// surely I can use references to point to the actual variable object
					let var_name = String::from(line_words[1]);
					let imm: i8 = line_words[2].parse().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::SubOperation => {
					let var_name = String::from(line_words[1]);
					let imm: i8 = -line_words[2].parse::<i8>().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::CopyOperation => {
					// copies are handled by the compiler I guess?
					parsed_block.commands.push(Command::CopyVariable {
						target_name: String::from(line_words[1]),
						source_name: String::from(line_words[2]),
					});
				}
				LineType::ClearOperation => {
					parsed_block.commands.push(Command::ClearVariable {
						var_name: String::from(line_words[1]),
					});
				}
				LineType::FunctionCall => {
					// tricky part, expanding function calls

					// get the function being called
					let function_name = line_words[1];
					let function_instance: Function = combined_functions
						.iter()
						.find(|f| f.name == function_name)
						.unwrap()
						.clone();
					let function_arg_names: Vec<String> = function_instance
						.block
						.variables
						.iter()
						.filter_map(|var| match var.argument {
							true => Some(var.name.clone()),
							false => None,
						})
						.collect();

					// get the variables being called as arguments
					let mut var_translations = HashMap::new();
					for (i, outer_var_name) in line_words[2..].iter().enumerate() {
						var_translations
							.insert(function_arg_names[i].clone(), String::from(*outer_var_name));
					}

					// stick the commands and variables into the block, final step

					parsed_block.commands.push(Command::ScopedBlock {
						var_translations,
						block: function_instance.block.clone(),
					});
				}
				LineType::OutputOperation => {
					parsed_block.commands.push(Command::OutputByte {
						var_name: String::from(line_words[1]),
					});
				}
				LineType::PushOperation => {
					parsed_block.commands.push(Command::PushStack {
						var_name: String::from(line_words[1]),
					});
				}
				LineType::PopOperation => {
					parsed_block.commands.push(Command::PopStack {
						var_name: String::from(line_words[1]),
					});
				}
				LineType::Debug => match line_words[0] {
					"#debug" => {
						parsed_block.commands.push(Command::DebugTape);
					}
					"#goto" => {
						parsed_block
							.commands
							.push(Command::DebugGoto(String::from(line_words[1])));
					}
					"#print" => {
						parsed_block
							.commands
							.push(Command::DebugPrintInt(String::from(line_words[1])));
					}
					_ => (),
				},
				_ => (),
			}
			i += 1;
		}

		parsed_block
	}
}

// TODO: use enums

#[derive(Debug, Clone)]
pub struct Block {
	pub variables: Vec<Variable>,
	pub commands: Vec<Command>,
}

#[derive(Debug, Clone)]
pub struct Function {
	pub name: String,
	pub block: Block,
}

#[derive(Debug, Clone)]
pub enum VariableType {
	ByteInteger,
	Boolean,
}

#[derive(Debug, Clone)]
pub struct Variable {
	pub name: String,
	pub argument: bool,
	pub var_type: VariableType,
	pub initial: i8,
}

#[derive(Debug, Clone)]
pub enum Command {
	AddImmediate {
		var_name: String,
		imm: i8,
	},
	CopyVariable {
		target_name: String,
		source_name: String,
	},
	ClearVariable {
		var_name: String,
	},
	PushStack {
		var_name: String,
	},
	PopStack {
		var_name: String,
	},
	ConsumeLoop {
		var_name: String,
		block: Block,
	},
	WhileLoop {
		var_name: String,
		block: Block,
	},
	ScopedBlock {
		var_translations: HashMap<String, String>,
		block: Block,
	},
	OutputByte {
		var_name: String,
	},
	DebugTape,
	DebugGoto(String),
	DebugPrintInt(String),
	IfElse {
		var_name: String,
		consume: bool, // TODO, really this should be determined by the compiler instead of the programmer
		if_block: Block,
		else_block: Option<Block>,
	},
}
