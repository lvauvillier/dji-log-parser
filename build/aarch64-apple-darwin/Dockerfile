FROM --platform=linux/arm64 rust:latest

RUN rustup target add aarch64-apple-darwin

WORKDIR /usr/src/myapp

COPY . .

RUN cargo build --release --target aarch64-apple-darwin

CMD ["/bin/bash"]
