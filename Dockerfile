FROM rust:1.81

# Set the working directory in the container
WORKDIR /app

# Copy only the necessary files for building
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=3001

# Expose the port
EXPOSE 3001

# Run the binary
CMD ["./target/release/chat-api"]