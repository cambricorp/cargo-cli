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
    }
}
