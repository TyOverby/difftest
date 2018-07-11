use std::io::Write;

pub fn bitwise_equals(expected: &[u8], actual: &[u8]) -> bool {
    expected == actual
}

pub fn binary_diff(expected: &[u8], actual: &[u8]) -> Vec<u8> {
    let mut out = vec![];
    writeln!(out, "expected (length {}):", expected.len());
    writeln!(out, "{:?}", expected);

    writeln!(out, "actual (length {}):", actual.len());
    writeln!(out, "{:?}", actual);
    out
}

pub fn text_diff(expected: &[u8], actual: &[u8]) -> Vec<u8> {
    unimplemented!();
}
