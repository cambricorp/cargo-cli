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
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use term;
use tmpl::{TemplateType, Templates};
use toml;

/// A partial representation of the Cargo.toml config.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    /// The package configuration section.
    package: Package,
    /// The dependencies list.
    dependencies: Option<BTreeMap<String, String>>,
}

/// A partial representation of the Cargo.toml package config.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Package {
    /// The package name.
    name: String,
    /// The package version.
    version: String,
    /// The list of authors.
    authors: Vec<String>,
    /// The licenses.
    license: Option<String>,
    /// The readme file.
    readme: Option<String>,
}

/// output level
#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum Level {
    /// Output everything
    Trace = 0,
    /// Everything but TRACE
    Debug = 1,
    /// Everything but TRACE and DEBUG
    Info = 2,
    /// Warn and above only.
    Warn = 3,
    // /// Error and above only.
    // Error = 4,
    // /// Only fatal messages are output.
    // Fatal = 5,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Level::Trace => write!(f, "Trace"),
            Level::Debug => write!(f, "Debug"),
            Level::Info => write!(f, "Info"),
            Level::Warn => write!(f, "Warn"),
        }
    }
}

/// Write to the given file based on the template type.
fn write_file(
    file: File,
    template: &Templates,
    template_type: &TemplateType,
    level: &Level,
) -> Result<()> {
    let mut file_writer = BufWriter::new(file);


    match *template_type {
        TemplateType::Main => {
            if template.has_license() {
                file_writer.write_all(template.prefix()?.as_bytes())?;
            }
            file_writer.write_all(template.main()?.as_bytes())?;
            debug("Updated", "src/main.rs", level)?;
        }
        TemplateType::Error => {
            if template.has_license() {
                file_writer.write_all(template.prefix()?.as_bytes())?;
            }
            file_writer.write_all(template.error()?.as_bytes())?;
            debug("Updated", "src/error.rs", level)?;
        }
        TemplateType::Run => {
            if template.has_license() {
                file_writer.write_all(template.prefix()?.as_bytes())?;
            }
            file_writer.write_all(template.run()?.as_bytes())?;
            debug("Updated", "src/run.rs", level)?;
        }
        TemplateType::Mit => if let Some(mit) = template.mit() {
            file_writer.write_all(mit.as_bytes())?;
            debug("Created", "LICENSE-MIT", level)?;
        },
        TemplateType::Apache => if let Some(apache) = template.apache() {
            file_writer.write_all(apache.as_bytes())?;
            debug("Created", "LICENSE-APACHE", level)?;
        },
        TemplateType::Readme => if let Some(Ok(readme)) = template.readme() {
            file_writer.write_all(readme.as_bytes())?;
            debug("Created", "README.md", level)?;
        },
    }

    Ok(())
}

/// Update a pre-existing file based on the given template.
fn update_file(
    path: &str,
    path_parts: &[&str],
    template: &Templates,
    template_type: &TemplateType,
    level: &Level,
) -> Result<()> {
    let mut file_path = PathBuf::from(path);
    for path_part in path_parts {
        file_path.push(path_part);
    }

    let file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .open(file_path.as_path())?;

    write_file(file, template, template_type, level)
}

/// Create a new file based on the given template.
fn create_file(
    path: &str,
    path_parts: &[&str],
    template: &Templates,
    template_type: &TemplateType,
    level: &Level,
) -> Result<()> {
    let mut file_path = PathBuf::from(path);
    for path_part in path_parts {
        file_path.push(path_part);
    }

    let create_file = match *template_type {
        TemplateType::Mit => template.mit().is_some(),
        TemplateType::Apache => template.apache().is_some(),
        TemplateType::Readme => template.readme().is_some(),
        _ => true,
    };

    if create_file {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(file_path.as_path())?;

        write_file(file, template, template_type, level)?;
    }

    Ok(())
}

/// Log a `cargo` formatted message to the terminal.
fn log_message(verb: &str, message: &str) -> Result<()> {
    let mut t = term::stdout().ok_or(ErrorKind::TermCommand)?;
    t.fg(term::color::BRIGHT_GREEN)?;
    t.attr(term::Attr::Bold)?;
    write!(t, "{:>12}", verb)?;
    t.reset()?;
    writeln!(t, " {}", message)?;
    t.flush()?;
    Ok(())
}

/// Log a debug level message to the terminal.
fn debug(verb: &str, message: &str, level: &Level) -> Result<()> {
    if *level <= Level::Debug {
        log_message(verb, message)?;
    }
    Ok(())
}

/// Log an info level message to the terminal.
fn info(verb: &str, message: &str, level: &Level) -> Result<()> {
    if *level <= Level::Info {
        log_message(verb, message)?;
    }
    Ok(())
}

/// Parse the args, and execute the generated commands.
pub fn run() -> Result<i32> {
    let matches =
        App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about("Creates a Rust command line application")
            .setting(AppSettings::GlobalVersion)
            .setting(AppSettings::VersionlessSubcommands)
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                SubCommand::with_name("cli")
                    .arg(
                        Arg::with_name("vcs")
                            .long("vcs")
                            .value_name("VCS")
                            .help(
                                "Initialize a new repository for the given version control system
                        or do not initialize any version control at all, overriding a
                        global configuration.",
                            )
                            .possible_values(&["git", "hg", "pijul", "fossil", "none"])
                            .default_value("git")
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("name")
                            .long("name")
                            .value_name("NAME")
                            .help(
                                "Set the resulting package name, defaults to the value of <path>.",
                            )
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("color")
                            .long("color")
                            .value_name("WHEN")
                            .help("Coloring")
                            .possible_values(&["auto", "always", "never"])
                            .default_value("auto")
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("frozen")
                            .long("frozen")
                            .conflicts_with("locked")
                            .help("Require Cargo.lock and cache are up to date"),
                    )
                    .arg(
                        Arg::with_name("locked")
                            .long("locked")
                            .help("Require Cargo.lock is up to date"),
                    )
                    .arg(
                        Arg::with_name("verbose")
                            .short("v")
                            .multiple(true)
                            .help("Use verbose output (-vv very verbose/build.rs output)"),
                    )
                    .arg(
                        Arg::with_name("quiet")
                            .short("q")
                            .long("quiet")
                            .conflicts_with("verbose")
                            .help("No output printed to stdout"),
                    )
                    .arg(
                        Arg::with_name("arg_parser")
                            .long("arg_parser")
                            .short("a")
                            .value_name("PARSER")
                            .default_value("clap")
                            .possible_values(&["clap", "docopt"])
                            .help("Specify the argument parser to use in the generated output."),
                    )
                    .arg(
                        Arg::with_name("license")
                            .long("license")
                            .value_name("TYPE")
                            .help("Specify licensing to include in the generated output.")
                            .possible_values(&["both", "mit", "apache", "none"])
                            .default_value("both")
                            .takes_value(true),
                    )
                    .arg(
                        Arg::with_name("no-readme")
                            .long("no-readme")
                            .help("Turn off README.md generation."),
                    )
                    .arg(
                        Arg::with_name("no-latest").long("no-latest").help(
                            "Turn off the crates.io query for the latest version (use defaults).",
                        ),
                    )
                    .arg(Arg::with_name("path").takes_value(true).required(true)),
            )
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

        let level = if cli_matches.is_present("quiet") {
            cargo_new_args.push("--quiet");
            Level::Warn
        } else {
            match cli_matches.occurrences_of("verbose") {
                0 => Level::Info,
                1 => {
                    cargo_new_args.push("-v");
                    Level::Debug
                }
                2 | _ => {
                    cargo_new_args.push("-vv");
                    Level::Trace
                }
            }
        };

        if let Some(color) = cli_matches.value_of("color") {
            cargo_new_args.push("--color");
            cargo_new_args.push(color);
        }

        if let Some(vcs) = cli_matches.value_of("vcs") {
            cargo_new_args.push("--vcs");
            cargo_new_args.push(vcs);
        }

        let path = if let Some(path) = cli_matches.value_of("path") {
            path
        } else {
            return Err(ErrorKind::InvalidPath.into());
        };

        let name = if let Some(name) = cli_matches.value_of("name") {
            cargo_new_args.push("--name");
            cargo_new_args.push(name);
            cargo_new_args.push(path);
            name
        } else {
            cargo_new_args.push(path);
            path
        };

        let readme = !cli_matches.is_present("no-readme");
        let query = !cli_matches.is_present("no-latest");

        let (mit, apache) = if let Some(license) = cli_matches.value_of("license") {
            match license {
                "both" => (true, true),
                "mit" => (true, false),
                "apache" => (false, true),
                "none" => (false, false),
                _ => return Err(ErrorKind::InvalidLicense.into()),
            }
        } else {
            return Err(ErrorKind::InvalidLicense.into());
        };

        let template = if let Some(arg_parser) = cli_matches.value_of("arg_parser") {
            match arg_parser {
                "clap" => Templates::new(name, true, mit, apache, readme, query),
                "docopt" => Templates::new(name, false, mit, apache, readme, query),
                _ => return Err(ErrorKind::InvalidArgParser.into()),
            }
        } else {
            return Err(ErrorKind::InvalidArgParser.into());
        };

        let mut cargo_new = Command::new("cargo")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .args(&cargo_new_args)
            .spawn()?;
        let ecode = cargo_new.wait()?;

        if !ecode.success() {
            if let Some(code) = ecode.code() {
                return Ok(code);
            } else {
                return Err(ErrorKind::InvalidExitCode.into());
            }
        }

        update_file(
            path,
            &["src", "main.rs"],
            &template,
            &TemplateType::Main,
            &level,
        )?;
        create_file(
            path,
            &["src", "error.rs"],
            &template,
            &TemplateType::Error,
            &level,
        )?;
        create_file(
            path,
            &["src", "run.rs"],
            &template,
            &TemplateType::Run,
            &level,
        )?;
        create_file(
            path,
            &["LICENSE-MIT"],
            &template,
            &TemplateType::Mit,
            &level,
        )?;
        create_file(
            path,
            &["LICENSE-APACHE"],
            &template,
            &TemplateType::Apache,
            &level,
        )?;
        create_file(
            path,
            &["README.md"],
            &template,
            &TemplateType::Readme,
            &level,
        )?;

        let mut cargo_toml_path = PathBuf::from(path);
        cargo_toml_path.push("Cargo.toml");
        let mut cargo_toml_str = String::new();
        let cargo_toml = File::open(cargo_toml_path.as_path())?;
        let mut cargo_toml_reader = BufReader::new(cargo_toml);
        cargo_toml_reader.read_to_string(&mut cargo_toml_str)?;

        let mut config: Config = toml::from_str(&cargo_toml_str)?;
        let mut pkg = config.package.clone();
        let mut deps = if let Some(deps) = config.dependencies {
            deps
        } else {
            BTreeMap::new()
        };

        template.add_deps(&mut deps);

        if readme {
            pkg.readme = Some(template.cargo_toml_readme().to_string());
        }

        if mit && apache {
            pkg.license = Some(template.cargo_toml_both().to_string());
        } else if mit {
            pkg.license = Some(template.cargo_toml_mit().to_string());
        } else if apache {
            pkg.license = Some(template.cargo_toml_apache().to_string());
        }

        config.package = pkg;
        config.dependencies = Some(deps);

        let new_cargo_toml = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(cargo_toml_path.as_path())?;
        let mut cargo_toml_writer = BufWriter::new(new_cargo_toml);
        cargo_toml_writer.write_all(toml::to_string(&config)?.as_bytes())?;

        debug("Updated", "Cargo.toml", &level)?;

        let msg = format!("binary cli (application) `{}` project", name);
        info("Created", &msg, &level)?;

        Ok(0)
    } else {
        Err(ErrorKind::InvalidSubCommand.into())
    }
}
