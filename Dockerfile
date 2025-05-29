FROM nvidia/cuda:11.8.0-runtime-ubuntu22.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the project
WORKDIR /app
COPY . .

# Build the project with GPU support
RUN cargo build --release --features gpu

# Expose port
EXPOSE 3001

# Start server
CMD ["./target/release/vanity", "server"]