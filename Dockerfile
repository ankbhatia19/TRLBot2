# Use the official Rust image as a base
FROM rust:1.82 AS builder

# Set the working directory
WORKDIR /usr/src/bot

# Copy the manifest and source files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the bot in release mode
RUN cargo build --release

# Use a slim runtime image for the final build
FROM debian:bookworm-slim

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder
COPY --from=builder /usr/src/bot/target/release/TRLBot2 /app/TRLBot2

# Set the entrypoint
ENTRYPOINT ["/app/TRLBot2"]
