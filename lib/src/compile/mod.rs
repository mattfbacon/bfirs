//! Compile and optimize Brainfuck input.

use crate::cell_type::CellType;
use crate::instruction::Instruction;

mod optimize;
mod render_c;

/// Errors that can occur while compiling.
#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum Error {
	/// A loop is started but not ended.
	#[error("unmatched loop start")]
	UnmatchedStart,
	/// A loop is ended without being started.
	#[error("unmatched loop end")]
	UnmatchedEnd,
}

/// A stream of instructions.
///
/// Provides helpful methods for manipulating the instruction stream.
#[derive(Debug)]
pub struct InstructionStream<T: CellType> {
	instructions: Vec<Instruction<T>>,
	recommended_array_size: usize,
}

impl<T: CellType> InstructionStream<T> {
	const MIN_ARRAY_SIZE: usize = 30_000;

	/// Creates a new instruction stream from instructions.
	///
	/// # Errors
	///
	/// Returns `Err` iff there are unmatched loop starts or ends.
	#[allow(clippy::missing_panics_doc)] // panic is exceptional
	pub fn new(instructions: Vec<Instruction<T>>) -> Result<Self, Error> {
		let mut stream = Self {
			instructions,
			recommended_array_size: Self::MIN_ARRAY_SIZE,
		};
		stream.update_jump_points()?;
		Ok(stream)
	}

	/// Create a new instruction stream from Brainfuck code.
	///
	/// # Errors
	///
	/// Returns `Err` iff there are unmatched loop starts or ends.
	#[allow(clippy::missing_panics_doc)] // panic is exceptional
	pub fn from_code(input: impl Iterator<Item = u8>) -> Result<Self, Error> {
		let instructions = Self::instructions_from_text(input);
		Self::new(instructions)
	}

	/// Create a new instruction stream from Brainfuck code and optimize it.
	///
	/// This is faster than using `from_code` and then calling `optimize`.
	///
	/// # Errors
	///
	/// Returns `Err` iff there are unmatched loop starts or ends.
	pub fn optimized_from_code(input: impl Iterator<Item = u8>) -> Result<Self, Error> {
		let mut stream = Self {
			instructions: Self::instructions_from_text(input),
			recommended_array_size: Self::MIN_ARRAY_SIZE,
		};

		stream.optimize()?;

		Ok(stream)
	}

	/// Returns a statically-guessed array size that would work best for this brainfuck stream.
	#[must_use]
	pub fn recommended_array_size(&self) -> usize {
		self.recommended_array_size
	}
}

impl<T: CellType> InstructionStream<T> {
	/// Get the instructions in the stream.
	#[must_use]
	pub fn instructions(&self) -> &[Instruction<T>] {
		&self.instructions
	}

	#[must_use]
	fn instructions_from_text(text: impl Iterator<Item = u8>) -> Vec<Instruction<T>> {
		text
			.filter_map(|byte| Instruction::try_from(byte).ok())
			.collect()
	}

	fn update_jump_points(&mut self) -> Result<(), Error> {
		let mut stack = Vec::<usize>::new();

		let stream = &mut self.instructions;

		for idx in 0..stream.len() {
			match stream.get(idx).unwrap() {
				Instruction::LoopStart(_) => {
					stack.push(idx);
				}
				Instruction::LoopEnd(_) => {
					let start_idx = stack.pop().ok_or(Error::UnmatchedEnd)?;
					stream[start_idx] = Instruction::LoopStart(idx.try_into().unwrap());
					stream[idx] = Instruction::LoopEnd(start_idx.try_into().unwrap());
				}
				_ => {}
			}
		}

		if stack.is_empty() {
			Ok(())
		} else {
			Err(Error::UnmatchedStart)
		}
	}
}
