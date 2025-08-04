FROM rust:latest

WORKDIR /app

#COPY . .
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

ENV TODOIST_API_TOKEN=$TODOIST_API_TOKEN
ENV GITHUB_API_TOKEN=$GITHUB_API_TOKEN
ENV GITHUB_USER_AGENT="victorprouff"
ENV EXECUTOR=$EXECUTOR

RUN cargo build --release

EXPOSE 3030

CMD ["./target/release/run"]