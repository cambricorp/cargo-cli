// Copyright (c) 2017 cargo-cli developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Create a command line interface binary with some common dependencies (([`clap`][clap] ||
//! [`docopt`][docopt]) and [`error_chain`][error_chain])
//!
//! # Installation
//! `cargo install cargo-cli`
//!
//! # Usage
//! In general, this is extension is used in the same manner as you would use `cargo new --bin`.
//! Most of the command line arguments supported by `cargo new` are supported by `cargo cli` and are
//! actually passed through to `cargo new`.  In addition, `cargo cli` supports specifying which
//! argument parser you would like to setup your CLI to use (clap or docopt).
//!
//! ```text
//! cargo-cli 0.1.0
//!
//! USAGE:
//!     cargo cli [FLAGS] [OPTIONS] <path>
//!
//! FLAGS:
//!         --frozen     Require Cargo.lock and cache are up to date
//!     -h, --help       Prints help information
//!         --locked     Require Cargo.lock is up to date
//!     -q, --quiet      No output printed to stdout
//!     -v, --verbose    Use verbose output (-vv very verbose/build.rs output)
//!
//! OPTIONS:
//!     -a, --arg_parser <PARSER>    Specify the argument parser to use [default: clap]
//!                                  [values: clap, docopt]
//!         --color <WHEN>           Coloring [default: auto]  [values: auto, always, never]
//!         --name <NAME>            Set the resulting package name, defaults to the value of
//!                                  <path>.
//!         --vcs <VCS>              Initialize a new repository for the given version control
//!                                  system or do not initialize any version control at all,
//!                                  overriding a global configuration. [default: git]
//!                                  [values: git, hg, pijul, fossil, none]
//!
//! ARGS:
//!     <path>
//! ```
//!
//! # Examples
//! ### With clap
//! ```cargo cli <path>```
//!
//! ### With docopt
//! ```cargo cli -a docopt <path>```
//!
//! ### With some `cargo new` arguments
//! ```cargo cli --vcs pijul -vv -a docopt --name flambe <path>```
//!
//! # CLI Layout
//!
//! ### Default
//! ```text
//! .
//! ├── Cargo.toml
//! └── src
//!     ├── error.rs
//!     ├── main.rs
//!     └── run.rs
//! ```
//!
//! ### With License(s)
//! ```text
//! . TODO
//! ```
//!
//! [clap]: https://clap.rs/
//! [docopt]: https://github.com/docopt/docopt.rs
//! [error_chain]: https://github.com/brson/error-chain
//!
#![deny(missing_docs)]
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;

extern crate clap;
extern crate rustache;
extern crate toml;

mod error;
mod run;
mod tmpl;

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
