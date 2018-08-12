#![feature(plugin)]
#![plugin(expectation_plugin)]

#[macro_use]
extern crate expectation;

#[cfg(test)]
use expectation::extensions::*;
#[cfg(test)]
use expectation::*;
#[cfg(test)]
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

expectation_test! {
    fn expectation_test_foo(p: &mut Provider) {
        let mut w = p.text_writer("foo.txt");
        writeln!(w, "hello world");
    }
}
