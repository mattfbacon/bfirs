//! Provides [`Instruction`] and conversion from `u8`.

use std::num::NonZeroU32;

use crate::cell_type::CellType;

/// An optimized Brainfuck instruction.
///
/// Some of the variants do not have a direct Brainfuck representation and only occur in optimized instruction streams.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Instruction<C: CellType> {
	/// Set the value of the current cell
	///
	/// With a value of `0`, equivalent to `[-]`.
	Set(C),
	/// Output the value of the current cell.
	///
	/// Equivalent to `.`.
	Write,
	/// Read input into the current cell.
	///
	/// Equivalent to `,`.
	Read,
	/// Start a loop.
	///
	/// The `u32` field stores the position of the end of the loop.
	/// When not optimizing, it can be set to `0`.
	///
	/// Equivalent to `[`.
	LoopStart(u32),
	/// End a loop.
	///
	/// The `u32` field stores the position of the start of the loop.
	/// When not optimizing, it can be set to `0`.
	///
	/// Equivalent to `]`.
	LoopEnd(u32),
	/// Add the contained value to the value in the current cell.
	///
	/// With a value of `1`, equivalent to `+`.
	Inc(C::NonZero),
	/// Subtract the contained value from the value in the current cell.
	///
	/// With a value of `1`, equivalent to `-`.
	Dec(C::NonZero),
	/// Add the contained value to the pointer.
	///
	/// With a value of `1`, equivalent to `<`
	IncPtr(NonZeroU32),
	/// Subtract the contained value from the pointer.
	///
	/// No direct equivalent. Emitted by the optimizer.
	DecPtr(NonZeroU32),
}

/// The error that occurs when attempting to convert a non-instruction character to [`Instruction`].
#[derive(Debug, thiserror::Error, Clone, Copy)]
#[error("not an instruction")]
#[allow(clippy::module_name_repetitions)] // clearer
pub struct NotInstruction;

impl<T: CellType> TryFrom<u8> for Instruction<T> {
	type Error = NotInstruction;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			b'+' => Self::Inc(T::ONE_NON_ZERO),
			b'-' => Self::Dec(T::ONE_NON_ZERO),
			b'>' => Self::IncPtr(NonZeroU32::new(1).unwrap()),
			b'<' => Self::DecPtr(NonZeroU32::new(1).unwrap()),
			b'.' => Self::Write,
			b',' => Self::Read,
			// Jump points must be computed later by the full stream parser
			b'[' => Self::LoopStart(0),
			b']' => Self::LoopEnd(0),
			_ => {
				return Err(NotInstruction);
			}
		})
	}
}
