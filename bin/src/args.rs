use std::path::PathBuf;

use anyhow::Context as _;
use strum_macros::EnumString;

#[derive(EnumString, Clone, Copy, Default)]
pub enum Mode {
	#[default]
	#[strum(serialize = "8")]
	U8,
	#[strum(serialize = "16")]
	U16,
	#[strum(serialize = "32")]
	U32,
}

#[derive(EnumString, Copy, Clone, Default)]
pub enum Output {
	#[default]
	#[strum(serialize = "interpret", serialize = "i")]
	Interpret,
	#[strum(serialize = "render", serialize = "c")]
	Render,
}

/// A low level brainfuck runtime.
#[derive(argh::FromArgs)]
struct CommandLine {
	/// read and run code from a given file
	#[argh(option, short = 'f')]
	file: Option<PathBuf>,

	/// read and run code from argv
	#[argh(option, short = 'a')]
	args: Option<String>,

	/// whether to use 8/16/32 bit mode, defaults to 8
	#[argh(option, short = 'm', default = "Default::default()")]
	mode: Mode,

	/// whether to 'interpret' or 'render' to C (shorthand i/c)
	#[argh(option, short = 'o', default = "Default::default()")]
	output: Output,

	/// an optional instruction limit for the interpreter
	#[argh(option, short = 'l')]
	limit: Option<u64>,
}

pub struct Args {
	pub mode: Mode,
	pub output: Output,
	pub code: Vec<u8>,
	pub instruction_limit: Option<u64>,
}

impl Args {
	pub fn from_env() -> anyhow::Result<Self> {
		let CommandLine {
			file,
			args,
			mode,
			output,
			limit,
		} = argh::from_env();

		let code = match (file, args) {
			(Some(_file), Some(_args)) => {
				return Err(anyhow::anyhow!("both file and args cannot be provided"));
			}
			(Some(file), None) => std::fs::read(&file).context("could not open file")?,
			(None, Some(args)) => args.into_bytes(),
			(None, None) => vec![],
		};

		Ok(Self {
			mode,
			output,
			code,
			instruction_limit: limit,
		})
	}
}
