use std::num::NonZeroU32;

use super::{Error, InstructionStream};
use crate::cell_type::CellType;
use crate::instruction::Instruction;

enum FoldResult<T> {
	CantFold,
	NoOp,
	Folded(T),
}

impl<T: CellType> FoldResult<Instruction<T>> {
	fn try_from_or<A, B: TryFrom<A>>(
		or: Self,
		value: A,
		variant: impl FnOnce(B) -> Instruction<T>,
	) -> Self {
		B::try_from(value)
			.ok()
			.map(variant)
			.map_or(or, Self::Folded)
	}

	fn try_from_or_noop<A, B: TryFrom<A>>(
		value: A,
		variant: impl FnOnce(B) -> Instruction<T>,
	) -> Self {
		Self::try_from_or(Self::NoOp, value, variant)
	}
}

impl<T: CellType> Instruction<T> {
	fn fold_with(self, next: Self) -> FoldResult<Self> {
		match (self, next) {
			(Self::Inc(amount1), Self::Inc(amount2)) => {
				FoldResult::try_from_or_noop(amount1.into().wrapping_add(amount2.into()), Self::Inc)
			}
			(Self::Dec(amount1), Self::Dec(amount2)) => {
				FoldResult::try_from_or_noop(amount1.into().wrapping_add(amount2.into()), Self::Dec)
			}
			(Self::Dec(sub), Self::Inc(add)) | (Self::Inc(add), Self::Dec(sub)) => {
				let add = add.into();
				let sub = sub.into();
				if sub > add {
					FoldResult::try_from_or_noop(sub - add, Self::Dec)
				} else {
					FoldResult::try_from_or_noop(add - sub, Self::Inc)
				}
			}
			(Self::IncPtr(amount1), Self::IncPtr(amount2)) => amount1
				.get()
				.checked_add(amount2.get())
				.and_then(NonZeroU32::new)
				.map(Self::IncPtr)
				.map_or(FoldResult::CantFold, FoldResult::Folded),
			(Self::DecPtr(amount1), Self::DecPtr(amount2)) => amount1
				.get()
				.checked_add(amount2.get())
				.and_then(NonZeroU32::new)
				.map(Self::DecPtr)
				.map_or(FoldResult::CantFold, FoldResult::Folded),
			(Self::DecPtr(sub), Self::IncPtr(add)) | (Self::IncPtr(add), Self::DecPtr(sub)) => {
				if sub > add {
					FoldResult::try_from_or_noop(sub.get() - add.get(), Self::DecPtr)
				} else {
					FoldResult::try_from_or_noop(add.get() - sub.get(), Self::IncPtr)
				}
			}
			(Self::Inc(..) | Self::Dec(..) | Self::Set(..), set @ Self::Set(..)) => {
				FoldResult::Folded(set)
			}
			(Self::Set(start), Self::Inc(add)) => {
				FoldResult::Folded(Self::Set(start.wrapping_add(add.into())))
			}
			(Self::Set(start), Self::Dec(sub)) => {
				FoldResult::Folded(Self::Set(start.wrapping_sub(sub.into())))
			}
			_ => FoldResult::CantFold,
		}
	}
}

impl<T: CellType> InstructionStream<T> {
	/// Optimize the instruction stream.
	///
	/// # Errors
	///
	/// Will return `Err` if there are unmatched loop starts or ends.
	#[allow(clippy::missing_panics_doc)] // panic is exceptional
	pub fn optimize(&mut self) -> Result<(), Error> {
		self.fold_like();
		self.recognize_zeroings();
		self.fold_like();
		self.update_jump_points()?;

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
	fn fold_like(&mut self) {
		let stream = &mut self.instructions;

		let len = stream.len();
		let mut read_idx = 0;
		let mut write_idx = 0;
		'stream: while read_idx < len {
			let mut current = stream[read_idx];
			eprintln!("current is {current:?}");
			'fold: while let Some(&next) = stream.get({
				read_idx += 1;
				read_idx
			}) {
				eprintln!("next is {next:?}");
				match current.fold_with(next) {
					FoldResult::Folded(folded) => {
						eprintln!("optimizing into {folded:?}");
						current = folded;
					}
					FoldResult::NoOp => {
						eprintln!("optimizing into no op");
						read_idx += 1; // we did read the next instruction
						continue 'stream; // but skip writing instruction and incrementing `write_idx`
					}
					FoldResult::CantFold => {
						eprintln!("can't fold");
						break 'fold;
					}
				}
			}
			stream[write_idx] = current;
			write_idx += 1;
		}

		stream.truncate(write_idx);
	}

	fn recognize_zeroings(&mut self) {
		let stream = &mut self.instructions;

		let len = stream.len();
		let mut read_idx = 0;
		let mut write_idx = 0;

		'stream: while read_idx < len {
			if let &[Instruction::LoopStart(..), Instruction::Dec(amount) | Instruction::Inc(amount), Instruction::LoopEnd(..), ..] =
				&stream[read_idx..]
			{
				if amount == T::ONE_NON_ZERO {
					stream[write_idx] = Instruction::Set(T::ZERO);
					read_idx += 3;
					write_idx += 1;
					continue 'stream;
				}
			}
			stream[write_idx] = stream[read_idx];
			read_idx += 1;
			write_idx += 1;
		}

		stream.truncate(write_idx);
	}
}
