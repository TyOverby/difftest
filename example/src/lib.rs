#![allow(dead_code, unused_imports)]

extern crate expectation;
extern crate expectation_plugin;

use expectation::{extensions::*, *};
use expectation_plugin::expectation_test;
use std::io::Write;

#[expectation_test]
fn test_with_annotation(p: Provider) {
    let mut w = p.text_writer("foo.txt");
    writeln!(w, "a");
    writeln!(w, "b");
    writeln!(w, "c");
    writeln!(w, "d");
    writeln!(w, "e");
}
