use super::Interpreter;

/// Builds an [Interpreter].
#[derive(Debug)]
pub struct Builder<T, I, O> {
	input: I,
	output: O,
	array_len: usize,
	initial_data_pointer: usize,
	fill: T,
	#[cfg(feature = "limited")]
	instruction_limit: Option<u64>,
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
			data: vec![self.fill; self.array_len].into_boxed_slice(),
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
			array_len: self.array_len,
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
			array_len: self.array_len,
			initial_data_pointer: self.initial_data_pointer,
			fill: self.fill,
			#[cfg(feature = "limited")]
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

	/// Set the initial position of the data pointer.
	#[must_use]
	pub const fn initial_data_pointer(mut self, ptr: usize) -> Self {
		self.initial_data_pointer = ptr;
		self
	}

	/// Set the instruction limit.
	#[cfg(feature = "limited")]
	#[must_use]
	pub const fn limit(mut self, limit: u64) -> Self {
		self.instruction_limit = Some(limit);
		self
	}

	/// Remove the instruction limit.
	#[cfg(feature = "limited")]
	#[must_use]
	pub const fn no_limit(mut self) -> Self {
		self.instruction_limit = None;
		self
	}
}
