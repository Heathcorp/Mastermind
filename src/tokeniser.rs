pub struct MastermindTokeniser;

impl MastermindTokeniser {
	pub fn tokenise<'a>(&'a self, source: &'a str) -> Vec<LinePair> {
		// TODO: make this an actual tokeniser and not filtering and trimming lines like this
		let lines: Vec<&str> = source.lines().collect();

		// step one strip and get the type of each line for ease of processing
		let line_pairs: Vec<LinePair> = lines
			.into_iter()
			.map(|line| {
				let trimmed = Self::strip_line(line);
				(Self::get_line_type(trimmed), trimmed)
			})
			.filter(|pair| pair.0 != LineType::None)
			.collect();

		line_pairs
	}

	fn get_line_type(line: &str) -> LineType {
		let mut split = line.split_whitespace();
		let word = split.next().unwrap_or(line);

		match word {
			"def" => LineType::FunctionDefinition,
			// TODO: change this?
			"int[1]" => LineType::IntegerDeclaration,
			"free" => LineType::VariableFreedom,
			"bool" => LineType::BooleanDeclaration,
			"start" | "{" => LineType::BlockStart,
			"end" | "}" => LineType::BlockEnd,
			"loop" => LineType::ConsumeLoopDefinition,
			"while" => LineType::WhileLoopDefinition,
			"if" => LineType::IfDefinition,
			"else" => LineType::ElseDefinition,
			"add" => LineType::AddOperation,
			"sub" => LineType::SubOperation,
			"copy" => LineType::CopyOperation,
			"drain" => LineType::DrainOperation,
			"clear" => LineType::ClearOperation,
			"push" => LineType::PushOperation,
			"pop" => LineType::PopOperation,
			"deal" => LineType::StackLoopDefinition,
			"call" => LineType::FunctionCall,
			"output" => LineType::OutputOperation,
			"#debug" => LineType::Debug,
			"#goto" => LineType::Debug,
			"#print" => LineType::Debug,
			_ => LineType::None,
		}
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineType {
	None,
	FunctionDefinition,
	FunctionCall,
	IntegerDeclaration,
	BooleanDeclaration,
	VariableFreedom,
	ConsumeLoopDefinition,
	WhileLoopDefinition,
	IfDefinition,
	ElseDefinition,
	BlockStart,
	BlockEnd,
	AddOperation,
	SubOperation,
	CopyOperation,
	DrainOperation,
	ClearOperation,
	PushOperation,
	PopOperation,
	StackLoopDefinition,
	OutputOperation,
	Debug,
}

pub type LinePair<'a> = (LineType, &'a str);
