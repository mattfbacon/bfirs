use std::io;

use super::Interpreter;

/// Builds an [Interpreter].
#[derive(Debug)]
pub struct Builder<T, I, O> {
	input: I,
	output: O,
	data_array_size: usize,
	initial_data_pointer: usize,
	fill: T,
	#[cfg(feature = "limited")]
	instruction_limit: Option<u64>,
}

impl<'i, 'o, T: Default> Builder<T, io::StdinLock<'i>, io::StdoutLock<'o>> {
	/// Create a new builder, using stdin for the input and stdout for the output.
	#[must_use]
	pub fn stdio() -> Self {
		Self::new(io::stdin().lock(), io::stdout().lock())
	}
}

impl<T, I, O> Builder<T, I, O> {
	/// Create a new builder.
	#[must_use]
	pub fn new(input: I, output: O) -> Self
	where
		T: Default,
	{
		Self {
			output,
			input,
			data_array_size: crate::MIN_DATA_ARRAY_SIZE,
			initial_data_pointer: 0,
			fill: T::default(),
			#[cfg(feature = "limited")]
			instruction_limit: None,
		}
	}

	/// Build an [Interpreter] based on the parameters that have been set.
	pub fn build(self) -> Interpreter<T, I, O>
	where
		T: Clone,
	{
		Interpreter {
			data: vec![self.fill; self.data_array_size].into_boxed_slice(),
			input: self.input,
			output: self.output,
			data_pointer: self.initial_data_pointer,
			last_flush: std::time::Instant::now(),
			#[cfg(feature = "limited")]
			instructions_left: self.instruction_limit,
		}
	}

	/// Set the input.
	#[must_use]
	pub fn input<I2>(self, input: I2) -> Builder<T, I2, O> {
		Builder {
			input,
			output: self.output,
			data_array_size: self.data_array_size,
			initial_data_pointer: self.initial_data_pointer,
			fill: self.fill,
			#[cfg(feature = "limited")]
			instruction_limit: self.instruction_limit,
		}
	}

	/// Set the output.
	#[must_use]
	pub fn output<O2>(self, output: O2) -> Builder<T, I, O2> {
		Builder {
			output,
			input: self.input,
			data_array_size: self.data_array_size,
			initial_data_pointer: self.initial_data_pointer,
			fill: self.fill,
			#[cfg(feature = "limited")]
			instruction_limit: self.instruction_limit,
		}
	}

	/// Set the size of the data array.
	#[must_use]
	pub const fn data_array_size(mut self, size: usize) -> Self {
		self.data_array_size = size;
		self
	}

	/// Set the value that the data array will be initialized to.
	#[must_use]
	pub fn fill(self, fill: T) -> Self {
		Self { fill, ..self }
	}

	/// Set the initial position of the data pointer.
	#[must_use]
	pub const fn initial_data_pointer(mut self, ptr: usize) -> Self {
		self.initial_data_pointer = ptr;
		self
	}

	/// Set the instruction limit.
	#[cfg(feature = "limited")]
	#[must_use]
	pub const fn instruction_limit(mut self, limit: u64) -> Self {
		self.instruction_limit = Some(limit);
		self
	}

	/// Remove the instruction limit.
	#[cfg(feature = "limited")]
	#[must_use]
	pub const fn no_instruction_limit(mut self) -> Self {
		self.instruction_limit = None;
		self
	}

	/// Configure the interpreter based on the given instruction stream.
	#[must_use]
	pub fn configure_for(mut self, stream: &crate::compile::InstructionStream<T>) -> Self
	where
		T: crate::cell_type::CellType,
	{
		self.data_array_size = stream.recommended_array_size();
		self
	}
}
