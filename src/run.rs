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
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use tmpl::Templates;
use toml;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    package: Package,
    dependencies: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Package {
    name: String,
    version: String,
    authors: Vec<String>,
    license: Option<String>,
    readme: Option<String>,
}

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
                 .help("Specify the argument parser to use in the generated output."))
            .arg(Arg::with_name("license")
                 .long("license")
                 .value_name("TYPE")
                 .help("Specify licensing to include in the generated output.")
                 .possible_values(&["both", "mit", "apache", "none"])
                 .default_value("both")
                 .takes_value(true))
            .arg(Arg::with_name("no-readme")
                 .long("no-readme")
                 .help("Turn off README.md generation."))
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
                "clap" => Templates::new(name, true, mit, apache, readme),
                "docopt" => Templates::new(name, false, mit, apache, readme),
                _ => return Err(ErrorKind::InvalidArgParser.into()),
            }
        } else {
            return Err(ErrorKind::InvalidArgParser.into());
        };


        let mut cargo_new = Command::new("cargo").args(&cargo_new_args).spawn()?;
        let ecode = cargo_new.wait()?;

        let mut main_path = PathBuf::from(path);
        main_path.push("src");
        main_path.push("main.rs");

        let main_rs = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(main_path.as_path())?;
        let mut main_rs_writer = BufWriter::new(main_rs);
        if template.has_license() {
            main_rs_writer.write_all(template.prefix()?.as_bytes())?;
        }
        main_rs_writer.write_all(template.main()?.as_bytes())?;

        let mut error_path = PathBuf::from(path);
        error_path.push("src");
        error_path.push("error.rs");

        let error_rs = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(error_path.as_path())?;
        let mut error_rs_writer = BufWriter::new(error_rs);
        if template.has_license() {
            error_rs_writer.write_all(template.prefix()?.as_bytes())?;
        }
        error_rs_writer.write_all(template.error()?.as_bytes())?;

        let mut run_path = PathBuf::from(path);
        run_path.push("src");
        run_path.push("run.rs");

        let run_rs = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(run_path.as_path())?;
        let mut run_rs_writer = BufWriter::new(run_rs);
        if template.has_license() {
            run_rs_writer.write_all(template.prefix()?.as_bytes())?;
        }
        run_rs_writer.write_all(template.run()?.as_bytes())?;

        if let Some(mit) = template.mit() {
            let mut mit_path = PathBuf::from(path);
            mit_path.push("LICENSE-MIT");

            let mit_license = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(mit_path.as_path())?;
            let mut mit_license_writer = BufWriter::new(mit_license);
            mit_license_writer.write_all(mit.as_bytes())?;
        }

        if let Some(apache) = template.apache() {
            let mut apache_path = PathBuf::from(path);
            apache_path.push("LICENSE-APACHE");

            let apache_license = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(apache_path.as_path())?;
            let mut apache_license_writer = BufWriter::new(apache_license);
            apache_license_writer.write_all(apache.as_bytes())?;
        }

        if let Some(Ok(readme)) = template.readme() {
            let mut readme_path = PathBuf::from(path);
            readme_path.push("README.md");

            let readme_license = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(readme_path.as_path())?;
            let mut readme_license_writer = BufWriter::new(readme_license);
            readme_license_writer.write_all(readme.as_bytes())?;
        }

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
            HashMap::new()
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
        cargo_toml_writer
            .write_all(toml::to_string(&config)?.as_bytes())?;

        if let Some(code) = ecode.code() {
            Ok(code)
        } else {
            Ok(-1)
        }
    } else {
        Err(ErrorKind::InvalidSubCommand.into())
    }
}
