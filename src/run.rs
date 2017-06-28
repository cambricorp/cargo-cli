//! `cargo-cli` runtime operation
use clap::{App, Arg};
use error::{ErrorKind, Result};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

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
Usage: blah ( -h | --help )\n       \
blah ( -V | --version )\n\n\
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

/// Parse the args, and execute the generated commands.
pub fn run() -> Result<i32> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Creates a Rust command line application")
        .arg(Arg::with_name("vcs")
                 .long("vcs")
                 .value_name("VCS")
                 .help("Initialize a new repository for the given version control system
                        or do not initialize any version control at all, overriding a
                        global configuration.")
                 .possible_values(&["git", "hg", "pijul", "fossil", "none"])
                 .default_value("git")
                 .takes_value(true))
        .arg(Arg::with_name("name")
                 .long("name")
                 .value_name("NAME")
                 .help("Set the resulting package name, defaults to the value of <path>.")
                 .takes_value(true))
        .arg(Arg::with_name("color")
                 .long("color")
                 .value_name("WHEN")
                 .help("Coloring")
                 .possible_values(&["auto", "always", "never"])
                 .default_value("auto")
                 .takes_value(true))
        .arg(Arg::with_name("frozen")
                 .long("frozen")
                 .conflicts_with("locked")
                 .help("Require Cargo.lock and cache are up to date"))
        .arg(Arg::with_name("locked")
                 .long("locked")
                 .help("Require Cargo.lock is up to date"))
        .arg(Arg::with_name("verbose")
                 .short("v")
                 .long("verbose")
                 .multiple(true)
                 .help("Use verbose output (-vv very verbose/build.rs output)"))
        .arg(Arg::with_name("quiet")
                 .short("q")
                 .long("quiet")
                 .conflicts_with("verbose")
                 .help("No output printed to stdout"))
        .arg(Arg::with_name("arg_parser")
                 .long("arg_parser")
                 .short("a")
                 .value_name("PARSER")
                 .default_value("clap")
                 .possible_values(&["clap", "docopt"])
                 .help("Specify the argument parser to use"))
        .arg(Arg::with_name("path").takes_value(true).required(true))
        .get_matches();

    let mut cargo_new_args = Vec::new();
    cargo_new_args.push("new");
    cargo_new_args.push("--bin");

    if matches.is_present("frozen") {
        cargo_new_args.push("--frozen");
    }

    if matches.is_present("locked") {
        cargo_new_args.push("--locked");
    }

    if matches.is_present("quiet") {
        cargo_new_args.push("--quiet");
    }

    match matches.occurrences_of("v") {
        0 => {}
        1 => cargo_new_args.push("-v"),
        2 | _ => cargo_new_args.push("-vv"),
    }

    if let Some(color) = matches.value_of("color") {
        cargo_new_args.push("--color");
        cargo_new_args.push(color);
    }

    if let Some(vcs) = matches.value_of("vcs") {
        cargo_new_args.push("--vcs");
        cargo_new_args.push(vcs);
    }

    let (main_str, run_str, dep_str, error_str) = if let Some(arg_parser) =
        matches.value_of("arg_parser") {
        match arg_parser {
            "clap" => (CLAP_MAIN_RS, CLAP_RUN_RS, CLAP_DEPS, CLAP_ERROR_RS),
            "docopt" => (DOCOPT_MAIN_RS, DOCOPT_RUN_RS, DOCOPT_DEPS, DOCOPT_ERROR_RS),
            _ => return Err(ErrorKind::InvalidArgParser.into()),
        }
    } else {
        return Err(ErrorKind::InvalidArgParser.into());
    };

    let path = if let Some(path) = matches.value_of("path") {
        cargo_new_args.push(path);
        path
    } else {
        return Err(ErrorKind::InvalidPath.into());
    };

    let mut cargo_new = Command::new("cargo").args(&cargo_new_args).spawn()?;
    let ecode = cargo_new.wait()?;

    let mut cargo_toml_path = PathBuf::from(path);
    cargo_toml_path.push("Cargo.toml");
    let cargo_toml = OpenOptions::new()
        .append(true)
        .write(true)
        .open(cargo_toml_path.as_path())?;
    let mut cargo_toml_writer = BufWriter::new(cargo_toml);
    cargo_toml_writer.write_all(dep_str.as_bytes())?;

    let mut main_path = PathBuf::from(path);
    main_path.push("src");
    main_path.push("main.rs");

    let main_rs = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(main_path.as_path())?;
    let mut main_rs_writer = BufWriter::new(main_rs);
    let main_rs_header = format!("//! `{}` 0.1.0\n", path);
    main_rs_writer.write_all(main_rs_header.as_bytes())?;
    main_rs_writer.write_all(main_str.as_bytes())?;

    let mut error_path = PathBuf::from(path);
    error_path.push("src");
    error_path.push("error.rs");

    let error_rs = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(error_path.as_path())?;
    let mut error_rs_writer = BufWriter::new(error_rs);
    let error_rs_header = format!("//! `{}` errors\n", path);
    error_rs_writer.write_all(error_rs_header.as_bytes())?;
    error_rs_writer.write_all(error_str.as_bytes())?;

    let mut run_path = PathBuf::from(path);
    run_path.push("src");
    run_path.push("run.rs");

    let run_rs = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(run_path.as_path())?;
    let mut run_rs_writer = BufWriter::new(run_rs);
    let run_rs_header = format!("//! `{}` runtime\n", path);
    run_rs_writer.write_all(run_rs_header.as_bytes())?;
    run_rs_writer.write_all(run_str.as_bytes())?;

    if let Some(code) = ecode.code() {
        Ok(code)
    } else {
        Ok(-1)
    }
}
