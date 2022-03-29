FROM rust:1.58 as builder

WORKDIR /app

COPY . .
COPY debian/config .cargo/config.toml

# use aliyun source
RUN sed -i "s@http://deb.debian.org@http://mirrors.aliyun.com@g" /etc/apt/sources.list
RUN apt-get clean

RUN apt-get update
RUN apt-get install cmake clang llvm gcc -y

RUN cargo build --release

FROM debian:bookworm-20211011-slim

# use aliyun source
RUN sed -i "s@http://deb.debian.org@http://mirrors.aliyun.com@g" /etc/apt/sources.list
RUN apt-get clean

RUN apt-get update && apt-get install -y libmariadb-dev

COPY --from=builder /app/target/release/cota-aggregator /app/cota-aggregator
RUN chmod +x /app/cota-aggregator

WORKDIR /app

ENV RUST_LOG info
ENV DATABASE_URL mysql://root:password@localhost:3306/db_name
ENV MAX_POOL 20
ENV THREADS 3
ENV IS_MAINNET false

EXPOSE 3030

CMD ["./cota-aggregator"]
