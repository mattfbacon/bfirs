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

pub use compile::InstructionStream;
pub use instruction::Instruction;
pub use interpret::Interpreter;

/// The minimum size of the data array.
pub const MIN_DATA_ARRAY_SIZE: usize = 30_000;
