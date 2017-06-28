//! Generate a Rust command line application.
#![deny(missing_docs)]
#[macro_use]
extern crate error_chain;
extern crate clap;

mod error;
mod run;

use std::io::{self, Write};
use std::process;

/// Userload Entry Point
fn main() {
    match run::run() {
        Ok(i) => process::exit(i),
        Err(e) => {
            writeln!(io::stderr(), "{}", e).expect("Unable to write to stderr!");
            process::exit(1)
        }
    }
}
