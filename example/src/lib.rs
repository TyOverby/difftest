#![feature(plugin)]
#![plugin(expectation_plugin)]

extern crate expectation;

use expectation::Provider;

#[expectation]
fn tests_a_thing(p: &mut Provider) {

}
