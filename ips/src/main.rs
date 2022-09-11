use std::io;
use std::num::{NonZeroU32, NonZeroU8};
use std::time::Instant;

use bfirs::{Instruction, Interpreter};

type Cell = u8;

const PROGRAM: &[Instruction<Cell>] = &[
	Instruction::Inc(unsafe { NonZeroU8::new_unchecked(1) }),
	Instruction::LoopStart(6),
	Instruction::IncPtr(unsafe { NonZeroU32::new_unchecked(1) }),
	Instruction::Dec(unsafe { NonZeroU8::new_unchecked(2) }),
	Instruction::Inc(unsafe { NonZeroU8::new_unchecked(4) }),
	Instruction::DecPtr(unsafe { NonZeroU32::new_unchecked(1) }),
	Instruction::LoopEnd(1),
];
const ITERATIONS: u64 = 100_000;

fn main() {
	let mut interpreter = Interpreter::build::<u8, _, _>(io::empty(), io::sink())
		.instruction_limit(ITERATIONS)
		.build();

	let start = Instant::now();

	let ret = interpreter.run(PROGRAM);

	let elapsed = start.elapsed();

	assert!(matches!(
		ret,
		Err(bfirs::interpret::Error::NotEnoughInstructions)
	));

	println!(
		"executed {ITERATIONS} instructions in {} nanoseconds",
		elapsed.as_nanos()
	);
}
