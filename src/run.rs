// Copyright (c) 2017 cargo-cli developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `cargo-cli` runtime.
use clap::{App, AppSettings, Arg, SubCommand};
use error::{ErrorKind, Result};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use tmpl::Templates;

/// Parse the args, and execute the generated commands.
pub fn run() -> Result<i32> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Creates a Rust command line application")
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("cli")
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
            .arg(Arg::with_name("path").takes_value(true).required(true)))
        .get_matches();

    if let Some(cli_matches) = matches.subcommand_matches("cli") {
        let mut cargo_new_args = Vec::new();
        cargo_new_args.push("new");
        cargo_new_args.push("--bin");

        if cli_matches.is_present("frozen") {
            cargo_new_args.push("--frozen");
        }

        if cli_matches.is_present("locked") {
            cargo_new_args.push("--locked");
        }

        if cli_matches.is_present("quiet") {
            cargo_new_args.push("--quiet");
        }

        match cli_matches.occurrences_of("v") {
            0 => {}
            1 => cargo_new_args.push("-v"),
            2 | _ => cargo_new_args.push("-vv"),
        }

        if let Some(color) = cli_matches.value_of("color") {
            cargo_new_args.push("--color");
            cargo_new_args.push(color);
        }

        if let Some(vcs) = cli_matches.value_of("vcs") {
            cargo_new_args.push("--vcs");
            cargo_new_args.push(vcs);
        }

        let template = if let Some(arg_parser) = cli_matches.value_of("arg_parser") {
            match arg_parser {
                "clap" => Templates::new(true, false, false),
                "docopt" => Templates::new(false, false, false),
                _ => return Err(ErrorKind::InvalidArgParser.into()),
            }
        } else {
            return Err(ErrorKind::InvalidArgParser.into());
        };

        let path = if let Some(path) = cli_matches.value_of("path") {
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
        cargo_toml_writer.write_all(template.deps().as_bytes())?;

        let mut main_path = PathBuf::from(path);
        main_path.push("src");
        main_path.push("main.rs");

        let main_rs = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(main_path.as_path())?;
        let mut main_rs_writer = BufWriter::new(main_rs);
        let main_rs_header = format!("//! `{}` 0.1.0\n", path);
        if template.has_license() {
            main_rs_writer.write_all(template.prefix().as_bytes())?;
        }
        main_rs_writer.write_all(main_rs_header.as_bytes())?;
        main_rs_writer.write_all(template.main().as_bytes())?;

        let mut error_path = PathBuf::from(path);
        error_path.push("src");
        error_path.push("error.rs");

        let error_rs = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(error_path.as_path())?;
        let mut error_rs_writer = BufWriter::new(error_rs);
        let error_rs_header = format!("//! `{}` errors\n", path);
        if template.has_license() {
            error_rs_writer.write_all(template.prefix().as_bytes())?;
        }
        error_rs_writer.write_all(error_rs_header.as_bytes())?;
        error_rs_writer.write_all(template.error().as_bytes())?;

        let mut run_path = PathBuf::from(path);
        run_path.push("src");
        run_path.push("run.rs");

        let run_rs = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(run_path.as_path())?;
        let mut run_rs_writer = BufWriter::new(run_rs);
        let run_rs_header = format!("//! `{}` runtime\n", path);
        if template.has_license() {
            run_rs_writer.write_all(template.prefix().as_bytes())?;
        }
        run_rs_writer.write_all(run_rs_header.as_bytes())?;
        run_rs_writer.write_all(template.run().as_bytes())?;

        if let Some(code) = ecode.code() {
            Ok(code)
        } else {
            Ok(-1)
        }
    } else {
        Err(ErrorKind::InvalidSubCommand.into())
    }
}
