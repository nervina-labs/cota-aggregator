use crate::ckb::constants::{MAINNET_COTA_CODE_HASH, TESTNET_COTA_CODE_HASH};
use crate::utils::error::Error;
use crate::utils::helper::parse_bytes_n;
use ckb_jsonrpc_types::{
    BlockNumber, Script as RPCScript, TransactionProof as JSONRPCTxProof, TransactionView, Uint64,
};
use ckb_sdk::CkbRpcClient;
use ckb_types::packed::{BytesVec, Script, Transaction};
use cota_smt::common::{
    Byte32, Byte32VecBuilder, Bytes, BytesBuilder, MerkleProofBuilder, TransactionProof,
    TransactionProofBuilder, Uint32, Uint32VecBuilder,
};
use cota_smt::smt::{blake2b_256, H256};
use molecule::prelude::{Builder, Byte, Entity};
use serde_json::from_str;
use std::env;

#[derive(Clone, Debug, Default)]
pub struct WithdrawRawTx {
    pub raw_tx:       Bytes,
    pub output_index: Uint32,
    pub tx_proof:     TransactionProof,
    pub block_hash:   H256,
    pub block_number: u64,
    pub witnesses:    BytesVec,
}

pub async fn get_withdraw_info(
    block_number: u64,
    withdrawal_lock_hash: [u8; 32],
    withdrawal_tx_hash: [u8; 32],
) -> Result<WithdrawRawTx, Error> {
    let is_mainnet: bool = match env::var("IS_MAINNET") {
        Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
        Err(_e) => false,
    };
    let cota_code_hash = if is_mainnet {
        parse_bytes_n::<32>(MAINNET_COTA_CODE_HASH.to_owned()).unwrap()
    } else {
        parse_bytes_n::<32>(TESTNET_COTA_CODE_HASH.to_owned()).unwrap()
    };

    tokio::task::spawn_blocking(move || {
        let mut client = ckb_node_client()?;
        let block = client
            .get_block_by_number(Uint64::from(block_number))
            .map_err(|_e| Error::CKBRPCError("get_block_by_number".to_string()))?
            .ok_or(Error::CKBRPCError("get_block error".to_string()))?;
        let block_hash = block.header.hash;
        let block_number = block.header.inner.number.value();
        let mut output_index = Uint32::default();
        let txs: Vec<TransactionView> = block
            .transactions
            .into_iter()
            .filter(|tx| {
                let position = tx.inner.outputs.clone().into_iter().position(|output| {
                    let lock: Script = output.lock.clone().into();
                    let lock_ret: bool = blake2b_256(lock.as_slice()) == withdrawal_lock_hash;
                    let type_ret: bool = output.type_.clone().map_or(false, |type_: RPCScript| {
                        type_.code_hash.as_bytes() == &cota_code_hash
                    });
                    lock_ret && type_ret
                });
                if position.is_some() {
                    output_index =
                        Uint32::from_slice(&(position.unwrap() as u32).to_be_bytes()).unwrap();
                }
                tx.hash.as_bytes() == &withdrawal_tx_hash
            })
            .collect();
        if txs.is_empty() {
            return Err(Error::CKBRPCError(format!(
                "The tx dose not exist in the block#{:?}",
                block_number
            )));
        }
        let tx_view = txs.get(0).cloned().unwrap();

        let tx: Transaction = tx_view.inner.into();
        let raw_tx: Bytes = BytesBuilder::default()
            .set(
                tx.raw()
                    .as_slice()
                    .into_iter()
                    .map(|v| Byte::from(*v))
                    .collect(),
            )
            .build();

        let transaction_proof = client
            .get_transaction_proof(vec![tx_view.hash], Some(block_hash.clone()))
            .map_err(|_e| Error::CKBRPCError("get_transaction_proof".to_string()))?;
        let tx_proof = get_tx_proof(transaction_proof);

        let withdraw_info = WithdrawRawTx {
            block_hash: H256::from(block_hash.0),
            block_number,
            raw_tx,
            tx_proof,
            output_index,
            witnesses: tx.witnesses(),
        };

        Ok(withdraw_info)
    })
    .await
    .unwrap()
}

pub async fn get_node_tip_block_number() -> Result<u64, Error> {
    tokio::task::spawn_blocking(move || {
        let mut client = ckb_node_client()?;
        let block_number = client
            .get_tip_block_number()
            .map_err(|_e| Error::CKBRPCError("get_tip_block_number".to_string()))?;
        Ok(u64::from(block_number))
    })
    .await
    .unwrap()
}

pub async fn get_block_timestamp(block_number: u64) -> Result<u64, Error> {
    tokio::task::spawn_blocking(move || {
        let mut client = ckb_node_client()?;
        let header = client
            .get_header_by_number(BlockNumber::from(block_number))
            .map_err(|_e| Error::CKBRPCError("get_header_by_number".to_string()))?
            .ok_or(Error::CKBRPCError("get_header_by_number".to_string()))?;
        Ok(header.inner.timestamp.value())
    })
    .await
    .unwrap()
}

fn get_tx_proof(transaction_proof: JSONRPCTxProof) -> TransactionProof {
    let indices = Uint32VecBuilder::default()
        .set(
            transaction_proof
                .proof
                .indices
                .into_iter()
                .map(|index| Uint32::from_slice(&index.value().to_be_bytes()).unwrap())
                .collect(),
        )
        .build();
    let lemmas = Byte32VecBuilder::default()
        .set(
            transaction_proof
                .proof
                .lemmas
                .into_iter()
                .map(|lemma| Byte32::from_slice(lemma.as_bytes()).unwrap())
                .collect(),
        )
        .build();
    TransactionProofBuilder::default()
        .witnesses_root(Byte32::from_slice(transaction_proof.witnesses_root.as_bytes()).unwrap())
        .proof(
            MerkleProofBuilder::default()
                .indices(indices)
                .lemmas(lemmas)
                .build(),
        )
        .build()
}

fn ckb_node_client() -> Result<CkbRpcClient, Error> {
    let ckb_node_url =
        env::var("CKB_NODE").map_err(|_e| Error::Other("CKB_NODE must be set".to_owned()))?;
    Ok(CkbRpcClient::new(&ckb_node_url))
}
