use anyhow::Context as _;
use bfirs::{InstructionStream, Interpreter};

mod args;
use args::{Mode, Output};

fn main() -> anyhow::Result<()> {
	let args = args::Args::from_env().context("parsing arguments")?;

	macro_rules! run_different_sizes {
		($ty:ty) => {{
			let code =
				InstructionStream::optimized_from_code(args.code.into_iter()).context("optimizing")?;

			match args.output {
				Output::Interpret => {
					let mut interpreter = Interpreter::new_stdio::<$ty>(code.recommended_array_size());
					if let Some(limit) = args.instruction_limit {
						interpreter.set_instruction_limit(limit);
					}
					interpreter.run(code.instructions()).context("executing")
				}
				Output::Render => code
					.render_c(std::io::stdout().lock())
					.context("rendering C code"),
			}
		}};
	}

	match args.mode {
		Mode::U8 => run_different_sizes!(u8),
		Mode::U16 => run_different_sizes!(u16),
		Mode::U32 => run_different_sizes!(u32),
	}
}
