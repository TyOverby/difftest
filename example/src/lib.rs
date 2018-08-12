#![feature(plugin)]
#![plugin(expectation_plugin)]

extern crate expectation;

use expectation::*;
use expectation::extensions::*;
use std::io::Write;

#[expectation]
fn test_with_annotation(p: &mut Provider) {
    let mut w = p.text_writer("foo.txt");
    writeln!(w, "a");
    writeln!(w, "b");
    writeln!(w, "c");
    writeln!(w, "d");
    writeln!(w, "e");
}