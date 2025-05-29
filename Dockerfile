# Use the official Rust image as a builder
FROM rust:latest as builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage with a newer base image
FROM ubuntu:22.04

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/vanity /usr/local/bin/vanity

# Expose the port the app runs on
EXPOSE 3001

# Run the binary with the server command
CMD ["vanity", "server"]