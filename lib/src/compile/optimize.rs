use super::{Error, InstructionStream};
use crate::cell_type::CellType;
use crate::instruction::Instruction;

impl<T: CellType> InstructionStream<T> {
	/// Optimize the instruction stream.
	///
	/// # Errors
	///
	/// Will return `Err` if there are unmatched loop starts or ends.
	#[allow(clippy::missing_panics_doc)] // panic is exceptional
	pub fn optimize(&mut self) -> Result<(), Error> {
		self.group_common_bf();
		self.static_optimize();
		self.insert_bf_jump_points()?;

		self.recommended_array_size = self
			.instructions
			.iter()
			.filter_map(|instruction| match instruction {
				Instruction::IncPtr(amount) => Some(usize::try_from(amount.get()).unwrap()),
				_ => None,
			})
			.sum::<usize>()
			.max(Self::MIN_ARRAY_SIZE);

		Ok(())
	}

	// without this inline attr it fails to inline this function into the main loop, preventing a considerable speedup
	#[inline]
	fn group_common_bf(&mut self) {
		let stream = &mut self.instructions;

		let len = stream.len();
		let mut new_len = len;
		let mut idx = 0;
		while idx < len {
			if let Some(&next) = stream.get(idx + 1) {
				if let Some(new) = stream[idx].fold_with(next) {
					stream[idx] = new;
					idx += 1;
					new_len -= 1;
				}
			}
		}

		stream.truncate(new_len);
	}

	fn static_optimize(&mut self) {
		const OPT_COUNT: usize = 2;

		let v = &mut self.instructions;

		let static_tree: [(&[Instruction<T>], Instruction<T>); OPT_COUNT] = [
			(
				&[
					Instruction::LoopStart(0),
					Instruction::Dec(T::ONE_NON_ZERO),
					Instruction::LoopEnd(0),
				],
				Instruction::Set(T::ZERO),
			),
			(
				&[
					Instruction::LoopStart(0),
					Instruction::Inc(T::ONE_NON_ZERO),
					Instruction::LoopEnd(0),
				],
				Instruction::Set(T::ZERO),
			),
		];

		let mut optimized_count = 1;

		while optimized_count != 0 {
			optimized_count = 0;

			let mut paths = [0usize; OPT_COUNT];

			let mut new_idx = 0usize;

			let mut i = 0;
			while i < v.len() {
				let mut optimized = None::<(Instruction<T>, usize)>;

				'opt: for (idx, p) in paths.iter_mut().enumerate() {
					if v[i] == static_tree[idx].0[*p] {
						*p += 1;
					} else {
						*p = 0;
						if v[i] == static_tree[idx].0[0] {
							*p += 1;
						}
					}

					if *p == static_tree[idx].0.len() {
						optimized = Some((static_tree[idx].1, *p));
						break 'opt;
					}
				}

				v[new_idx] = v[i];
				new_idx += 1;

				if let Some((ins, cnt)) = optimized {
					optimized_count += 1;
					paths = [0; OPT_COUNT];

					new_idx -= cnt;
					v[new_idx] = ins;
					new_idx += 1;
				}

				i += 1;
			}
			v.truncate(new_idx);
		}
	}
}
