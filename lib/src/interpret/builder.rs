use std::num::NonZeroU64;

use super::Interpreter;

/// Builds an [Interpreter].
#[derive(Debug)]
pub struct Builder<T, I, O> {
	input: I,
	output: O,
	array_len: usize,
	starting_ptr: usize,
	fill: T,
	instruction_limit: Option<NonZeroU64>,
}

impl<T, I, O> Builder<T, I, O> {
	/// Create a new builder.
	#[must_use]
	pub fn new(input: I, output: O, array_len: usize) -> Self
	where
		T: Default,
	{
		Self {
			output,
			input,
			array_len,
			starting_ptr: 0,
			fill: T::default(),
			instruction_limit: None,
		}
	}

	/// Build an [Interpreter] based on the parameters that have been set.
	pub fn build(self) -> Interpreter<T, I, O>
	where
		T: Clone,
	{
		Interpreter {
			data: vec![self.fill; self.array_len].into_boxed_slice(),
			input: self.input,
			output: self.output,
			ptr: self.starting_ptr,
			last_flush: std::time::Instant::now(),
			instructions_left: self.instruction_limit.map_or(0, NonZeroU64::get),
		}
	}

	/// Set the input.
	#[must_use]
	pub fn input<I2>(self, input: I2) -> Builder<T, I2, O> {
		Builder {
			input,
			output: self.output,
			array_len: self.array_len,
			starting_ptr: self.starting_ptr,
			fill: self.fill,
			instruction_limit: self.instruction_limit,
		}
	}

	/// Set the output.
	#[must_use]
	pub fn output<O2>(self, output: O2) -> Builder<T, I, O2> {
		Builder {
			output,
			input: self.input,
			array_len: self.array_len,
			starting_ptr: self.starting_ptr,
			fill: self.fill,
			instruction_limit: self.instruction_limit,
		}
	}

	/// Set the length of the data array.
	#[must_use]
	pub const fn array_len(mut self, array_len: usize) -> Self {
		self.array_len = array_len;
		self
	}

	/// Set the value that the data array will be initialized to.
	#[must_use]
	pub fn fill(self, fill: T) -> Self {
		Self { fill, ..self }
	}

	/// Set the starting position.
	#[must_use]
	pub const fn starting_ptr(mut self, ptr: usize) -> Self {
		self.starting_ptr = ptr;
		self
	}

	/// Set the instruction limit.
	#[must_use]
	pub const fn limit(mut self, limit: NonZeroU64) -> Self {
		self.instruction_limit = Some(limit);
		self
	}

	/// Remove the instruction limit.
	#[must_use]
	pub const fn no_limit(mut self) -> Self {
		self.instruction_limit = None;
		self
	}
}
