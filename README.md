# cota-aggregator

[![License](https://img.shields.io/badge/license-MIT-green)](https://github.com/nervina-labs/cota-aggregator/blob/develop/LICENSE)
[![CI](https://github.com/nervina-labs/cota-aggregator/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/nervina-labs/cota-aggregator/actions)

The aggregator service of [CoTA](https://talk.nervos.org/t/rfc-cota-a-compact-token-aggregator-standard-for-extremely-low-cost-nfts-and-fts/6338)

[CoTA Docs](https://cotadev.io)

## Prerequisites

- [CoTA Syncer](https://github.com/nervina-labs/cota-nft-entries-syncer): The server to index CoTA data from CKB

> The aggregator and syncer share the same mysql database, and the aggregator use CoTA data from the database to provide RPC service

- `mysql-client` for macOS: `brew install mysql-client`

If the output is as blow:

```shell
If you need to have mysql-client first in your PATH, run:
  echo 'export PATH="/opt/homebrew/opt/mysql-client/bin:$PATH"' >> ~/.zshrc

For compilers to find mysql-client you may need to set:
  export LDFLAGS="-L/opt/homebrew/opt/mysql-client/lib"
  export CPPFLAGS="-I/opt/homebrew/opt/mysql-client/include"
```

Then put the `RUSTFLAGS='-L/opt/homebrew/opt/mysql-client/lib' ` in front of `cargo build` and `cargo test`

## Quick Start

### Manual

- Rename `.env.example` to `.env`
  - Update the database connection string in `DATABASE_URL` key
  - Update the ckb-indexer url string in `CKB_INDEXER`
  - Update the ckb-node url string in `CKB_NODE`
  - Update the mainnet or testnet in `IS_MAINNET`
- Build with release profile: `make build-release`
- Run with release profile: `make run-release`

### Release

```shell
RUST_LOG=info DATABASE_URL=mysql://root:password@localhost:3306/db_name CKB_NODE=http://localhost:8114 CKB_INDEXER=http://localhost:8116 IS_MAINNET=false ./cota-aggregator
```

### docker

> The RocksDB data of SMT will be saved into `src/store.db`, so the store.db should be mounted into docker. E.g. `-v "$(pwd)":/app/store.db`

```shell
# Build cota-aggregator images from the Dockerfile and run cota-aggregator via docker
docker build -t cota-aggregator .
docker run -d -p 3030:3030 cota-aggregator:latest

# or
docker-compose up -d --build
```

### Public cota aggregator rpc url as blow can be used to develop and test

```
testnet:
https://cota.nervina.dev/aggregator
```

## SDK

[SDK](https://github.com/nervina-labs/cota-sdk-js) can help you implement RPC APIs call and build ckb transactions

## APIs

- [generate_define_cota_smt](#generate_define_cota_smt)
- [generate_mint_cota_smt](#generate_mint_cota_smt)
- [generate_transfer_cota_smt](#generate_transfer_cota_smt)
- [get_hold_cota_nft](#get_hold_cota_nft)
- [get_withdrawal_cota_nft](#get_withdrawal_cota_nft)
- [get_mint_cota_nft](#get_mint_cota_nft)
- [is_claimed](#is_claimed)
- [get_cota_nft_sender](#get_cota_nft_sender)
- [get_define_info](#get_define_info)
- [get_issuer_info](#get_issuer_info)
- [parse_witness](#parse_witness)
- [get_cota_count](#get_cota_count)
- [get_history_transactions](#get_history_transactions)
- [get_transactions_by_block_number](#get_transactions_by_block_number)
- [get_aggregator_info](#get_aggregator_info)

### generate_define_cota_smt

Generate smt data(`smt_entry` for `witness_args.input_type` and `smt_root` for cell data) for CoTA define transaction

#### Parameters

```
lock_script - The definer's lock script
cota_id - CoTA NFT Class Unique ID
total - The total of CoTA NFT Class
issued - The issued count (default to zero)
configure - A bitmap variable to constrain the behavior of the NFT items issued by the NFT Class
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_define_cota_smt",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000e616d1460d634668b8ad81971c3a53e705f51e60",
        "cota_id":"0xea28c98f38b4a57aa81756b167bb37fa42daf67e",
        "total":"0x00000050",
        "issued":"0x00000000",
        "configure":"0x00"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
define_smt_entry - The SMT define information (origin SMT leaves, SMT proof and other information)
smt_root_hash - The latest SMT root hash after defining
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4397583,
        "define_smt_entry":"73000000140000002e0000003b00000042000000010000008100ea28c98f38b4a57aa81756b167bb37fa42daf67e01000000000000500000000000030000004c4f002d0000004372656174652061206e6577204e465420636f6c6c656374696f6e207769746820000000502065646974696f6e",
        "smt_root_hash":"3c3199f83af98669e9e6dbf421702379ae530998441a1e0d3b8a0670ef3c2aba"
    },
    "id":2
}
```

### generate_mint_cota_smt

Generate smt data(`smt_entry` for `witness_args.input_type` and `smt_root` for cell data) for CoTA mint transaction

#### Parameters

```
lock_script - The minter's lock script
cota_id - CoTA NFT Class Unique ID
out_point - The out_point([12..]) of minter's live cell
withdrawals - The information of withdrawers
  token_index - The index of the NFT Class (increment from zero)
  state - Used for indication of current NFT state
  characteristic - A user defined variable to set up the NFT, we could consider it as the DNA of the items
  to_lock_script - The receiver's lock script
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_mint_cota_smt",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000e616d1460d634668b8ad81971c3a53e705f51e60",
        "cota_id":"0xb22585a8053af3fed0fd39127f5b1487ce08b756",
        "out_point":"0x90dceb81ab5b6c0a364b502d6d4febff0ca570c100000000",
        "withdrawals":[
            {
                "token_index":"0x00000000",
                "state":"0x00",
                "characteristic":"0xa505050505050505050505050505050505050505",
                "to_lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801"
            },
            {
                "token_index":"0x00000001",
                "state":"0x00",
                "characteristic":"0xa505050505050505050505050505050505050505",
                "to_lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801"
            }
        ]
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
mint_smt_entry - The SMT mint information (origin SMT leaves, SMT proof and other information)
smt_root_hash - The latest SMT root hash after minting
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4427822,
        "mint_smt_entry":"59030000200000003a00000047000000540000008c000000ae01000055030000010000008100b22585a8053af3fed0fd39127f5b1487ce08b7560100000000000132000000030001000000000001320000000500020000008102b22585a8053af3fed0fd39127f5b1487ce08b756000000008102b22585a8053af3fed0fd39127f5b1487ce08b75600000001220100000c000000970000008b0000001000000026000000730000000000a50505050505050505050505050505050505050549000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d0277480190dceb81ab5b6c0a364b502d6d4febff0ca570c1000000008b0000001000000026000000730000000000a50505050505050505050505050505050505050549000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d0277480190dceb81ab5b6c0a364b502d6d4febff0ca570c100000000a30100004c4f094c4f09484fa3509b1585ebedd80512715cd0cda31767a5899bc0b28f1ec04f971ccdeb4bf753eb4f0150215e3b8a2d27bf9d9cc375dc4e465b41d4a799cfae4dadc65dc08632378babb94f184c4fad50c09bdeb6fe5e6f43d0e84954d8c7f25011fdff303c6c0526017f17423306a71b4f0150976665ff7f197e1c7c01aff439099fb41d294eea384a08fd60fef37d0dd9b8844f184850b692d3cd042b26bc7331e259c05eb9557189c775e4f1023666cfcd5475f8a32c50d3faa2368005192c36452dc1a9ba6826f199a9938f75e578f065e835b394ebce50b35af002a80e3caa9a8b3dbce9c232731e451cc92c9c48cc75638afdf5f2cd1450b2ba915f9b3d18f706d9d4a02a84b155bcdcb46f2430b0cc2edf863e5bc6a6b15041494d9e44cde24fd5d6addc4dc2b11d3480be97aca4df1dec25567ed229bd885057bff03dd7da91f3daaba0e074678107f30d970789f8e79eb615dadd9518572e51019d3151d711c6f867027eef7056c2a45c6ab4d392cd82b373011514a52aa9a27700000000000000000000000000000000000000000000000000000000000000004f3000000000",
        "smt_root_hash":"e80b458c31f3c59ab741e2dbe7fa2857ab04c25304263834f8b193c3589ce12e"
    },
    "id":2
}
```

### generate_transfer_cota_smt

Generate smt data(`smt_entry` for `witness_args.input_type` and `smt_root` for cell data) for CoTA transfer transaction

#### Parameters

```
lock_script - The sender's lock script
withdrawal_lock_script - The withdrawal's lock script of the NFTs
transfer_out_point - The out_point([12..]) of sender's live cell
transfers - The information of transfer
  cota_id - CoTA NFT Class Unique ID
  token_index - The index of the NFT Class (increment from zero)
  to_lock_script - The receiver's lock script
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_transfer_cota_smt",
    "params":{
        "lock_script":"0x49000000100000003000000031000000124a60cd799e1fbca664196de46b3f7f0ecb7138133dcaea4893c51df5b02be60114000000fa15357eb4ad2989f910268db3b3a585a9b51cbe",
        "withdrawal_lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000af4baf7e15c13b9f95ee56166b9c840dd46973b1",
        "transfer_out_point":"0x777347181a25dc39c31ad290b9e2d52ded42295000000000",
        "transfers":[
            {
                "cota_id":"0x4f3b21fc113bfc423f1185ba6c37f16d02c6c71e",
                "token_index":"0x00000000",
                "to_lock_script":"0x49000000100000003000000031000000577a5e5930e2ecdd6200765f3442e6119dc99e87df474f22f13cab819c80b24201140000009cc2405a07d067c98bf6824134b2759b44079629"
            }
        ]
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
smt_root_hash - The latest SMT root hash after transferring
withdraw_block_hash - The block hash containing the withdraw transaction
mint_smt_entry - The SMT transfer information (origin SMT leaves, SMT proof and other information)
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number": 5648377,
        "smt_root_hash": "035dfe06d8aaf28daec16f394b226f1357bfa857b436b506274a32b024b15507",
        "withdraw_block_hash": "0x1e5ee51aee1bcb6ee45400147fb57162fb47941641e66e44b8186752a04cacfe",
        "transfer_smt_entry":"4b03000020000000560000007a000000980000002b01000036010000d10200000100000081034f3b21fc113bfc423f1185ba6c37f16d02c6c71e00000000fb22817c592d8e96982e708d4a6c2135627ee8950000000001000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0100000081024f3b21fc113bfc423f1185ba6c37f16d02c6c71e0000000093000000080000008b000000100000002600000073000000c00000000000000000000000000000000000000000004900000049000000100000003000000031000000577a5e5930e2ecdd6200765f3442e6119dc99e87df474f22f13cab819c80b24201140000009cc2405a07d067c98bf6824134b2759b44079629777347181a25dc39c31ad290b9e2d52ded42295000000000070000004c4fff4c4fff48970100004c4f095109d2beef40bb50b06b1701f6bbae63be2b4123a9a57171b35f794926ef26c9735381000000000000000000000000000000000000000000000000000000000000004fa051a00000e96fe831821c10fe6cfec6e6d334688a5e7e11c6d263e3d900b0702b8664000054c0f76c724f432ff9b0b1f282ed81a17f342301000000000000000000004f0451018e9e5d7cfc6fe1c855d97930ea5737f4aae49897a79765515ff02ed27a53c88900000000000000000000000000000000000000000000000000000000000000004f185118fdc39dedf04b0a51bc3f190a0bab16aa78e36a171a944a799520a4151ed9495900000000000000000000000000000000000000000000000000000000000000005034c98329580d675bc226969e982888bf506dc2fce273b3a9df7d7adb5fc0541d5098969dc267aa28b349b9beacae7edc7ee10faf8d9f68ca13385c1a331be900e451021ae9d0fc52005e139df54ad1b4581f0537028a339558d7d6c639975f3f5b4a8600000000000000000000000000000000000000000000000000000000000000004f34760000005472616e7366657220746865204e4654204f3b21fc113bfc423f1185ba6c37f16d02c6c71e0000000020746f2049000000100000003000000031000000577a5e5930e2ecdd6200765f3442e6119dc99e87df474f22f13cab819c80b24201140000009cc2405a07d067c98bf6824134b2759b44079629"
    },
    "id":2
}
```

### get_hold_cota_nft

Get CoTA NFT information(name, description, image, configure, state etc.) held(not include withdrew) by someone

#### Parameters

```
lock_script - The holder's lock script
page - The page number of the result
page_size - The page size of the result
cota_id - CoTA NFT Class Unique ID (optional)
```

- Without `cota_id` parameter

```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_hold_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "page":"0",
        "page_size":"2"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4875330,
        "nfts":[
            {
                "audio":"",
                "characteristic":"0xa5a5a50505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x780ae7e2eb39ce4985f632567224ccdd1dccfefe",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000001",
                "video":""
            },
            {
                "audio":"",
                "characteristic":"0xa5a5a50505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x2d1ac02ac52548c9b002067e3fed31ad9981b027",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000001",
                "video":""
            }
        ],
        "page_size":2,
        "total":250
    },
    "id":2
}
```

- With `cota_id` parameter

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_hold_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "page":"0",
        "page_size":"2",
        "cota_id": "0x780ae7e2eb39ce4985f632567224ccdd1dccfefe"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030

```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4949226,
        "nfts":[
            {
                "audio":"",
                "characteristic":"0xa5a5a50505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x780ae7e2eb39ce4985f632567224ccdd1dccfefe",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000001",
                "video":""
            }
        ],
        "page_size":2,
        "total":1
    },
    "id":2
}
```

### get_withdrawal_cota_nft

Get CoTA NFT information(name, description, image, configure, state etc.) withdrew(not include held) by someone

#### Parameters

```
lock_script - The withdrawer's lock script
page - The page number of the result
page_size - The page size of the result
cota_id - CoTA NFT Class Unique ID (optional)
```

- Without `cota_id` parameter

```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_withdrawal_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "page":"0",
        "page_size":"3"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4949175,
        "nfts":[
            {
                "audio":"",
                "characteristic":"0x0505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x3766e323d1b70a5536ab2d8dfcfaa03f9b5c4fea",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000001",
                "video":""
            },
            {
                "audio":"",
                "characteristic":"0x0505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x3766e323d1b70a5536ab2d8dfcfaa03f9b5c4fea",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000000",
                "video":""
            },
            {
                "audio":"",
                "characteristic":"0x0505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0xc27aaf7033c51364be0232d1831e33addd90f9ed",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000000",
                "video":""
            }
        ],
        "page_size":3,
        "total":1139
    },
    "id":2
}
```

- With `cota_id` parameter

```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_withdrawal_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "page":"0",
        "page_size":"10",
        "cota_id": "0x3766e323d1b70a5536ab2d8dfcfaa03f9b5c4fea"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030

```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4949165,
        "nfts":[
            {
                "audio":"",
                "characteristic":"0x0505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x3766e323d1b70a5536ab2d8dfcfaa03f9b5c4fea",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000000",
                "video":""
            },
            {
                "audio":"",
                "characteristic":"0x0505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x3766e323d1b70a5536ab2d8dfcfaa03f9b5c4fea",
                "description":"First step to Blockchain mass adoption. NFT platform launch memento.\n\n-- Nervina Labs & Lay2 Tech, 4/30/2021.",
                "image":"https://i.loli.net/2021/04/29/qyJNSE4iHAas7GL.png",
                "meta_characteristic":"",
                "model":"",
                "name":"First Step",
                "properties":"",
                "state":"0x00",
                "token_index":"0x00000001",
                "video":""
            }
        ],
        "page_size":10,
        "total":2
    },
    "id":2
}
```

### get_mint_cota_nft

Get CoTA NFT information(name, description, image, configure, state etc.) minted by issuer

#### Parameters

```
lock_script - The minter's lock script
page - The page number of the result
page_size - The page size of the result
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_mint_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000f86332ab26fe5baa89f7a8f458cffd8de379f255",
        "page":"0",
        "page_size":"2"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4875342,
        "nfts":[
            {
                "audio":"",
                "characteristic":"0x0000000000000000000000000000000000000000",
                "configure":"0x00",
                "cota_id":"0xd3b2bc022b52ce7282b354d97f9e5e5baf6698d7",
                "description":"Test 1 desc",
                "image":"ipfs://bafyreidq5eujpiq5fkygqtmiy7ansuyeujsvpnwieagekmr4y6gllzdsq4/metadata.json",
                "meta_characteristic":"",
                "model":"",
                "name":"Test 1",
                "properties":"",
                "receiver_lock":"0x49000000100000003000000031000000081dbffa88dab54ba426d231ca64eb760cea2fe9e16761a1da400da1b2cbe1280114000000657d0af440ef3804e01f9250b57b8eae1c3cf60d",
                "state":"0x00",
                "token_index":"0x0000000d",
                "video":""
            },
            {
                "audio":"",
                "characteristic":"0x0000000000000000000000000000000000000000",
                "configure":"0x00",
                "cota_id":"0xd3b2bc022b52ce7282b354d97f9e5e5baf6698d7",
                "description":"Test 1 desc",
                "image":"ipfs://bafyreidq5eujpiq5fkygqtmiy7ansuyeujsvpnwieagekmr4y6gllzdsq4/metadata.json",
                "meta_characteristic":"",
                "model":"",
                "name":"Test 1",
                "properties":"",
                "receiver_lock":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000971117321898d14ac238b0f403e109140388b444",
                "state":"0x00",
                "token_index":"0x0000000b",
                "video":""
            }
        ],
        "page_size":2,
        "total":55
    },
    "id":2
}
```

### is_claimed

Check whether an NFT is claimed

#### Parameters

```
lock_script - The checker's lock script
cota_id - CoTA NFT Class Unique ID
token_index - The index of the NFT Class (increment from zero)
```

```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"is_claimed",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce801140000005b600202aa2e99d61b1bbfc3daf2d0cc7b849800",
        "cota_id":"0x2dd97617e685c0cd44b87cba7e8756ea67a721cd",
        "token_index":"0x00000000"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
claimed - true for claimed and false fot unclaimed
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4397583,
        "claimed":true
    },
    "id":2
}
```

### get_cota_nft_sender

Get the sender lock hash of the CoTA NFT

#### Parameters

```
lock_script - The owner's lock script
cota_id - CoTA NFT Class Unique ID
token_index - The index of the NFT Class (increment from zero)
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_cota_nft_sender",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "cota_id":"0xb22585a8053af3fed0fd39127f5b1487ce08b756",
        "token_index":"0x00000000"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
sender_lock_hash - The sender lock hash of the NFT
sender_address - The sender ckb address of the NFT
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4397997,
        "sender_lock_hash":"0x8a8f45a094cbe050d1a612924901b11edc1bce28c0fd8d96cdc8779889f28aa8",
        "sender_address": "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq0xzmg5vrtrge5t3tvpjuwr55l8qh63ucqhqne0u"
    },
    "id":2
}
```

### get_define_info

Get define CoTA NFT Class information(name, description, image, total, issued, configure etc.) by the cota_id

#### Parameters

```
cota_id - CoTA NFT Class Unique ID
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_define_info",
    "params":{
        "cota_id":"0xd3b2bc022b52ce7282b354d97f9e5e5baf6698d7"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "audio":"",
        "block_number":4948295,
        "configure":"0x00",
        "description":"Test 1 desc",
        "image":"ipfs://bafyreidq5eujpiq5fkygqtmiy7ansuyeujsvpnwieagekmr4y6gllzdsq4/metadata.json",
        "issued":26,
        "meta_characteristic":"",
        "model":"",
        "name":"Test 1",
        "properties":"",
        "total":100,
        "video":""
    },
    "id":2
}
```

### get_issuer_info

Get issuer's information

#### Parameters

```
lock_script - The issuer's lock script
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_issuer_info",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000d5f13ab18e9f6b55eae6535b6ec141865437854d"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
avatar - The issuer's avatar
name - The issuer's name
description - The issuer's description
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "avatar":"https://i.loli.net/2021/04/29/IigbpOWP8fw9qDn.png",
        "block_number":4872987,
        "description":"Community building protocol",
        "name":"Rostra"
    },
    "id":2
}
```

### parse_witness

Parse CoTA witness

#### Parameters

```
witness - The CoTA transaction witness
version - The version of CoTA
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"parse_witness",
    "params":{
        "witness":"0x8107000010000000550000008107000041000000f6b9acf8267464b7aaa7729c26fcf4ce1ff16582b4bb40ea122498b8d89de2ef67e8ee44517ce204681f1ed4435d147db2b6ce345731791311ae7246987a67df00280700000227070000200000003a0000004700000054000000bc000000a6010000230700000100000081001e23dc506c1b15f286c9db84a4d12a453266097501000000000000640000006200010000000000006400000064000200000081021e23dc506c1b15f286c9db84a4d12a453266097500000062dc6fb71f2caea8949e8a7a88f53ed031589ed65d0000000081021e23dc506c1b15f286c9db84a4d12a453266097500000063dc6fb71f2caea8949e8a7a88f53ed031589ed65d00000000ea0000000c0000007b0000006f0000000c000000220000000000050505050505050505050505050505050505050549000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d027748016f0000000c000000220000000000050505050505050505050505050505050505050549000000490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801790500004c4fa95091a4c55d156aa9299b3094733123a9ccbdf20a4ca4ea5414e0212bf6202c2dfc50edfd0724f0e61e58169515abe0ad9d58622b6c11cb8e6538333190672f6de23c51ab643619549f2d589d230daf6feebcce7385fc4efbee4499523133294ceaa7dbf68100e477dc008e244d26dc87a97f6bda123292ff0a0000000000000000000000504ba8433d54e0b8940b1f58a7ae1f58c0b360eccac36a67f6fdc5738c5ce3e3eb5011238f883e0519d2627e7f8ae405ade0ad769ab84afe70520f9c7d62d26cf842500874023b2277b101808e0faeb2a65864049e781265be50ff1d060c596d00042350fb4076fdd2e1218bddcfa8cd5e932412565814bc3544ca867c5268373ea68f294f185118f7e7780fdee42f2bcff16c1cf50c159440ce34ad34984105b0d0a08a490b28da000000000000000000000000000000000000000000000000000000000000000051c9373c9f4e89ce45af216c2ca83a19be7f9963ee92a60930f793e25f9f24c6aeb28101b066e0f068aa8be6548063a18d811c489a9e2141000000000000000000004f2951f32bb62c965b80ae2351946062f857a05740d1007cfdfcd79571395bc095d1bc8b6fc899e5a5fce9b5a62297bbffa6803530bac3715640a8f2728ff4c171e2060051f486a090928af903f712a376fd1d45c2b0bad9b3f31fcd3137816f08656fdd554a95d27744b861422abdfafeb0e40ba8c793fab8dad90cf7ab5bdc9772194d090050e73032d84ca4d732a8c3d7239c9ffc2413c8abbba6f66e03575d13ffa9bc145251f6fc114bc93ecc14ce0d584ae36703ce6fc71774cdf1755ce6cab6ce67b7801f32ab98df4af1a44b6a88b5849550431abba0ab2cfe4b3cb93bb0230f02108e3d0051f722628b8d83acb29ed79f1929328f5f2d36513215426edc610a613d23f0035139a9180f58b189c5d5cdf1114384cca860bdd49a1c88b67a670800aed1869d100050cf4bcb3dddb4f33d22dd690c5ef46f1e1819ae866b1b38df2de14d8064bc2a88500cc298e2f2db864c99ba8ba85f1cb94e6d0516dc88cfc71969fa4aed1e7e5a21501a5eb3e7d7687ee2450a884147091cda21c59c780b22c22a6426a71ccbf372b550a58d80132aa023b05c079dd276c7335a5f5512bd50cea98fd1c1f6ba788eb13750de3d22b1a7725911d0eb142dae4947b1ebd68c220471fa0c7eab6d70d60674e550e54e5228a6f740816a3538f72281bee3a5ba681f92688b4832c977574c183372501d496836b0d38dc322857e736adf869f5b57b423ec78c7089bd642f95f6998634c4ff7501d47608687cba32d95e1519b28374c681e3fc30ef6ec88174fdade1f4bb268715039e1b23de083aa42de52ad9d891a3a2c141fbe2d821411e00d16488bcbb2cb33501dc1f382523fc46c6f9f6c0e3a77c96fb473d57b4bac4037f38683c363bd8ec750e476d6204d69099795d0e7300074949377000027bb3837423917d3d5b4ea906c50ab9b35f4aad5594a87d7f68193de993fcb62e46ff985464dd251b2936b2264cb4c4ff65068a05a45b4faf89a9402b24c04b3a46fc2e18a93c12da126e11d3c8f036cd11251f712c1e6b1a0d27af3d1a72acbc92105b08626248ef932c94260accb9502afef22442d4e0cf6ab7f36e0a9834a9f338d198065a66d8d2412603188b5278b865b0050e737a5fade47517f44b6a8d5f720609a1053bd9e1428a76d96ec1e4fa3d3cacb50a766c1da702421881cd1085a83e67ca17a3dca02badecb69738f78ced905fd2950f9e79576dc135d92368a94edcd45b4cfb9dc4e6abc20411defa216c332dd486350a8dfe6c905c799e319f0f8fcdbe6f9e4bb7c57bbe85f848c6167b3a2cca345184850cf2365392df4f78464069bbcaddc3fbd5849b7d946a97a82d718c3d2416b43d850bd380a2a20fccede15c4a38e827ecb81c35d8f282ed73f4492c0d79e39c6f6564800000000",
        "version": "1"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "cota":{
            "action":"0x436c61696d2000000001204e465473",
            "claim_keys":[
                {
                    "nft_id":{
                        "cota_id":"0xb066e0f068aa8be6548063a18d811c489a9e2141",
                        "index":"0x00000002",
                        "smt_type":"0x8103"
                    },
                    "out_point":"0x801bf2a1cf9bc19a1f76c4bde975453aeeda3bb900000000"
                }
            ],
            "claim_values":[
                "0x01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            ],
            "hold_keys":[
                {
                    "cota_id":"0xb066e0f068aa8be6548063a18d811c489a9e2141",
                    "index":"0x00000002",
                    "smt_type":"0x8101"
                }
            ],
            "hold_values":[
                {
                    "characteristic":"0x0505050505050505050505050505050505050505",
                    "configure":"0x00",
                    "state":"0x00"
                }
            ],
            "proof":"0x4c4fc9504e5f5f264e833e52e56fd637174835e8b62f9a41d072492952ee131d1f4bbc834f2951f32bb62c965b80ae2351946062f857a05740d1007cfdfcd79571395bc095d1bc8b6fc899e5a5fce9b5a62297bbffa6803530bac3715640a8f2728ff4c171e2060051f486a090928af903f712a376fd1d45c2b0bad9b3f31fcd3137816f08656fdd554a95d27744b861422abdfafeb0e40ba8c793fab8dad90cf7ab5bdc9772194d090051f5be1e78619b3184b2e72ac894c8c893ce7f5bb2cfa180718a5f86f417196d6a8e4e7a1e6a96c56a484e4099dca95e94198228cf321b5c94701ded37d98061070051f6fc114bc93ecc14ce0d584ae36703ce6fc71774cdf1755ce6cab6ce67b7801f32ab98df4af1a44b6a88b5849550431abba0ab2cfe4b3cb93bb0230f02108e3d0051f722628b8d83acb29ed79f1929328f5f2d36513215426edc610a613d23f0035139a9180f58b189c5d5cdf1114384cca860bdd49a1c88b67a670800aed1869d100050cf4bcb3dddb4f33d22dd690c5ef46f1e1819ae866b1b38df2de14d8064bc2a8850930b1b1c41f2cab355a98d95ec67733b79cfaca6ed2a7f6199b7ca2641dd071e5080f6302e040b1b7588d87b042979152880b9f19abc2a8c92dfb940163887937e50a0180d13681e7cf98e171c4a231d9421cd2e20c126157f7f975735c82698fed3503a6d882662760922d8764d2d270649fff1f64e14384380572cacb27d87b756ae4c4ff451f464c871152f423765d8fdca48a8fd6e4c4a5de5eda3ea1570555f639a91fd1ce50bf6d6bc86dcbf88464318a4f39703e3a3ffce795a187a872d86195695d50b0051f5eaa2418203426b2571d7f423d5c290d30f4a44faa996a77a9dccb67801277376c46611c8e54c66a11cdac1cb7f735707d06f336eb55ca4230710f4741899010051f6952df4f5ea998a65a78075ec6385017cded66102d4bd6ec2cec9c2e17702eb194eb2c14e639ddfec5621092c8955b2ef7702a674b2e7d39769317f4df58b1a004f015076afaaee13543ffe626bcc3ab57598fd6b6f3e6a3f694cb0dda65ba1b4476b6450038eaea85d217df5a19e7dbcba6b8f3a8739ce74ff34b90ee2225818984ac2d750a372b2f4bda98ff5f58bc3d77367169449e904a88f657a84100906d32ef60a6e50998ad34f7f56a2ae88187b2a5918b7f65520cbaa503192ccdb1bacb142d387c250f374ac20ead7871639afd63d9f18c6d232be2bd42014892d0052c50725b2e6a5485065114104f9b9a683b37c32193dd1fc71f8ed982319ed5b866d8a2bcd7f136b0d50e5ea3b4845fcc36263096c811589749cae3b5eff33b01e1f6e8948fa73a50920",
            "type":"claim"
        },
        "info":null
    },
    "id":2
}
```

### get_cota_count

Get the count of NFTs held and withdrew by the owner

#### Parameters

```
lock_script - The owner's lock script
cota_id - CoTA NFT Class Unique ID
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_cota_count",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
        "cota_id":"0x1e23dc506c1b15f286c9db84a4d12a4532660975"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
block_number - The latest block number of cota-nft-entries-syncer
count - The count of NFTs held and withdrew by the owner
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":5120925,
        "count":80
    },
    "id":2
}
```

### get_history_transactions

Get the history transactions of the specific CoTA NFT with `cota_id` and `token_index`

#### Parameters

```
cota_id - CoTA NFT Class Unique ID
token_index - The index of the NFT Class (increment from zero)
page - The page number of the result
page_size - The page size of the result
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_history_transactions",
    "params":{
        "cota_id":"0x16166f5eb4ec88ad89710bbd596ac8987052a85e",
	    "token_index": "0x00000000",
	    "page": "0",
	    "page_size": "10"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

### Response

```
block_number - The latest block number of cota-nft-entries-syncer
page_size - page_size - The page size of the CoTA NFT transaction list
total - The total amount of the CoTA NFT transaction list
transactions - The transaction list of the sepcific CoTA NFT
    age - The block timestamp of the mint or transfer transaction (millisecond)
    block_number - The CoTA NFT transfer or mint block number
    from - The sender address of the CoTA NFT transfer or mint transaction
    to - The receiver address of the CoTA NFT transfer or mint transaction
    tx_hash - The hash of the CoTA NFT transfer or mint transaction
    type - The type of the CoTA NFT transaction: 'transfer' or 'mint'
```

```shell
 "jsonrpc":"2.0",
    "result":{
        "block_number":7587159,
        "page_size":10,
        "total":6,
        "transactions":[
            {
                "age":1650608618595,
                "block_number":6960258,
                "from":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "to":"ckb1qrgp752jcfnm0uemj723grpyva6zappyuj0tuge3etkpjlmjsxmq5qt7jq0g09af5m0yqjncd5v9mg3p2yuu5es2w57jj",
                "tx_hash":"0x325a2f20531ab5020aa1a95ccff633140d7c3e67ec0011a01892c177d6fb572b",
                "type":"transfer"
            },
            {
                "age":1647613397194,
                "block_number":6727018,
                "from":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqvuj4m34cfxmz74jlsk0dkp6yrwf7q2z4q9puchc",
                "to":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "tx_hash":"0xa1edd805418bca0cd999df49d042eb46139b8b09a964fd8f11e0de6c8c57f467",
                "type":"transfer"
            },
            {
                "age":1647613281847,
                "block_number":6727011,
                "from":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "to":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqvuj4m34cfxmz74jlsk0dkp6yrwf7q2z4q9puchc",
                "tx_hash":"0x9c547d7a5b38e3e46062d48bf4887bb0ca98ea593bf805882b41bf1738f8ac12",
                "type":"transfer"
            },
            {
                "age":1647613127780,
                "block_number":6726999,
                "from":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqvuj4m34cfxmz74jlsk0dkp6yrwf7q2z4q9puchc",
                "to":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "tx_hash":"0xa5a63fd171029e1424a5d155206977680cea40d9b0fdf0d6040855d18f8e02bb",
                "type":"transfer"
            },
            {
                "age":1647612280580,
                "block_number":6726919,
                "from":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "to":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqvuj4m34cfxmz74jlsk0dkp6yrwf7q2z4q9puchc",
                "tx_hash":"0xe0ed4ccddf35d1a36ce6979fa4c1b64af9dd8fe248ec25346951e0a3d0fa09c6",
                "type":"transfer"
            },
            {
                "age":1645804795836,
                "block_number":6585724,
                "from":"ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqgk2yu24zgcj0dq2vlsshmhg4hqprlm23c4jwrps",
                "to":"ckb1qqypm0l63rdt2jayymfrrjnyadmqe630a8skwcdpmfqqmgdje0sjsqt3emxu792v2yatrwfgp48huss2yg4yq9g083utk",
                "tx_hash":"0x3fc95d95a6ecaafab402ac2dc1200c37018052f61fa1bd02ecf81a36f7b6b142",
                "type":"mint"
            }
        ]
    },
    "id":2
}
```

### get_transactions_by_block_number

Get the CoTA transactions of the specific block number

#### Parameters

```
block_number - The block number of CKB
```

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_transactions_by_block_number",
    "params":{
        "block_number": "6586282"
    }
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

### Response

```
block_number - The latest block number of cota-nft-entries-syncer
transactions - The transaction list of the sepcific CoTA NFT
    cota_id - CoTA NFT Class Unique ID
    token_index - The index of the NFT Class (increment from zero)
    block_number - The block number of the mint or transfer transaction
    from - The sender address of the CoTA NFT transfer or mint transaction
    to - The receiver address of the CoTA NFT transfer or mint transaction
    tx_hash - The hash of the CoTA NFT transfer or mint transaction
    type - The type of the CoTA NFT transaction: 'transfer' or 'mint'
```

```
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":7893309,
        "transactions":[
            {
                "block_number":6586282,
                "cota_id":"0x2b738f9f6c2326f6ef198d7a0dff19636ee00ed4",
                "from":"ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqdy2wj0q6wxpda8ft6xnxgq83zu6t4cvvcrlhnz7",
                "to":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqg78jg823ckd4k2h3vme6yhgzyvaxs5jfqlcnjj2",
                "token_index":"0x00000006",
                "tx_hash":"0x52a49c3e4d9344c429b5d55c8c1616b6e8aa9a5783f4a5c5588b1f92e07aa73c",
                "type":"mint"
            },
            {
                "block_number":6586282,
                "cota_id":"0x2b738f9f6c2326f6ef198d7a0dff19636ee00ed4",
                "from":"ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqdy2wj0q6wxpda8ft6xnxgq83zu6t4cvvcrlhnz7",
                "to":"ckb1qzl58smqy32hnrq6vxjedcxe2fugvnz497h7yvwqvwel40uh4rltcqg78jg823ckd4k2h3vme6yhgzyvaxs5jfqlcnjj2",
                "token_index":"0x00000007",
                "tx_hash":"0x52a49c3e4d9344c429b5d55c8c1616b6e8aa9a5783f4a5c5588b1f92e07aa73c",
                "type":"mint"
            }
        ]
    },
    "id":2
}
```

### get_aggregator_info

Get the cota-aggregator information

#### Parameters

None

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_aggregator_info"
}' \
| tr -d '\n' \
| curl -H 'content-type: application/json' -d @- \
http://127.0.0.1:3030
```

#### Response

```
indexer_block_number - The latest block number of ckb-indexer
node_block_number - The latest block number of ckb-node
syncer_block_number - The latest block number of cota-nft-entries-syncer
version - The current version of cota-aggregator
is_mainnet - The environment variable to indicate ckb network
```

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "indexer_block_number":7561075,
        "node_block_number":7561075,
        "syncer_block_number":7561075,
        "version":"v0.6.5",
        "is_mainnet": true
    },
    "id":2
}
```
