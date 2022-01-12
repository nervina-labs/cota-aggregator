FROM rust:1.56
WORKDIR /usr/src/cota-aggregator
COPY . .
COPY ./debian/config /usr/local/cargo
RUN CARGO_HTTP_MULTIPLEXING=false cargo fetch
RUN cargo install --path .
ENV RUST_LOG info
EXPOSE 3030
CMD ["cota-aggregator"]