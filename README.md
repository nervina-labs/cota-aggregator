# cota-aggregator

The aggregator of [CoTA](https://talk.nervos.org/t/rfc-cota-a-compact-token-aggregator-standard-for-extremely-low-cost-nfts-and-fts/6338) service

## Quick Start

### Manual

- Rename `.env.example` to `.env` and update the database connection string in `DATABASE_URL` key.
- Build with release profile: `make build-release`
- Run with release profile: `make run-release`

### docker

```shell
# Build cota-aggregator images from the Dockerfile and run cota-aggregator via docker
docker build -t cota-aggregator .
docker run -d -p 3030:3030 cota-aggregator:latest

# or
docker-compose up -d --build
```

### APIs

- generate_define_cota_smt

```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_define_cota_smt",
    "params":{
        "lock_hash":"0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1",
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

- generate_mint_cota_smt
```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_mint_cota_smt",
    "params":{
        "lock_hash":"0x8a8f45a094cbe050d1a612924901b11edc1bce28c0fd8d96cdc8779889f28aa8",
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

- generate_transfer_cota_smt
```shell
echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"generate_transfer_cota_smt",
    "params":{
        "lock_script":"0x49000000100000003000000031000000124a60cd799e1fbca664196de46b3f7f0ecb7138133dcaea4893c51df5b02be60114000000fa15357eb4ad2989f910268db3b3a585a9b51cbe",
        "withdrawal_lock_hash":"0xc84947784cce65bdd259948630a5e77ebcfce205fd53a55dc333afe98007bd19",
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

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4427822,
        "smt_root_hash":"035dfe06d8aaf28daec16f394b226f1357bfa857b436b506274a32b024b15507",
        "transfer_smt_entry":"4b03000020000000560000007a000000980000002b01000036010000d10200000100000081034f3b21fc113bfc423f1185ba6c37f16d02c6c71e00000000fb22817c592d8e96982e708d4a6c2135627ee8950000000001000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0100000081024f3b21fc113bfc423f1185ba6c37f16d02c6c71e0000000093000000080000008b000000100000002600000073000000c00000000000000000000000000000000000000000004900000049000000100000003000000031000000577a5e5930e2ecdd6200765f3442e6119dc99e87df474f22f13cab819c80b24201140000009cc2405a07d067c98bf6824134b2759b44079629777347181a25dc39c31ad290b9e2d52ded42295000000000070000004c4fff4c4fff48970100004c4f095109d2beef40bb50b06b1701f6bbae63be2b4123a9a57171b35f794926ef26c9735381000000000000000000000000000000000000000000000000000000000000004fa051a00000e96fe831821c10fe6cfec6e6d334688a5e7e11c6d263e3d900b0702b8664000054c0f76c724f432ff9b0b1f282ed81a17f342301000000000000000000004f0451018e9e5d7cfc6fe1c855d97930ea5737f4aae49897a79765515ff02ed27a53c88900000000000000000000000000000000000000000000000000000000000000004f185118fdc39dedf04b0a51bc3f190a0bab16aa78e36a171a944a799520a4151ed9495900000000000000000000000000000000000000000000000000000000000000005034c98329580d675bc226969e982888bf506dc2fce273b3a9df7d7adb5fc0541d5098969dc267aa28b349b9beacae7edc7ee10faf8d9f68ca13385c1a331be900e451021ae9d0fc52005e139df54ad1b4581f0537028a339558d7d6c639975f3f5b4a8600000000000000000000000000000000000000000000000000000000000000004f34760000005472616e7366657220746865204e4654204f3b21fc113bfc423f1185ba6c37f16d02c6c71e0000000020746f2049000000100000003000000031000000577a5e5930e2ecdd6200765f3442e6119dc99e87df474f22f13cab819c80b24201140000009cc2405a07d067c98bf6824134b2759b44079629"
    },
    "id":2
}
```

- get_hold_cota_nft
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
        "block_number":4427822,
        "nfts":[
            {
                "characteristic":"0x2525250505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0xb22585a8053af3fed0fd39127f5b1487ce08b756",
                "state":"0x00",
                "token_index":"0x00000002"
            },
            {
                "characteristic":"0xa505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0xc7801a1d8ff707d2076b85de002160cf92ec7b65",
                "state":"0x00",
                "token_index":"0x00000050"
            }
        ],
        "page_size":2,
        "total":241
    },
    "id":2
}
```

- get_withdrawal_cota_nft
```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_withdrawal_cota_nft",
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
        "block_number":4427822,
        "nfts":[
            {
                "characteristic":"0xa505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x1deb31f603652bf59ff5027b522e1d81c288b72f",
                "state":"0x00",
                "token_index":"0x00000000"
            },
            {
                "characteristic":"0xa505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0x1deb31f603652bf59ff5027b522e1d81c288b72f",
                "state":"0x00",
                "token_index":"0x00000001"
            }
        ],
        "page_size":2,
        "total":616
    },
    "id":2
}
```

- get_mint_cota_nft
```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"get_mint_cota_nft",
    "params":{
        "lock_script":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000e616d1460d634668b8ad81971c3a53e705f51e60",
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
        "block_number":4427822,
        "nfts":[
            {
                "characteristic":"0xa505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0xb22585a8053af3fed0fd39127f5b1487ce08b756",
                "receiver_lock":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
                "state":"0x00",
                "token_index":"0x00000000"
            },
            {
                "characteristic":"0xa505050505050505050505050505050505050505",
                "configure":"0x00",
                "cota_id":"0xb22585a8053af3fed0fd39127f5b1487ce08b756",
                "receiver_lock":"0x490000001000000030000000310000009bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce80114000000dc70f33de86fdf381b4fc5bf092bb23d02774801",
                "state":"0x00",
                "token_index":"0x00000001"
            }
        ],
        "page_size":2,
        "total":616
    },
    "id":2
}
```

- is_claimed

```shell
 echo '{
    "id":2,
    "jsonrpc":"2.0",
    "method":"is_claimed",
    "params":{
        "lock_hash":"0x3162711f5048d416c62c4ee5483a9c289dbe607fb00790b14ad7dc7edf1c21d9",
        "cota_id":"0x2dd97617e685c0cd44b87cba7e8756ea67a721cd",
        "token_index":"0x00000000"
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
        "block_number":4397583,
        "claimed":true
    },
    "id":2
}
```

- get_cota_nft_sender

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

```shell
{
    "jsonrpc":"2.0",
    "result":{
        "block_number":4397997,
        "sender_lock_hash":"0x8a8f45a094cbe050d1a612924901b11edc1bce28c0fd8d96cdc8779889f28aa8"
    },
    "id":2
}
```