FROM clux/muslrust:stable as builder

WORKDIR /app

COPY . .
COPY debian/config .cargo/config.toml

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cota-aggregator /app/cota-aggregator
RUN chmod +x /app/cota-aggregator

WORKDIR /app

ENV RUST_LOG info
ENV DATABASE_URL mysql://root:password@localhost:3306/db_name
ENV MAX_POOL 20

EXPOSE 3030

CMD ["./cota-aggregator"]
