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

	pub fn compile<'a>(&mut self, source: String) -> String {
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

		let functions = Vec::new();
		let root_block = self.parse_block(&line_pairs, &functions);

		// println!("{:#?}", root_block);

		let mut builder = BrainfuckBuilder::new();
		Self::compile_block(root_block, &mut builder);

		let output = builder.to_string();

		output
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
				LineType::VariableDeclaration => {
					let var_name = String::from(line_words[1]);
					parsed_block.variables.push(Variable {
						name: var_name.clone(),
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
					let var_name = String::from(line_words[1]);
					let imm: i32 = line_words[2].parse().unwrap();

					parsed_block
						.commands
						.push(Command::AddImmediate { var_name, imm });
				}
				LineType::SubOperation => {
					let var_name = String::from(line_words[1]);
					let imm: i32 = -line_words[2].parse::<i32>().unwrap();

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
			"start" | "{" => LineType::BlockStart,
			"end" | "}" => LineType::BlockEnd,
			"loop" => LineType::ConsumeLoopDefinition,
			"while" => LineType::WhileLoopDefinition,
			"if" => LineType::IfDefinition,
			"else" => LineType::ElseDefinition,
			"add" => LineType::AddOperation,
			"sub" => LineType::SubOperation,
			"copy" => LineType::CopyOperation,
			"clear" => LineType::ClearOperation,
			"call" => LineType::FunctionCall,
			"output" => LineType::OutputOperation,
			"#debug" => LineType::Debug,
			"#goto" => LineType::Debug,
			"#print" => LineType::Debug,
			_ => LineType::None,
		}
	}

	// I don't think this construction is a good one, not very functional
	// kind of dependency injection but not really, it's not too too bad but I'm sure there is a better way
	fn compile_block(block: Block, builder: &mut BrainfuckBuilder) {
		builder.open_scope(None);

		let start_len = builder.program.len();
		// the real meat and potatoes

		// start a new variable name scope

		for var in block.variables.iter() {
			if !var.argument {
				builder.allocate_var(var.name.as_str());
			}
		}

		for cmd in block.commands.clone() {
			match cmd {
				Command::AddImmediate { var_name, imm } => {
					builder.move_to_var(&var_name);
					builder.add_to_current_cell(imm);
				}
				Command::CopyVariable {
					target_name,
					source_name,
				} => {
					// because bf is fucked, the quickest way to copy consumes the original variable
					// so we have to copy it twice, then copy one of them back to the original variable

					// if the variable is in its top level scope then it's okay to move the variable
					let temp_cell = builder.allocate_cell();

					builder.move_to_var(&source_name);
					builder.open_loop();
					// decrement the source variable
					builder.add_to_current_cell(-1);
					// increment the target variables
					builder.move_to_var(&target_name);
					builder.add_to_current_cell(1);
					builder.move_to_pos(temp_cell);
					builder.add_to_current_cell(1);

					builder.move_to_var(&source_name);
					builder.close_loop();

					match builder.check_var_scope(&source_name) {
						true => {
							// variable is defined in this scope so copying and moving is not a problem
							// simply move the source variable to point to the temporary cell
							let old_var_cell = builder.change_var_cell(&source_name, temp_cell);
							// free up the old variable's spot for something else
							// this won't be very helpful currently as variables are allocated before commands, TODO: change this
							builder.free_cell(old_var_cell);
						}
						false => {
							// variable is not within this scope so we need to satisfy the loop balance
							// copy back the temp cell
							builder.move_to_pos(temp_cell);
							builder.open_loop();
							builder.add_to_current_cell(-1);

							builder.move_to_var(&source_name);
							builder.add_to_current_cell(1);

							builder.move_to_pos(temp_cell);
							builder.close_loop();

							// free the temp memory
							builder.free_cell(temp_cell);
						}
					}
				}
				Command::ClearVariable { var_name } => {
					builder.move_to_var(&var_name);
					builder.open_loop();
					builder.add_to_current_cell(-1);
					builder.close_loop();
				}
				Command::ConsumeLoop {
					var_name,
					block: loop_block,
				} => {
					// to start the loop move to the variable you want to consume
					builder.move_to_var(&var_name);
					builder.open_loop();
					// do what you want to do in the loop
					Self::compile_block(loop_block, builder);
					// decrement the variable
					builder.move_to_var(&var_name);
					builder.add_to_current_cell(-1);

					builder.close_loop();
				}
				Command::WhileLoop {
					var_name,
					block: loop_block,
				} => {
					// to start the loop move to the variable you want to consume
					builder.move_to_var(&var_name);
					builder.open_loop();

					// do what you want to do in the loop
					Self::compile_block(loop_block, builder);

					builder.move_to_var(&var_name);
					builder.close_loop();
				}
				Command::IfElse {
					var_name,
					consume,
					if_block,
					else_block,
				} => {
					let temp_move_cell = match consume {
						false => Some(builder.allocate_cell()),
						true => None,
					};
					let else_condition_cell = match else_block {
						Some(_) => {
							let cell = builder.allocate_cell();
							builder.move_to_pos(cell);
							builder.add_to_current_cell(1);
							Some(cell)
						}
						None => None,
					};

					// if block
					builder.move_to_var(&var_name);
					builder.open_loop();

					match temp_move_cell {
						Some(cell) => {
							// move if condition to temp cell
							builder.open_loop();
							builder.add_to_current_cell(-1);
							builder.move_to_pos(cell);
							builder.add_to_current_cell(1);
							builder.move_to_var(&var_name);
							builder.close_loop();
						}
						None => {
							// consume the if variable instead
							// TODO: check if it is a boolean and just decrement instead
							builder.open_loop();
							builder.add_to_current_cell(-1);
							builder.close_loop();
						}
					}

					// reassign the variable to the temporary cell so that if statement inner can use the variable
					let old_var_cell = match temp_move_cell {
						Some(cell) => Some(builder.change_var_cell(&var_name, cell)),
						None => None,
					};

					match else_condition_cell {
						Some(cell) => {
							// remove the else condition so that it does not run
							builder.move_to_pos(cell);
							builder.add_to_current_cell(-1);
						}
						None => (),
					};

					Self::compile_block(if_block, builder);

					match old_var_cell {
						Some(cell) => {
							// move to the original variable position to close the loop
							builder.move_to_pos(cell);
							builder.close_loop();

							// now copy back the variable from the temp cell
							builder.move_to_var(&var_name);
							builder.open_loop();
							builder.add_to_current_cell(-1);
							builder.move_to_pos(cell);
							builder.add_to_current_cell(1);
							builder.move_to_var(&var_name);
							builder.close_loop();

							// reassign the variable again (undo from before)
							builder.change_var_cell(&var_name, cell);
							builder.free_cell(temp_move_cell.unwrap());
						}
						None => {
							// just move to the variable
							builder.move_to_var(&var_name);
							builder.close_loop();
						}
					}

					match else_condition_cell {
						Some(cell) => {
							// else block
							builder.move_to_pos(cell);
							builder.open_loop();
							// clear else condition variable
							builder.add_to_current_cell(-1);

							Self::compile_block(else_block.unwrap(), builder);

							builder.move_to_pos(cell);
							builder.close_loop();
							builder.free_cell(cell);
						}
						None => (),
					};
				}
				Command::ScopedBlock {
					var_translations,
					block,
				} => {
					// tricky stuff, this is only used for functions atm
					// basically we need to recursively compile the contained block,
					// I think this will have to be done in the builder

					// prime the builder with the variable translations
					builder.open_scope(Some(&var_translations));

					Self::compile_block(block, builder);

					// remove the variable translations from the builder
					builder.close_scope();
				}
				Command::DebugTape => {
					builder.add_symbol('#');
				}
				Command::DebugGoto(var_name) => {
					builder.move_to_var(&var_name);
				}
				Command::DebugPrintInt(var_name) => {
					builder.move_to_var(&var_name);
					builder.add_symbol('@');
				}
				Command::OutputByte { var_name } => {
					builder.move_to_var(&var_name);
					builder.add_symbol('.');
				}
			}
		}

		for var in block.variables.iter() {
			if !var.argument {
				builder.free_var(var.name.as_str());
			}
		}

		builder.close_scope();
		///////
		// let s: String = builder.program[start_len..builder.program.len()]
		// 	.iter()
		// 	.collect();
		// println!("{block:#?} ::: {s}");
	}
}

// TODO: use enums

#[derive(Debug, Clone)]
struct Block {
	variables: Vec<Variable>,
	commands: Vec<Command>,
}

#[derive(Debug, Clone)]
struct Function {
	name: String,
	block: Block,
}

#[derive(Debug, Clone)]
enum ValueType {
	Str,
	Int,
}

#[derive(Debug, Clone)]
struct Variable {
	name: String,
	val_type: ValueType,
	length: usize,
	argument: bool,
}

#[derive(Debug, Clone)]
enum Command {
	AddImmediate {
		var_name: String,
		imm: i32,
	},
	CopyVariable {
		target_name: String,
		source_name: String,
	},
	ClearVariable {
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

#[derive(Debug, PartialEq, Clone)]
enum LineType {
	None,
	FunctionDefinition,
	FunctionCall,
	VariableDeclaration,
	ConsumeLoopDefinition,
	WhileLoopDefinition,
	IfDefinition,
	ElseDefinition,
	BlockStart,
	BlockEnd,
	AddOperation,
	SubOperation,
	CopyOperation,
	ClearOperation,
	OutputOperation,
	Debug,
}

type LinePair<'a> = (LineType, &'a str);
