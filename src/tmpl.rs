//! `cargo-cli` template files

/// Contaier for file templates for various auto-generated files.
pub struct Templates {
    /// The dependencies to add.
    deps: &'static str,
    /// The `main.rs` replacement.
    main: &'static str,
    /// The `run.rs` file.
    run: &'static str,
    /// The `error.rs` file.
    error: &'static str,
    /// The license prefix.
    prefix: &'static str,
    /// The `LICENSE-MIT` file.
    mit: Option<&'static str>,
    /// The `LICENSE-APACHE` file.
    apache: Option<&'static str>,
}

impl Templates {
    /// Create a new template use for file creation.
    pub fn new(clap: bool, mit: bool, apache: bool) -> Templates {
        let mut template = Templates {
            deps: "",
            main: "",
            run: "",
            error: "",
            prefix: "",
            mit: None,
            apache: None,
        };

        if mit && apache {
            template.prefix = PREFIX_BOTH;
        }
        if mit {
            template.mit = Some(LICENSE_MIT);
            if !apache {
                template.prefix = PREFIX_MIT;
            }
        }
        if apache {
            template.apache = Some(LICENSE_APACHE);
            if !mit {
                template.prefix = PREFIX_APACHE;
            }
        }

        if clap {
            // Setup clap templates
            template.deps = CLAP_DEPS;
            template.main = CLAP_MAIN_RS;
            template.run = CLAP_RUN_RS;
            template.error = CLAP_ERROR_RS;
        } else {
            // Setup docopt templates
            template.deps = DOCOPT_DEPS;
            template.main = DOCOPT_MAIN_RS;
            template.run = DOCOPT_RUN_RS;
            template.error = DOCOPT_ERROR_RS;
        }
        template
    }

    /// Get the `deps` value.
    pub fn deps(&self) -> &str {
        self.deps
    }

    /// Get the `main` value.
    pub fn main(&self) -> &str {
        self.main
    }

    /// Get the `run` value.
    pub fn run(&self) -> &str {
        self.run
    }

    /// Get the `error` value.
    pub fn error(&self) -> &str {
        self.error
    }

    /// Get the `prefix` value.
    pub fn prefix(&self) -> &str {
        self.prefix
    }

    /// Get the `mit` value.
    pub fn mit(&self) -> Option<&str> {
        self.mit
    }

    /// Get the `apache` value.
    pub fn apache(&self) -> Option<&str> {
        self.apache
    }

    /// Does this set of templates include license information?
    pub fn has_license(&self) -> bool {
        self.mit.is_some() || self.apache.is_some()
    }
}

/// clap version of dependencies
const CLAP_DEPS: &'static str = "clap = \"2.25.0\"\n\
error-chain = \"0.10.0\"\n";

/// clap version of `main.rs`
const CLAP_MAIN_RS: &'static str = "#![deny(missing_docs)]\n\
#[macro_use]\n\
extern crate error_chain;\n\
extern crate clap;\n\n\
mod error;\n\
mod run;\n\n\
use std::io::{self, Write};\n\
use std::process;\n\n\
/// CLI Entry Point\n\
fn main() {\n    \
match run::run() {\n        \
Ok(i) => process::exit(i),\n        \
Err(e) => {\n            \
writeln!(io::stderr(), \"{}\", e).expect(\"Unable to write to stderr!\");\n            \
process::exit(1)\n        \
}\n    \
}\n\
}\n";

/// clap version of `run.rs`
const CLAP_RUN_RS: &'static str = "use clap::App;\n\
use error::Result;\n\
use std::io::{self, Write};\n\n\
/// CLI Runtime\n\
pub fn run() -> Result<i32> {\n    \
let _matches = App::new(env!(\"CARGO_PKG_NAME\"))\n        \
.version(env!(\"CARGO_PKG_VERSION\"))\n        \
.author(env!(\"CARGO_PKG_AUTHORS\"))\n        \
.about(\"Prints 'Hello, Rustaceans!' to stdout\")\n        \
.get_matches();\n\n    \
writeln!(io::stdout(), \"Hello, Rustaceans!\")?;\n    \
Ok(0)\n\
}";

/// clap version of `error.rs`
const CLAP_ERROR_RS: &'static str = "error_chain!{\n    \
foreign_links {\n        \
Io(::std::io::Error);\n    \
}\n\
}\n";

/// docopt version of dependencies
const DOCOPT_DEPS: &'static str = "docopt = \"0.8.1\"\n\
error-chain = \"0.10.0\"\n\
serde = \"1.0.8\"\n\
serde_derive = \"1.0.8\"\n";

/// docopt version of `main.rs`
const DOCOPT_MAIN_RS: &'static str = "#![deny(missing_docs)]\n\
#[macro_use]\n\
extern crate error_chain;\n\
#[macro_use]\n\
extern crate serde_derive;\n\
extern crate docopt;\n\n\
mod error;\n\
mod run;\n\n\
use std::io::{self, Write};\n\
use std::process;\n\n\
/// CLI Entry Point\n\
fn main() {\n    \
match run::run() {\n        \
Ok(i) => process::exit(i),\n        \
Err(e) => {\n            \
writeln!(io::stderr(), \"{}\", e).expect(\"Unable to write to stderr!\");\n            \
process::exit(1)\n        \
}\n    \
}\n\
}\n";

/// docopt version of `run.rs`
const DOCOPT_RUN_RS: &'static str = "use docopt::Docopt;\n\
use error::Result;\n\
use std::io::{self, Write};\n\n\
/// Write the Docopt usage string.\n\
const USAGE: &'static str = \"\n\
Usage: {{ name }} ( -h | --help )\n       \
{{ name }} ( -V | --version )\n\n\
Options:\n    \
-h --help     Show this screen.\n    \
-v --version  Show version.\n\
\";\n\n\
/// Command line arguments\n\
#[derive(Debug, Deserialize)]\n\
struct Args;\n\n\
/// CLI Runtime\n\
pub fn run() -> Result<i32> {\n    \
let _args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize())?;\n    \
writeln!(io::stdout(), \"Hello, Rustaceans!\")?;\n    \
Ok(0)\n\
}";

/// docopt version of `error.rs`
const DOCOPT_ERROR_RS: &'static str = "error_chain!{\n    \
foreign_links {\n        \
Docopt(::docopt::Error);\n        \
Io(::std::io::Error);\n    \
}\n\
}\n";

const PREFIX_BOTH: &'static str = "// Copyright (c) 2017 ##NAME## developers\n\
//\n\
// Licensed under the Apache License, Version 2.0\n\
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT\n\
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your\n\
// option. All files in the project carrying such notice may not be copied,\n\
// modified, or distributed except according to those terms.\n\n";

const PREFIX_MIT: &'static str = "// Copyright (c) 2017 ##NAME## developers\n\
//\n\
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>. \n\
// All files in the project carrying such notice may not be copied,\n\
// modified, or distributed except according to those terms.\n\n";

const PREFIX_APACHE: &'static str = "// Copyright (c) 2017 ##NAME## developers\n\
//\n\
// Licensed under the Apache License, Version 2.0\n\
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>. \n\
// All files in the project carrying such notice may not be copied,\n\
// modified, or distributed except according to those terms.\n\n";

const LICENSE_MIT: &'static str = "";
const LICENSE_APACHE: &'static str = "";