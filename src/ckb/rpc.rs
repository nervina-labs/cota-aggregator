use crate::utils::error::Error;
use ckb_jsonrpc_types::{TransactionProof as JSONRPCTxProof, TransactionView, Uint64};
use ckb_sdk::CkbRpcClient;
use ckb_types::packed::{BytesVec, Transaction};
use cota_smt::common::{
    Byte32, Byte32VecBuilder, Bytes, MerkleProofBuilder, TransactionProof, TransactionProofBuilder,
    Uint32, Uint32VecBuilder,
};
use cota_smt::smt::H256;
use molecule::prelude::{Builder, Entity};
use std::env;

#[derive(Clone, Debug, Default)]
pub struct WithdrawRawTx {
    pub raw_tx:     Bytes,
    pub tx_proof:   TransactionProof,
    pub block_hash: H256,
    pub witnesses:  BytesVec,
}

pub fn get_withdraw_info(
    block_number: u64,
    out_point_slice: [u8; 24],
) -> Result<WithdrawRawTx, Error> {
    let ckb_node_url =
        env::var("CKB_NODE").map_err(|_e| Error::Other("CKB_NODE must be set".to_owned()))?;
    let mut client = CkbRpcClient::new(&ckb_node_url);
    let block = client
        .get_block_by_number(Uint64::from(block_number))
        .map_err(|_e| Error::CKBRPCError("get_block_by_number".to_string()))?
        .ok_or(Error::CKBRPCError("get_block error".to_string()))?;
    let block_hash = block.header.hash;
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

    let transaction_proof = client
        .get_transaction_proof(vec![tx_view.hash], Some(block_hash.clone()))
        .map_err(|_e| Error::CKBRPCError("get_transaction_proof".to_string()))?;
    let tx_proof = get_tx_proof(transaction_proof);

    let withdraw_info = WithdrawRawTx {
        block_hash: H256::from(block_hash.0),
        raw_tx,
        tx_proof,
        witnesses: tx.witnesses(),
    };

    Ok(withdraw_info)
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
