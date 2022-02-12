FROM rust:1.58 as builder

WORKDIR /app

COPY . .
COPY debian/config .cargo/config.toml

RUN apt-get update
RUN apt-get install cmake clang llvm gcc -y

RUN cargo build --release

FROM alpine:latest
COPY --from=builder /app/target/release/cota-aggregator /app/cota-aggregator
RUN chmod +x /app/cota-aggregator

WORKDIR /app

ENV RUST_LOG info
ENV DATABASE_URL mysql://root:password@localhost:3306/db_name
ENV MAX_POOL 20

EXPOSE 3030

CMD ["./cota-aggregator"]
