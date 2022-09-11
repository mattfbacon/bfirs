//! Interpret Brainfuck code.

use std::io;
use std::time::Instant;

use thiserror::Error;

use crate::cell_type::CellType;
use crate::instruction::Instruction;

mod builder;
pub use builder::Builder;

/// Errors that can occur while interpreting Brainfuck.
#[derive(Debug, Error)]
pub enum Error {
	/// Runtime overflowed its data array.
	///
	/// Occurs when the pointer is already at the end of the data and the program attempts to increment the pointer.
	#[error("runtime overflowed its data array")]
	Overflow,
	/// Runtime underflowed its data array.
	///
	/// Occurs when the pointer is already at the start of the data (0) and the program attempts to decrement the pointer.
	#[error("runtime underflowed its data array")]
	Underflow,
	/// The pointer was already overflowed when the runtime started.
	///
	/// Occurs when the `starting_ptr` has been set above the `array_len` when using the [Builder].
	#[error("the pointer was already overflowed when the runtime started")]
	InitOverflow,
	/// The instruction limit was reached.
	#[error("instruction limit reached. not enough instructions to complete this task. task halted before completion.")]
	NotEnoughInstructions,
	/// An IO error occurred while reading from the input.
	#[error("IO error while reading from input: {0}")]
	InputIo(io::Error),
	/// An IO error occurred while writing to the output.
	#[error("IO error while writing to output: {0}")]
	OutputIo(io::Error),
}

/// A Brainfuck interpreter.
///
/// Many parameters can be customized.
/// Use `new_stdio` for the default configuration, or see [Builder] for the options that can be configured.
#[derive(Debug)]
pub struct Interpreter<T, I, O> {
	output: O,
	input: I,
	data: Box<[T]>,
	ptr: usize,
	last_flush: Instant,
	#[cfg(feature = "limited")]
	instructions_left: Option<u64>,
}

impl Interpreter<(), (), ()> {
	/// Start building a new interpreter.
	#[must_use]
	pub fn build<T: Default, I, O>(input: I, output: O, array_len: usize) -> Builder<T, I, O> {
		Builder::new(input, output, array_len)
	}

	/// Create a new interpreter using stdin and stdout.
	#[must_use]
	pub fn new_stdio<'i, 'o, T: Clone + Default>(
		array_len: usize,
	) -> Interpreter<T, io::StdinLock<'i>, io::StdoutLock<'o>> {
		Builder::new(io::stdin().lock(), io::stdout().lock(), array_len).build()
	}
}

impl<T: CellType, I: io::Read, O: io::Write> Interpreter<T, I, O> {
	#[inline]
	#[must_use]
	unsafe fn cur_unchecked(&self) -> T {
		// SAFETY: The caller has asserted that the current pointer is a valid index
		debug_assert!(self.ptr < self.data.len());
		*self.data.get_unchecked(self.ptr)
	}

	#[inline]
	unsafe fn map_current(&mut self, func: impl FnOnce(T) -> T) {
		// SAFETY: The caller has asserted that the current pointer is a valid index
		debug_assert!(self.ptr < self.data.len());
		*self.data.get_unchecked_mut(self.ptr) = func(self.cur_unchecked());
	}

	#[inline]
	fn inc_ptr_by(&mut self, v: usize) -> Result<(), Error> {
		self.ptr += v;
		if self.ptr >= self.data.len() {
			self.ptr -= v;
			return Err(Error::Overflow);
		}
		Ok(())
	}

	#[inline]
	fn dec_ptr_by(&mut self, v: usize) -> Result<(), Error> {
		self.ptr = self.ptr.checked_sub(v).ok_or(Error::Underflow)?;
		Ok(())
	}

	#[inline]
	fn write(&mut self, v: u8) -> Result<(), Error> {
		self.output.write_all(&[v]).map_err(Error::OutputIo)?;

		// based on 60 fps update (actually 62.5)
		if self.last_flush.elapsed().as_millis() > 16 {
			self.output.flush().map_err(Error::OutputIo)?;
			self.last_flush = Instant::now();
		}

		Ok(())
	}

	#[inline]
	fn read(&mut self) -> Result<u8, Error> {
		<&mut I as io::Read>::bytes(&mut self.input)
			.next()
			.unwrap_or(Ok(0))
			.map_err(Error::InputIo)
	}

	/// Run the interpreter on the given instruction stream.
	///
	/// If you want to run an interpreter based on the output of the compiler, use the `instructions` method on the compiler to get the instructions.
	///
	/// # Errors
	///
	/// See the variants of [Error].
	#[allow(clippy::missing_panics_doc)] // panics are exceptional
	pub fn run(&mut self, stream: &[Instruction<T>]) -> Result<(), Error> {
		let mut instruction_pointer = 0usize;
		let len = stream.len();

		// SAFETY: check the pointer now to ensure it's in bounds before any `_unchecked` ops assume so.
		if self.ptr >= self.data.len() {
			return Err(Error::InitOverflow);
		}

		// SAFETY: `ptr` bounds are checked by `ptr` mutating operations, so it will remain valid within this block.
		while instruction_pointer < len {
			#[cfg(feature = "limited")]
			if let Some(0) = self.instructions_left {
				return Err(Error::NotEnoughInstructions);
			}

			unsafe {
				use Instruction as I;
				match *stream.get_unchecked(instruction_pointer) {
					I::Set(value) => self.map_current(|_| value),
					I::Inc(amount) => self.map_current(|c| c.wrapping_add(amount.into())),
					I::Dec(amount) => self.map_current(|c| c.wrapping_sub(amount.into())),
					I::IncPtr(by) => self.inc_ptr_by(usize::try_from(by.get()).unwrap())?,
					I::DecPtr(by) => self.dec_ptr_by(usize::try_from(by.get()).unwrap())?,
					I::Write => self.write(self.cur_unchecked().truncate_to_byte())?,
					I::Read => {
						let new = self.read()?.into();
						self.map_current(|_| new);
					}
					I::LoopStart(end) => {
						if self.cur_unchecked() == T::ZERO {
							instruction_pointer = end as usize;
						}
					}
					I::LoopEnd(start) => {
						if self.cur_unchecked() != T::ZERO {
							instruction_pointer = start as usize;
						}
					}
				}
			}

			instruction_pointer += 1;

			#[cfg(feature = "limited")]
			if let Some(left) = &mut self.instructions_left {
				*left = left.checked_sub(1).ok_or(Error::NotEnoughInstructions)?;
			}
		}

		Ok(())
	}

	/// Get the number of instructions remaining.
	///
	/// Returns `None` if there is no instruction limit.
	#[cfg(feature = "limited")]
	#[must_use]
	pub fn instructions_left(&self) -> Option<u64> {
		self.instructions_left
	}

	/// Set the instruction limit.
	///
	/// Enables the limit if it was not already enabled.
	#[cfg(feature = "limited")]
	pub fn set_instruction_limit(&mut self, left: u64) {
		self.instructions_left = Some(left);
	}

	/// Remove the instruction limit if one existed.
	#[cfg(feature = "limited")]
	pub fn remove_instruction_limit(&mut self) {
		self.instructions_left = None;
	}
}
