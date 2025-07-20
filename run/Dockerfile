# Use the latest version of the Rust base image
FROM rust:latest
# Set the working directory in the container to /my
WORKDIR /src/main
# Copy the Rust project files to the working directory
COPY . .
# Build the Rust app
RUN cargo build
# Set the command to run the Rust app
ENV TODOIST_API_TOKEN=$TODOIST_API_TOKEN
ENV GITHUB_API_TOKEN=$GITHUB_API_TOKEN
ENV GITHUB_USER_AGENT="victorprouff"

CMD cargo run
