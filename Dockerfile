# Use the official Rust image as the base image
FROM rust:1.57 AS build

# Create a new working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create an empty src directory (this helps with caching dependencies)
RUN mkdir src && touch src/main.rs

# Build the dependencies
RUN cargo build --release

# Copy the source code
COPY . .

# Build the project
RUN cargo build --release

# Start a new stage with a smaller base image for the final image
FROM debian:bullseye-slim

# Set the working directory in the final image
WORKDIR /app

# Copy the compiled binary from the build image to the final image
COPY --from=build /app/target/release/your_app_name /app

# Expose any necessary ports (if applicable)
# EXPOSE 8080

# Define the command to run your Rust application
CMD ["./dredge"]
