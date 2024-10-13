NB: This Readme is an explanation of the automation process of creating a C-compatible interface of the RUST library for use by other languages.

                
lib.rs and c_api.rs:
Both files contribute to the public API that will be exposed in the C header file. The key is that both files are part of the same crate (library).

lib.rs: Contains the core functionality and exposes get_geojson_string_from_bytes and get_geojson_string functions..
c_api.rs: Contains additional FFI-compatible functions.

cbindgen.toml:
This configuration file plays a crucial role in tying everything together. It specifies which functions should be included in the generated header file.

[export]
include = [
    "get_geojson_string",
    "get_geojson_string_from_bytes",
    "parse_dji_log",
    "get_last_error",
    "c_api_free_string"
]
This tells cbindgen to include these specific functions in the header file, regardless of which Rust file they're defined in.

build.rs:
This script generates the C header file. It uses cbindgen to scan your entire crate (including both lib.rs and c_api.rs) and generate the C bindings based on the configuration in cbindgen.toml.
The key line is:
.with_crate(crate_dir)
This tells cbindgen to look at the entire crate, not just a single file.

Resulting Header File:
The generated dji-log-parser.h will include function declarations for all the functions specified in the cbindgen.toml file, regardless of whether they were defined in lib.rs or c_api.rs.                

The build.rs automates the process of creating a C-compatible interface for the RUST library. It ensures that the header
file always matches the actual implementation in the RUST code, in order to reduce the chance of compile error from manual 
updates.

To use the RUST library ("libdji_log_parser.a") from other languages, include the generated dji-log-parser.h and link against library. 