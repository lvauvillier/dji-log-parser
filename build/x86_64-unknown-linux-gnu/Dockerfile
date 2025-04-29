FROM ubuntu:latest

# Install necessary packages and tools
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    g++ \
    gcc-x86-64-linux-gnu \
    g++-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    curl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust and Cargo to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install the target for x86_64-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-gnu

# Set up cargo config for cross-compilation
RUN mkdir -p /root/.cargo && \
    echo '[target.x86_64-unknown-linux-gnu]' >> /root/.cargo/config && \
    echo 'linker = "x86_64-linux-gnu-gcc"' >> /root/.cargo/config

# Set environment variables for cross-compilation
ENV CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
ENV CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++

# Set the working directory
WORKDIR /usr/src/myapp

# Copy the entire project
COPY . .

# Build the project
RUN cargo build --release --target x86_64-unknown-linux-gnu

# This command will be run when you use 'docker run'
CMD ["/bin/bash"]