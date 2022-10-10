use crate::entries::helper::get_value_padding_block_height;
use crate::utils::error::Error;
use ckb_types::packed::{BytesVec, WitnessArgs};
use cota_smt::common::{
    Bytes, BytesBuilder, ClaimCotaNFTKeyVec, CotaNFTId, DefineCotaNFTKeyVec, DefineCotaNFTValueVec,
    HoldCotaNFTKeyVec, WithdrawalCotaNFTKeyV1Vec, WithdrawalCotaNFTKeyVec,
    WithdrawalCotaNFTValueV1Vec, WithdrawalCotaNFTValueVec, *,
};
use cota_smt::mint::{MintCotaNFTEntries, MintCotaNFTV1Entries};
use cota_smt::molecule::prelude::*;
use cota_smt::smt::{blake2b_256, Blake2bHasher};
use cota_smt::transfer::{
    TransferCotaNFTEntries, TransferCotaNFTV1Entries, TransferCotaNFTV2Entries,
    WithdrawalCotaNFTEntries, WithdrawalCotaNFTV1Entries,
};
use cota_smt::transfer_update::{
    TransferUpdateCotaNFTEntries, TransferUpdateCotaNFTV1Entries, TransferUpdateCotaNFTV2Entries,
};
use molecule::prelude::Entity;
use sparse_merkle_tree::{CompiledMerkleProof, H256};

const MINT: u8 = 2;
const WITHDRAW: u8 = 3;
const TRANSFER: u8 = 6;
const TRANSFER_UPDATE: u8 = 8;

type Pairs<'a> = &'a [([u8; 20], [u8; 4])];
type Leaves = Vec<(H256, H256, bool)>;

pub fn parse_witness_withdraw_proof(
    witnesses: BytesVec,
    pairs: Pairs,
    block_number: u64,
) -> Result<Bytes, Error> {
    for index in 0..witnesses.len() {
        let witness = witnesses.get(index);
        if witness.is_none() {
            continue;
        }
        if let Ok(witness_args) = WitnessArgs::from_slice(&witness.unwrap().raw_data().to_vec()) {
            let input_type_opt = witness_args.input_type();
            if input_type_opt.is_none() {
                continue;
            }
            let input_type = input_type_opt.to_opt().unwrap().raw_data();
            match u8::from(input_type[0]) {
                MINT => {
                    if let Ok(entries_v0) = MintCotaNFTEntries::from_slice(&input_type[1..]) {
                        let withdrawal_keys = entries_v0.withdrawal_keys();
                        let count = withdrawal_keys
                            .into_iter()
                            .filter(|key| match_cota_id_index(key, pairs))
                            .count();
                        if count == pairs.len() {
                            let leaves = parse_define(
                                block_number,
                                entries_v0.define_keys(),
                                entries_v0.define_new_values(),
                            );
                            return parse_withdraw_v0(
                                leaves,
                                entries_v0.withdrawal_keys(),
                                entries_v0.withdrawal_values(),
                                pairs,
                                entries_v0.proof().raw_data().to_vec(),
                            );
                        }
                    }
                    if let Ok(entries_v1) = MintCotaNFTV1Entries::from_slice(&input_type[1..]) {
                        let withdrawal_keys = entries_v1.withdrawal_keys();
                        let leaves = parse_define(
                            block_number,
                            entries_v1.define_keys(),
                            entries_v1.define_new_values(),
                        );
                        return parse_withdraw_v1(
                            leaves,
                            withdrawal_keys,
                            entries_v1.withdrawal_values(),
                            pairs,
                            entries_v1.proof().raw_data().to_vec(),
                        );
                    }
                    return Err(Error::WitnessParseError(
                        "Mint witness parse error".to_string(),
                    ));
                }
                WITHDRAW => {
                    if let Ok(entries_v0) = WithdrawalCotaNFTEntries::from_slice(&input_type[1..]) {
                        let leaves = parse_hold(entries_v0.hold_keys());
                        return parse_withdraw_v0(
                            leaves,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                            entries_v0.proof().raw_data().to_vec(),
                        );
                    }
                    if let Ok(entries_v1) = WithdrawalCotaNFTV1Entries::from_slice(&input_type[1..])
                    {
                        let leaves = parse_hold(entries_v1.hold_keys());
                        return parse_withdraw_v1(
                            leaves,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                            entries_v1.proof().raw_data().to_vec(),
                        );
                    }
                    return Err(Error::WitnessParseError(
                        "Withdraw witness parse error".to_string(),
                    ));
                }
                TRANSFER => {
                    if let Ok(entries_v0) = TransferCotaNFTEntries::from_slice(&input_type[1..]) {
                        let leaves = parse_claim(entries_v0.claim_keys());
                        return parse_withdraw_v0(
                            leaves,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                            entries_v0.proof().raw_data().to_vec(),
                        );
                    }
                    if let Ok(entries_v1) = TransferCotaNFTV1Entries::from_slice(&input_type[1..]) {
                        let leaves = parse_claim(entries_v1.claim_keys());
                        return parse_withdraw_v1(
                            leaves,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                            entries_v1.proof().raw_data().to_vec(),
                        );
                    }
                    if let Ok(entries_v2) = TransferCotaNFTV2Entries::from_slice(&input_type[1..]) {
                        let leaves = parse_claim(entries_v2.claim_keys());
                        return parse_withdraw_v1(
                            leaves,
                            entries_v2.withdrawal_keys(),
                            entries_v2.withdrawal_values(),
                            pairs,
                            entries_v2.proof().raw_data().to_vec(),
                        );
                    }
                    return Err(Error::WitnessParseError(
                        "Transfer witness parse error".to_string(),
                    ));
                }
                TRANSFER_UPDATE => {
                    if let Ok(entries_v0) =
                        TransferUpdateCotaNFTEntries::from_slice(&input_type[1..])
                    {
                        let leaves = parse_claim(entries_v0.claim_keys());
                        return parse_withdraw_v0(
                            leaves,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                            entries_v0.proof().raw_data().to_vec(),
                        );
                    }
                    if let Ok(entries_v1) =
                        TransferUpdateCotaNFTV1Entries::from_slice(&input_type[1..])
                    {
                        let leaves = parse_claim(entries_v1.claim_keys());
                        return parse_withdraw_v1(
                            leaves,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                            entries_v1.proof().raw_data().to_vec(),
                        );
                    }
                    if let Ok(entries_v2) =
                        TransferUpdateCotaNFTV2Entries::from_slice(&input_type[1..])
                    {
                        let leaves = parse_claim(entries_v2.claim_keys());
                        return parse_withdraw_v1(
                            leaves,
                            entries_v2.withdrawal_keys(),
                            entries_v2.withdrawal_values(),
                            pairs,
                            entries_v2.proof().raw_data().to_vec(),
                        );
                    }
                    return Err(Error::WitnessParseError(
                        "Transfer-update witness parse error".to_string(),
                    ));
                }
                _ => continue,
            }
        } else {
            continue;
        }
    }
    Err(Error::WitnessParseError(
        "Match cota_id and token_index error".to_string(),
    ))
}

fn match_cota_id_index(id: &CotaNFTId, pairs: Pairs) -> bool {
    pairs.to_vec().into_iter().any(|(cota_id, token_index)| {
        &cota_id == id.cota_id().as_slice() && &token_index == id.index().as_slice()
    })
}

fn parse_define(
    block_number: u64,
    define_keys: DefineCotaNFTKeyVec,
    define_values: DefineCotaNFTValueVec,
) -> Leaves {
    let after_padding = block_number > get_value_padding_block_height();
    let mut leaves: Leaves = Vec::with_capacity(define_keys.len());
    for index in 0..define_keys.len() {
        let define_key = define_keys.get(index).unwrap();
        let mut key = [0u8; 32];
        key[0..22].copy_from_slice(define_key.as_slice());

        let define_value = define_values.get(index).unwrap();
        let mut value = [0u8; 32];
        value[0..9].copy_from_slice(define_value.as_slice());
        if after_padding || value == [0u8; 32] {
            value[31] = 255u8;
        }

        leaves.push((H256::from(key), H256::from(value), false));
    }
    leaves
}

fn parse_hold(hold_keys: HoldCotaNFTKeyVec) -> Leaves {
    let mut leaves: Leaves = Vec::with_capacity(hold_keys.len());
    for index in 0..hold_keys.len() {
        let hold_key = hold_keys.get(index).unwrap();
        let mut key = [0u8; 32];
        key[0..26].copy_from_slice(hold_key.as_slice());

        leaves.push((H256::from(key), H256::default(), false));
    }
    leaves
}

fn parse_claim(claim_keys: ClaimCotaNFTKeyVec) -> Leaves {
    let mut leaves: Leaves = Vec::with_capacity(claim_keys.len());
    for index in 0..claim_keys.len() {
        let claim_key = claim_keys.get(index).unwrap();
        let key = blake2b_256(claim_key.as_slice());

        leaves.push((H256::from(key), H256::from([255u8; 32]), false));
    }
    leaves
}

fn parse_withdraw_v0(
    leaves: Leaves,
    withdrawal_keys: WithdrawalCotaNFTKeyVec,
    withdrawal_value: WithdrawalCotaNFTValueVec,
    pairs: Pairs,
    withdrawal_proof: Vec<u8>,
) -> Result<Bytes, Error> {
    let mut count: usize = 0;
    let mut all_leaves: Leaves = vec![];
    all_leaves.extend(leaves);
    for index in 0..withdrawal_keys.len() {
        let withdrawal_key = withdrawal_keys.get(index).unwrap();
        let mut key = [0u8; 32];
        key[0..26].copy_from_slice(withdrawal_key.as_slice());

        let withdrawal_value = withdrawal_value.get(index).unwrap();
        let value = blake2b_256(withdrawal_value.as_slice());

        if match_cota_id_index(&withdrawal_key, pairs) {
            count += 1;
            all_leaves.push((H256::from(key), H256::from(value), true));
        } else {
            all_leaves.push((H256::from(key), H256::from(value), false));
        }
    }
    if count != pairs.len() {
        return Err(Error::WitnessParseError(
            "Match cota_id and token_index error".to_string(),
        ));
    }
    parse_sub_proof(withdrawal_proof, all_leaves)
}

fn parse_withdraw_v1(
    leaves: Leaves,
    withdrawal_keys: WithdrawalCotaNFTKeyV1Vec,
    withdrawal_value: WithdrawalCotaNFTValueV1Vec,
    pairs: Pairs,
    withdrawal_proof: Vec<u8>,
) -> Result<Bytes, Error> {
    let mut count: usize = 0;
    let mut all_leaves: Leaves = vec![];
    all_leaves.extend(leaves);
    for index in 0..withdrawal_keys.len() {
        let withdrawal_key = withdrawal_keys.get(index).unwrap();
        let key = blake2b_256(withdrawal_key.as_slice());

        let withdrawal_value = withdrawal_value.get(index).unwrap();
        let value = blake2b_256(withdrawal_value.as_slice());

        if match_cota_id_index(&withdrawal_key.nft_id(), pairs) {
            count += 1;
            all_leaves.push((H256::from(key), H256::from(value), true));
        } else {
            all_leaves.push((H256::from(key), H256::from(value), false));
        }
    }
    if count != pairs.len() {
        return Err(Error::WitnessParseError(
            "Match cota_id and token_index error".to_string(),
        ));
    }
    parse_sub_proof(withdrawal_proof, all_leaves)
}

fn parse_sub_proof(proof: Vec<u8>, all_leaves: Leaves) -> Result<Bytes, Error> {
    let merkel_proof = CompiledMerkleProof(proof);
    let compiled_proof = merkel_proof
        .extract_proof::<Blake2bHasher>(all_leaves)
        .map_err(|e| Error::SMTProofError(e.to_string()))?;
    let sub_merkel_proof: Vec<u8> = compiled_proof.into();
    let sub_proof = BytesBuilder::default()
        .extend(sub_merkel_proof.iter().map(|v| Byte::from(*v)))
        .build();
    Ok(sub_proof)
}
