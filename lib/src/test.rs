use rand::distributions::Distribution as _;
use rand::Rng as _;

use crate::interpret::Error;

macro_rules! pipeline_tests {
	(@expect ($actual:expr) == Ok($expected:expr)) => {
		assert_eq!($actual.as_ref().unwrap(), $expected);
	};

	(@expect ($actual:expr) == Err($error:pat)) => {
		match $actual.as_ref().unwrap_err() {
			$error => (),
			other => panic!("expected {}, got {other:?}", stringify!($error)),
		}
	};

	($($input:expr => $variant:ident($($content:tt)*)),* $(,)?) => {
		#[test]
		fn full_pipeline() {
			$({
				let result = run_output($input, true);
				pipeline_tests!(@expect (&result) == $variant($($content)*));
			})*
		}
	};
}

fn run_output(input: &str, optimize: bool) -> Result<Vec<u8>, Error> {
	let mut stream = crate::InstructionStream::<u8>::from_code(input.bytes()).unwrap();
	if optimize {
		stream.optimize().unwrap();
	}
	let mut out = Vec::new();
	let mut interpreter = crate::Interpreter::build(std::io::empty(), &mut out)
		.instruction_limit(1_000_000)
		.build();
	let result = interpreter.run(stream.instructions());
	result.map(|()| out)
}

pipeline_tests![
	"++++[>++++[>++++<-]<-]>>+." => Ok(&b"A"),
	"<" => Err(Error::Underflow),
	"+[>+]" => Err(Error::Overflow),
	"+[]" => Err(Error::NotEnoughInstructions),
];

fn generate_random_code() -> String {
	const NUM_SECTIONS: usize = 20;
	const NON_LOOP_CHARS: &[u8] = b"+-<>.."; // `.` is doubled to have a higher probability

	let loop_distribution = rand::distributions::Bernoulli::new(0.3).unwrap();
	let mut ret = String::new();

	for _ in 0..NUM_SECTIONS {
		let should_loop = loop_distribution.sample(&mut rand::thread_rng());
		if should_loop {
			ret.push('[');
		}
		let num_chars = rand::thread_rng().gen_range(0..100);
		ret.extend(
			rand::seq::SliceRandom::choose_multiple(NON_LOOP_CHARS, &mut rand::thread_rng(), num_chars)
				.copied()
				.map(char::from),
		);
		if should_loop {
			ret.push(']');
		}
	}

	ret
}

#[test]
fn optimization_fuzzer() {
	const NUM_FUZZES: usize = 250;
	for _ in 0..NUM_FUZZES {
		let code = generate_random_code();
		eprintln!("fuzzing optimization of {code:?}");
		let unoptimized = run_output(&code, false);
		let optimized = run_output(&code, true);

		if matches!(optimized, Ok(..)) {
			if let Err(error) = &unoptimized {
				match error {
					// the optimizer can remove instructions and allow it to finish
					Error::NotEnoughInstructions => continue,
					// these can cause {under,over}flows but are optimized out
					Error::Overflow | Error::Underflow if code.contains("<>") || code.contains("><") => {
						continue
					}
					_ => (),
				}
			}
		}

		// we don't want to compare the exact error that occurred, just that an error occurred.
		// preserving the same error is not within the optimizer's guarantees.
		eprintln!("unoptimized result: {unoptimized:?}");
		eprintln!("optimized result: {optimized:?}");
		assert_eq!(unoptimized.ok(), optimized.ok());
	}
}
