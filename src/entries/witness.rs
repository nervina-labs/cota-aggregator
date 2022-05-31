use crate::entries::helper::get_value_padding_block_height;
use crate::utils::error::Error;
use ckb_types::packed::{BytesVec, WitnessArgs};
use cota_smt::common::{
    Byte32, Bytes, ClaimCotaNFTKeyVec, CotaNFTId, DefineCotaNFTKeyVec, DefineCotaNFTValueVec,
    HoldCotaNFTKeyVec, WithdrawalCotaNFTKeyV1Vec, WithdrawalCotaNFTKeyVec,
    WithdrawalCotaNFTValueV1Vec, WithdrawalCotaNFTValueVec,
};
use cota_smt::mint::{MintCotaNFTEntries, MintCotaNFTV1Entries};
use cota_smt::smt::blake2b_256;
use cota_smt::transfer::{
    TransferCotaNFTEntries, TransferCotaNFTV1Entries, TransferCotaNFTV2Entries,
    WithdrawalCotaNFTEntries, WithdrawalCotaNFTV1Entries,
};
use cota_smt::transfer_update::{
    TransferUpdateCotaNFTEntries, TransferUpdateCotaNFTV1Entries, TransferUpdateCotaNFTV2Entries,
};
use molecule::prelude::Entity;

const MINT: u8 = 2;
const WITHDRAW: u8 = 3;
const TRANSFER: u8 = 6;
const TRANSFER_UPDATE: u8 = 8;

type Pairs<'a> = &'a [([u8; 20], [u8; 4])];

#[derive(Clone, Debug, Default)]
pub struct WithdrawLeafProof {
    pub leaf_keys:      Vec<Byte32>,
    pub leaf_values:    Vec<Byte32>,
    pub withdraw_proof: Bytes,
}

pub fn parse_withdraw_witness(
    witnesses: BytesVec,
    pairs: Pairs,
    block_number: u64,
) -> Result<WithdrawLeafProof, Error> {
    let mut leaf_keys: Vec<Byte32> = vec![];
    let mut leaf_values: Vec<Byte32> = vec![];
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
                            parse_define(
                                block_number,
                                &mut leaf_keys,
                                &mut leaf_values,
                                entries_v0.define_keys(),
                                entries_v0.define_new_values(),
                            );
                            parse_withdraw_v0(
                                &mut leaf_keys,
                                &mut leaf_values,
                                entries_v0.withdrawal_keys(),
                                entries_v0.withdrawal_values(),
                                pairs,
                            )?;
                            return Ok(WithdrawLeafProof {
                                leaf_keys:      leaf_keys.clone(),
                                leaf_values:    leaf_values.clone(),
                                withdraw_proof: entries_v0.proof(),
                            });
                        }
                    }
                    if let Ok(entries_v1) = MintCotaNFTV1Entries::from_slice(&input_type[1..]) {
                        let withdrawal_keys = entries_v1.withdrawal_keys();
                        parse_define(
                            block_number,
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v1.define_keys(),
                            entries_v1.define_new_values(),
                        );
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            withdrawal_keys,
                            entries_v1.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v1.proof(),
                        });
                    }
                    return Err(Error::WitnessParseError(
                        "Mint witness parse error".to_string(),
                    ));
                }
                WITHDRAW => {
                    if let Ok(entries_v0) = WithdrawalCotaNFTEntries::from_slice(&input_type[1..]) {
                        parse_hold(&mut leaf_keys, &mut leaf_values, entries_v0.hold_keys());
                        parse_withdraw_v0(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v0.proof(),
                        });
                    }
                    if let Ok(entries_v1) = WithdrawalCotaNFTV1Entries::from_slice(&input_type[1..])
                    {
                        parse_hold(&mut leaf_keys, &mut leaf_values, entries_v1.hold_keys());
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v1.proof(),
                        });
                    }
                    return Err(Error::WitnessParseError(
                        "Withdraw witness parse error".to_string(),
                    ));
                }
                TRANSFER => {
                    if let Ok(entries_v0) = TransferCotaNFTEntries::from_slice(&input_type[1..]) {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v0.claim_keys());
                        parse_withdraw_v0(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v0.proof(),
                        });
                    }
                    if let Ok(entries_v1) = TransferCotaNFTV1Entries::from_slice(&input_type[1..]) {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v1.claim_keys());
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v1.proof(),
                        });
                    }
                    if let Ok(entries_v2) = TransferCotaNFTV2Entries::from_slice(&input_type[1..]) {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v2.claim_keys());
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v2.withdrawal_keys(),
                            entries_v2.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v2.proof(),
                        });
                    }
                    return Err(Error::WitnessParseError(
                        "Transfer witness parse error".to_string(),
                    ));
                }
                TRANSFER_UPDATE => {
                    if let Ok(entries_v0) =
                        TransferUpdateCotaNFTEntries::from_slice(&input_type[1..])
                    {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v0.claim_keys());
                        parse_withdraw_v0(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v0.withdrawal_keys(),
                            entries_v0.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v0.proof(),
                        });
                    }
                    if let Ok(entries_v1) =
                        TransferUpdateCotaNFTV1Entries::from_slice(&input_type[1..])
                    {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v1.claim_keys());
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v1.withdrawal_keys(),
                            entries_v1.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v1.proof(),
                        });
                    }
                    if let Ok(entries_v2) =
                        TransferUpdateCotaNFTV2Entries::from_slice(&input_type[1..])
                    {
                        parse_claim(&mut leaf_keys, &mut leaf_values, entries_v2.claim_keys());
                        parse_withdraw_v1(
                            &mut leaf_keys,
                            &mut leaf_values,
                            entries_v2.withdrawal_keys(),
                            entries_v2.withdrawal_values(),
                            pairs,
                        )?;
                        return Ok(WithdrawLeafProof {
                            leaf_keys:      leaf_keys.clone(),
                            leaf_values:    leaf_values.clone(),
                            withdraw_proof: entries_v2.proof(),
                        });
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
    leaf_keys: &mut Vec<Byte32>,
    leaf_values: &mut Vec<Byte32>,
    define_keys: DefineCotaNFTKeyVec,
    define_values: DefineCotaNFTValueVec,
) {
    let after_padding = block_number > get_value_padding_block_height();
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

        leaf_keys.push(Byte32::from_slice(&key).unwrap());
        leaf_values.push(Byte32::from_slice(&value).unwrap());
    }
}

fn parse_hold(
    leaf_keys: &mut Vec<Byte32>,
    leaf_values: &mut Vec<Byte32>,
    hold_keys: HoldCotaNFTKeyVec,
) {
    for index in 0..hold_keys.len() {
        let hold_key = hold_keys.get(index).unwrap();
        let mut key = [0u8; 32];
        key[0..26].copy_from_slice(hold_key.as_slice());
        leaf_keys.push(Byte32::from_slice(&key).unwrap());
        leaf_values.push(Byte32::default());
    }
}

fn parse_withdraw_v0(
    leaf_keys: &mut Vec<Byte32>,
    leaf_values: &mut Vec<Byte32>,
    withdrawal_keys: WithdrawalCotaNFTKeyVec,
    withdrawal_value: WithdrawalCotaNFTValueVec,
    pairs: Pairs,
) -> Result<(), Error> {
    let mut count: usize = 0;
    for index in 0..withdrawal_keys.len() {
        let withdrawal_key = withdrawal_keys.get(index).unwrap();
        if match_cota_id_index(&withdrawal_key, pairs) {
            count += 1;
            continue;
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(withdrawal_key.as_slice());
        leaf_keys.push(Byte32::from_slice(&key).unwrap());

        let withdrawal_value = withdrawal_value.get(index).unwrap();
        let value = blake2b_256(withdrawal_value.as_slice());
        leaf_values.push(Byte32::from_slice(&value).unwrap());
    }
    if count != pairs.len() {
        return Err(Error::WitnessParseError(
            "Match cota_id and token_index error".to_string(),
        ));
    }
    Ok(())
}

fn parse_withdraw_v1(
    leaf_keys: &mut Vec<Byte32>,
    leaf_values: &mut Vec<Byte32>,
    withdrawal_keys: WithdrawalCotaNFTKeyV1Vec,
    withdrawal_value: WithdrawalCotaNFTValueV1Vec,
    pairs: Pairs,
) -> Result<(), Error> {
    let mut count: usize = 0;
    for index in 0..withdrawal_keys.len() {
        let withdrawal_key = withdrawal_keys.get(index).unwrap();
        if match_cota_id_index(&withdrawal_key.nft_id(), pairs) {
            count += 1;
            continue;
        }

        let key = blake2b_256(withdrawal_key.as_slice());
        leaf_keys.push(Byte32::from_slice(&key).unwrap());

        let withdrawal_value = withdrawal_value.get(index).unwrap();
        let value = blake2b_256(withdrawal_value.as_slice());
        leaf_values.push(Byte32::from_slice(&value).unwrap());
    }
    if count != pairs.len() {
        return Err(Error::WitnessParseError(
            "Match cota_id and token_index error".to_string(),
        ));
    }
    Ok(())
}

fn parse_claim(
    leaf_keys: &mut Vec<Byte32>,
    leaf_values: &mut Vec<Byte32>,
    claim_keys: ClaimCotaNFTKeyVec,
) {
    for index in 0..claim_keys.len() {
        let claim_key = claim_keys.get(index).unwrap();
        let mut key = [0u8; 32];
        key.copy_from_slice(&blake2b_256(claim_key.as_slice()));
        leaf_keys.push(Byte32::from_slice(&key).unwrap());
        leaf_values.push(Byte32::from_slice(&[255u8; 32]).unwrap());
    }
}
