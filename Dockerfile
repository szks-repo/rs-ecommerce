FROM rust:1.88-bookworm

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

COPY src ./src
COPY build.rs ./build.rs
RUN cargo fetch

CMD ["cargo", "run"]
