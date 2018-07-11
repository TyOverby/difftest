use std::io::Write;

pub fn bitwise_equals(expected: &[u8], actual: &[u8]) -> bool {
    expected == actual
}

pub fn binary_diff(expected: Vec<u8>, actual: Vec<u8>) -> Vec<u8> {
    let mut out = vec![];
    writeln!(out, "expected (length {}):", expected.len()).unwrap();
    writeln!(out, "{:?}", expected).unwrap();

    writeln!(out, "actual (length {}):", actual.len()).unwrap();
    writeln!(out, "{:?}", actual).unwrap();
    out
}

pub fn text_diff(_expected: Vec<u8>, _actual: Vec<u8>) -> (String, Vec<u8>) {
    unimplemented!();
}
