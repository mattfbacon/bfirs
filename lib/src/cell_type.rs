//! Provides [`CellType`].

mod sealed {
	pub trait Sealed:
		Copy
		+ Default
		+ Eq
		+ From<u8>
		+ Into<u32>
		+ Into<u32>
		+ Ord
		+ std::fmt::Debug
		+ std::fmt::Display
		+ std::ops::Add<Self, Output = Self>
		+ std::ops::Rem<Self, Output = Self>
		+ std::ops::Sub<Self, Output = Self>
	{
		type NonZero: Copy + Eq + Into<Self> + TryFrom<Self> + std::fmt::Debug + std::fmt::Display;

		const ZERO: Self;
		const ONE: Self;
		const ONE_NON_ZERO: Self::NonZero;
		const MAX: u32;
		const C_TYPE: &'static str;

		fn wrapping_add(self, amount: Self) -> Self;
		fn wrapping_sub(self, amount: Self) -> Self;
		fn checked_add(self, amount: Self) -> Option<Self>;
		fn truncate_to_byte(self) -> u8;
	}
}

/// Types that can be used as the cell of the Brainfuck interpreter.
pub trait CellType: sealed::Sealed {}

macro_rules! impl_cell_type {
	($ty:ty, $non_zero_ty:ident, $c_type:expr) => {
		impl sealed::Sealed for $ty {
			type NonZero = std::num::$non_zero_ty;

			const ZERO: Self = 0;
			const ONE: Self = 1;
			const ONE_NON_ZERO: Self::NonZero = unsafe { std::num::$non_zero_ty::new_unchecked(1) };
			const MAX: u32 = <$ty>::MAX as u32;
			const C_TYPE: &'static str = $c_type;

			fn wrapping_add(self, amount: Self) -> Self {
				self.wrapping_add(amount)
			}

			fn wrapping_sub(self, amount: Self) -> Self {
				self.wrapping_sub(amount)
			}

			fn checked_add(self, amount: Self) -> Option<Self> {
				self.checked_add(amount)
			}

			fn truncate_to_byte(self) -> u8 {
				self as u8
			}
		}

		impl CellType for $ty {}
	};
}

impl_cell_type!(u8, NonZeroU8, "unsigned char");
impl_cell_type!(u16, NonZeroU16, "unsigned short");
impl_cell_type!(u32, NonZeroU32, "unsigned int");
