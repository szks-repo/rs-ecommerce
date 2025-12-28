FROM rust:1.78-bullseye

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
RUN cargo fetch

COPY src ./src

CMD ["cargo", "run"]
