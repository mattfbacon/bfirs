#![doc = include_str!("../../README.md")]
#![deny(
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	missing_copy_implementations,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms,
	unused_qualifications
)]
#![warn(
	clippy::pedantic,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs
)]

pub mod cell_type;
pub mod compile;
pub mod instruction;
pub mod interpret;

pub use cell_type::CellType;
pub use compile::InstructionStream;
pub use instruction::Instruction;
pub use interpret::Interpreter;

/// Convenience method to compile and optimize the code in `input`.
///
/// # Errors
///
/// Returns `Err` iff compilation or optimization returns `Err`.
pub fn compile<T: CellType>(input: &str) -> Result<InstructionStream<T>, compile::Error> {
	InstructionStream::optimized_from_code(input.bytes())
}

/// Convenience method to interpret the given instruction stream.
///
/// # Errors
///
/// Returns `Err` iff the interpreter returns `Err`.
pub fn interpret<T: CellType>(stream: &InstructionStream<T>) -> Result<(), interpret::Error> {
	Interpreter::build_stdio()
		.configure_for(stream)
		.build()
		.run(stream.instructions())
}

/// Convenience method to compile, optimize, and interpret the code in `input`.
///
/// Uses 8-bit mode.
///
/// # Panics
///
/// Panics if compiling, optimizing, or interpreting fails.
pub fn run(input: &str) {
	let stream = compile::<u8>(input).expect("compilation failed");
	interpret(&stream).expect("interpreting failed");
}

/// The minimum size of the data array.
pub const MIN_DATA_ARRAY_SIZE: usize = 30_000;
