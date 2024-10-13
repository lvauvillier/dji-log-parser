1: dji-log-parser/dji-log-parser/Cargo.toml (shared for all architectures with target-specific sections):

        [package]
        name = "dji-log-parser"
        version.workspace = true
        authors.workspace = true
        edition = "2021"
        description = "Library for parsing DJI txt logs"
        repository.workspace = true
        license.workspace = true
        keywords.workspace = true
        categories.workspace = true
        rust-version = "1.56"

        [dependencies]
        aes.workspace = true
        base64.workspace = true
        binrw.workspace = true
        cbc.workspace = true
        chrono = { workspace = true, features = ["serde"] }
        crc64.workspace = true
        serde = { workspace = true, features = ["derive"] }
        serde_json.workspace = true
        thiserror.workspace = true
        ureq = { workspace = true, features = ["json"] }
        log = "0.4"
        geojson.workspace = true
        once_cell = "1.8.0"
        libc = "0.2"

        [lib]
        name = "dji_log_parser"
        crate-type = ["staticlib", "rlib"]

        [build-dependencies]
        cbindgen = "0.24.0"
        cc = "1.0"

        [target.'cfg(target_os = "linux")'.dependencies]
        openssl = { version = "0.10", features = ["vendored"] }

        [target.'cfg(target_os = "macos")'.build-dependencies]
        cc = "1.0"

2: build.rs (shared for all architectures):

        use std::env;
        use std::path::PathBuf;

        fn main() {
            if cfg!(target_os = "linux") {
                println!("cargo:rustc-link-lib=m");
            }
            let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
            let package_name = env::var("CARGO_PKG_NAME").unwrap();
            let output_file = PathBuf::from(&crate_dir)
                .join("include")
                .join(format!("{}.h", package_name));

            let config = cbindgen::Config::from_file("cbindgen.toml")
                .expect("Unable to find cbindgen.toml configuration file");

            cbindgen::Builder::new()
                .with_crate(crate_dir)
                .with_config(config)
                .generate()
                .expect("Unable to generate bindings")
                .write_to_file(output_file);
        }

3: Dockerfiles for each architecture:

         a: aarch64-apple-darwin (M1/M2 Mac):

                    FROM --platform=linux/arm64 rust:latest

                    RUN rustup target add aarch64-apple-darwin

                    WORKDIR /usr/src/myapp
                    COPY . .

                    CMD ["cargo", "build", "--release", "--target", "aarch64-apple-darwin"]


         b: x86_64-apple-darwin (Intel Mac):

                    FROM --platform=linux/amd64 rust:latest

                    RUN rustup target add x86_64-apple-darwin

                    WORKDIR /usr/src/myapp
                    COPY . .

                    CMD ["cargo", "build", "--release", "--target", "x86_64-apple-darwin"]



        c. aarch64-unknown-linux-gnu:

                    FROM --platform=linux/arm64 rust:latest

                    RUN apt-get update && apt-get install -y \
                        gcc-aarch64-linux-gnu \
                        g++-aarch64-linux-gnu \
                        libc6-dev-arm64-cross

                    RUN rustup target add aarch64-unknown-linux-gnu

                    ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
                    ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
                    ENV CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

                    WORKDIR /usr/src/myapp
                    COPY . .

                    CMD ["cargo", "build", "--release", "--target", "aarch64-unknown-linux-gnu"]


        d. x86_64-unknown-linux-gnu:

                    FROM --platform=linux/amd64 rust:latest

                    RUN rustup target add x86_64-unknown-linux-gnu

                    WORKDIR /usr/src/myapp
                    COPY . .

                    CMD ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-gnu"]



These configurations allow you to build the Rust library for all four target architectures. The Cargo.toml and build.rs files are shared across all targets, with conditional compilation and linking handled within them. The Dockerfiles are specific to each target architecture, setting up the appropriate environment and build commands.
Please note: building for macOS targets (aarch64-apple-darwin and x86_64-apple-darwin) from a Linux-based Docker container might not be possible due to macOS-specific dependencies. In practice, you typically build these on a macOS host directly.






Cross compilation tools were needed to be installed on my macOS, this is an explanation of what was installed and why:

The brew install commands I needed are related to setting up cross-compilation tools on macOS. These tools allow us to compile code for different target architectures from my macOS development environment. Here's a breakdown of the key installations and their purposes:

brew install x86_64-elf-gcc
    Purpose: This installs the GNU Compiler Collection (GCC) toolchain for x86_64 ELF (Executable and Linkable Format) targets.
    Use: It's used for cross-compiling to 64-bit x86 Linux systems from macOS.

brew install FiloSottile/musl-cross/musl-cross
    Purpose: This installs the musl-cross toolchain, which allows cross-compilation to Linux using the musl libc instead of glibc.
    Use: It's used for creating statically linked Linux binaries, which can run on virtually any Linux system without dependency issues.

brew install FiloSottile/musl-cross/x86_64-linux-musl-gcc
    Purpose: This installs a specific GCC toolchain for compiling to x86_64 Linux with musl libc.
    Use: It's used for creating statically linked Linux binaries for x86_64 architecture.

brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu
    Purpose: This installs a cross-compilation toolchain for the x86_64-unknown-linux-gnu target.
    Use: It allows compiling Rust code for 64-bit Linux systems from macOS.

These installations were necessary because:

Cross-compilation: 
    I needed to build the Rust library for different target architectures (x86_64 and aarch64) and operating systems (macOS and Linux) from a single macOS development environment.

Static linking: 
    For the Linux targets, I wanted to create statically linked libraries to avoid dependency issues on the target systems.

Rust and Go integration: 
    The project Rust library is intended to be called from Go code, which requires proper cross-compilation setup to ensure the Rust library can be linked correctly in different environments.

CI/CD compatibility: 
    These tools allow us to build for various targets locally, mimicking what might happen in a CI/CD pipeline, ensuring the build process works consistently across different environments.

By installing these tools, we set up a comprehensive cross-compilation environment on macOS, allowing us to build the project for multiple target architectures and operating systems, which was crucial for solving the linking issues in Go, specifically for its use in the Flight-Management repo and ensuring the Rust project could be built and run on various platforms.

For further questions contact Edan Cain.