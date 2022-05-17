use crate::utils::error::Error;
use ckb_jsonrpc_types::{TransactionView, Uint64};
use ckb_sdk::CkbRpcClient;
use ckb_types::packed::Transaction;
use cota_smt::common::{
    Byte32, Byte32VecBuilder, Bytes, MerkleProofBuilder, OutPointSlice, TransactionProof,
    TransactionProofBuilder, Uint32, Uint32VecBuilder,
};
use molecule::prelude::{Builder, Entity};
use std::env;

pub fn get_raw_tx_and_proof(
    block_number: u64,
    out_point_slice: OutPointSlice,
) -> Result<(Bytes, TransactionProof), Error> {
    let ckb_node_url =
        env::var("CKB_NODE").map_err(|_e| Error::Other("CKB_NODE must be set".to_owned()))?;
    let mut client = CkbRpcClient::new(&ckb_node_url);
    let block = client
        .get_block_by_number(Uint64::from(block_number))
        .map_err(|_e| Error::CKBRPCError("get_block_by_number".to_string()))?
        .ok_or(Error::CKBRPCError("get_block error".to_string()))?;
    let txs: Vec<TransactionView> = block
        .transactions
        .into_iter()
        .filter(|tx| tx.hash.as_bytes()[12..] == out_point_slice.as_slice()[0..20])
        .collect();
    if txs.is_empty() {
        return Err(Error::CKBRPCError(format!(
            "The tx dose not exist in the block#{:?}",
            block_number
        )));
    }
    let tx_view = txs.get(0).cloned().unwrap();
    let tx: Transaction = tx_view.inner.into();
    let raw_tx: Bytes = Bytes::from_slice(tx.raw().as_slice()).unwrap();

    let tx_proof = client
        .get_transaction_proof(vec![tx_view.hash], Some(block.header.hash))
        .map_err(|_e| Error::CKBRPCError("get_transaction_proof".to_string()))?;

    let indices = Uint32VecBuilder::default()
        .set(
            tx_proof
                .proof
                .indices
                .into_iter()
                .map(|index| Uint32::from_slice(&index.value().to_be_bytes()).unwrap())
                .collect(),
        )
        .build();
    let lemmas = Byte32VecBuilder::default()
        .set(
            tx_proof
                .proof
                .lemmas
                .into_iter()
                .map(|lemma| Byte32::from_slice(lemma.as_bytes()).unwrap())
                .collect(),
        )
        .build();
    let transaction_proof = TransactionProofBuilder::default()
        .witnesses_root(Byte32::from_slice(tx_proof.witnesses_root.as_bytes()).unwrap())
        .proof(
            MerkleProofBuilder::default()
                .indices(indices)
                .lemmas(lemmas)
                .build(),
        )
        .build();
    Ok((raw_tx, transaction_proof))
}
