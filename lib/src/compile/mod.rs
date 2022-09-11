//! Compile and optimize Brainfuck input.

use std::num::NonZeroU32;

use crate::cell_type::CellType;
use crate::instruction::Instruction;

mod optimize;
mod render_c;

impl<T: CellType> Instruction<T> {
	fn fold_with(self, next: Self) -> Option<Self> {
		match (self, next) {
			(Self::Inc(amount1), Self::Inc(amount2)) => Some(Self::Inc(
				T::NonZero::try_from(amount1.into().wrapping_add(amount2.into())).ok()?,
			)),
			(Self::Dec(amount1), Self::Dec(amount2)) => Some(Self::Dec(
				T::NonZero::try_from(amount1.into().wrapping_add(amount2.into())).ok()?,
			)),
			(Self::Dec(sub), Self::Inc(add)) | (Self::Inc(add), Self::Dec(sub)) => {
				let add = add.into();
				let sub = sub.into();
				if sub > add {
					Some(Self::Dec(T::NonZero::try_from(sub - add).ok()?))
				} else {
					Some(Self::Inc(T::NonZero::try_from(add - sub).ok()?))
				}
			}
			(Self::IncPtr(amount1), Self::IncPtr(amount2)) => Some(Self::IncPtr(NonZeroU32::new(
				amount1.get().checked_add(amount2.get())?,
			)?)),
			(Self::DecPtr(amount1), Self::DecPtr(amount2)) => Some(Self::DecPtr(NonZeroU32::new(
				amount1.get().checked_add(amount2.get())?,
			)?)),
			(Self::DecPtr(sub), Self::IncPtr(add)) | (Self::IncPtr(add), Self::DecPtr(sub)) => {
				if sub > add {
					Some(Self::DecPtr(NonZeroU32::new(sub.get() - add.get())?))
				} else {
					Some(Self::IncPtr(NonZeroU32::new(add.get() - sub.get())?))
				}
			}
			(Self::Inc(..) | Self::Dec(..) | Self::Set(..), set @ Self::Set(..)) => Some(set),
			(Self::Set(start), Self::Inc(add)) => Some(Self::Set(start.wrapping_add(add.into()))),
			(Self::Set(start), Self::Dec(sub)) => Some(Self::Set(start.wrapping_sub(sub.into()))),
			_ => None,
		}
	}
}

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
		stream.insert_bf_jump_points()?;
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

	fn insert_bf_jump_points(&mut self) -> Result<(), Error> {
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
