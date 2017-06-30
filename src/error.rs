// Copyright (c) 2017 cargo-cli developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `cargo-cli` errors
error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Rustache(::rustache::Error);
        FromUtf8(::std::string::FromUtf8Error);
        TomlDe(::toml::de::Error);
        TomlSe(::toml::ser::Error);
    }

    errors {
        InvalidArgParser {
            description("An invalid argument parser was specified!")
            display("An invalid argument parser was specified!")
        }
        InvalidExitCode {
            description("An invalid exit code was received from 'cargo new'!")
            display("An invalid exit code was received from 'cargo new'!")
        }
        InvalidLicense {
            description("An invalid license type was specified!")
            display("An invalid license type was specified!")
        }
        InvalidPath {
            description("An invalid path was specified!")
            display("An invalid path was specified!")
        }
        InvalidSubCommand {
            description("An invalid subcommand was specified!")
            display("An invalid subcommand was specified!")
        }
    }
}
