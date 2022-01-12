# cota-aggregator

The aggregator of [CoTA](https://github.com/nervina-labs/ckb-cota-scripts) service

### Quick Start

Update `database_url` in `aggregator.toml` with your mysql url

```shell
cargo build

cargo run

cargo test
```

### Usage

```shell
cargo build --release
RUST_LOG=info ./target/release/cota-aggregator

# or
cargo install --path .
RUST_LOG=info cota-aggregator
```

```shell
echo '{
    "id": 2,
    "jsonrpc": "2.0",
    "method": "generate_define_cota_smt",
    "params": {"lock_hash": "0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1", "cota_id" : "0xea28c98f38b4a57aa81756b167bb37fa42daf67e", "total": "0x00000050", "issued": "0x00000000", "configure": "0x00"}
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://localhost:3030
```