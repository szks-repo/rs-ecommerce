FROM rust:1.88-bookworm

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

COPY crates ./crates
RUN cargo fetch

CMD ["cargo", "run", "-p", "rs-ecommerce"]
