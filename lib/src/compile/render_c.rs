use std::io;

use super::InstructionStream;
use crate::cell_type::CellType;

impl<T: CellType> InstructionStream<T> {
	/// Renders this instruction stream as C code to the writer `out`.
	///
	/// # Errors
	///
	/// Returns `Err` iff writing to `out` returns `Err`.
	pub fn render_c(&self, mut out: impl io::Write) -> io::Result<()> {
		let c_type = T::C_TYPE;
		let arr_size = self.recommended_array_size;
		writeln!(out, "#include <stdio.h>")?;
		writeln!(out, "typedef {c_type} bf_cell_t;")?;
		writeln!(out, "static bf_cell_t arr[{arr_size}] = {{0,}};")?;
		writeln!(out, "int main() {{")?;
		writeln!(out, "\tbf_cell_t* cursor = arr;")?;

		for instruction in &self.instructions {
			use crate::instruction::Instruction as I;

			write!(out, "\t")?;

			match instruction {
				I::Set(amount) => writeln!(out, "*cursor = {amount};"),
				I::Write => writeln!(out, "putchar(*cursor);"),
				I::Read => writeln!(
					out,
					"*cursor = getchar(); if (*cursor == EOF) {{ *cursor = 0; }}"
				),
				I::LoopStart(_) => writeln!(out, "while (*cursor != 0) {{"),
				I::LoopEnd(_) => writeln!(out, "}}"),
				I::Inc(amount) => writeln!(out, "*cursor += {amount};"),
				I::Dec(amount) => writeln!(out, "*cursor -= {amount};"),
				I::IncPtr(amount) => writeln!(out, "cursor += {amount};"),
				I::DecPtr(amount) => writeln!(out, "cursor -= {amount};"),
			}?;
		}

		writeln!(out, "}}")?;
		Ok(())
	}
}
