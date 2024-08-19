# Use the official Rust base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /relayer

# Copy your Rust project files into the container
COPY . .

# Build your program for release
RUN cargo build --release

# Run the binary (adjust the binary name accordingly)
ENTRYPOINT ["./target/release/relayer"]
