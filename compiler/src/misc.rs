#[derive(serde::Deserialize)]
pub struct MastermindConfig {
	// basic pure brainfuck optimisations
	pub optimise_generated_code: bool,
	// track cell value and clear with constant addition if possible
	pub optimise_cell_clearing: bool,
	// track cell value and skip loops which can never be entered
	pub optimise_unreachable_loops: bool,
	// TODO: prune variables that aren't needed? Maybe combine with empty blocks stuff
	pub optimise_variable_usage: bool,
	// TODO: optimise memory layout to minimise tape head movement
	// recommended to turn on these next two together
	pub optimise_memory_allocation: bool,
	// golf constants, useful for single characters or large numbers
	// probably not great with strings yet, may need another optimisation for that
	pub optimise_constants: bool,
	// TODO: recursively prune if statements/loops if they do nothing
	pub optimise_empty_blocks: bool,
	// Memory Allocation Method
	//'1D Mastermind'  0
	// '2D Mastermind - Spiral'  1
	// '2D Mastermind - Tiles'  2
	// '2D Mastermind - Nearest' 3
	pub memory_allocation_method: u8,
}

impl MastermindConfig {
	pub fn new(optimise_bitmask: usize) -> MastermindConfig {
		MastermindConfig {
			optimise_generated_code: (optimise_bitmask & 0b00000001) > 0,
			optimise_cell_clearing: (optimise_bitmask & 0b00000010) > 0,
			optimise_unreachable_loops: (optimise_bitmask & 0b00000100) > 0,
			optimise_variable_usage: false,
			optimise_memory_allocation: false,
			optimise_constants: false,
			optimise_empty_blocks: false,
			memory_allocation_method: 0,
		}
	}
}
