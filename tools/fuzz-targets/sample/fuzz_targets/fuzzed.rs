fuzz_target!(|bytes: &[u8]| Fuzzed::read(bytes));
