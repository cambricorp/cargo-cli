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
    }

    errors {
        InvalidArgParser {
            description("An invalid argument parser was specified!")
            display("An invalid argument parser was specified!")
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
