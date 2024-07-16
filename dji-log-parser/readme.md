NB: This Readme is an explanation of the automation process of creating a C-compatible interface of the RUST library for use by other languages.

The build.rs file works in conjunction with the c_api.rs file to create the dji-log-parser.h file. 
    . build.rs: is a special RUST file that runs before the main compilation. It's purpose is to generate the C header file automatically.
                : it determines the crate directory and package name.
                : it sets up the output path for the header file. 
                : it uses the cbindgen library to generate C bindings from the RUST code.

The c_api.rs file contains the RUST functions that will be exposed to C.
                : functions marked with #[no_mangle] and pub extern "C" are made available to C.
                : these function use C-compatible types (like c_char instead of RUST's str).


How it works:
    . when the code is run with "cargo build", the build.rs script is executed first.
    . it looks for functions marked with #[no_mangle] and pub extern "C".
    . for each of these functions, it generates a corresponding C function declaration in the header file.

Resulting dji-log-parser.h file.
                ; this file is the interface that other languages use to interact with the RUST library.
                : it declares the functions that are implemented in RUST but callable from C.


The build.rs automates the process of creating a C-compatible interface for the RUST library. It ensures that the header
file always matches the actual implementation in the RUST code, in order to reduce the chance of compile error from manual 
updates.

To use the RUST library ("libdji_log_parser.a") from other languages, include the generated dji-log-parser.h and link against library. 