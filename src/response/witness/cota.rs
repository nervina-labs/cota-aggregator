use super::info::{ClassInfo, IssuerInfo};
use super::parser::{
    parse_claim, parse_claim_update, parse_define, parse_mint, parse_mint_v1, parse_transfer,
    parse_transfer_update, parse_transfer_update_v1, parse_transfer_v1, parse_update,
    parse_withdrawal, parse_withdrawal_v1,
};
use crate::response::helper::Inserter;
use crate::utils::error::Error;
use ckb_types::bytes::Bytes;
use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::Unpack;
use cota_smt::define::DefineCotaNFTEntries;
use cota_smt::mint::{MintCotaNFTEntries, MintCotaNFTV1Entries};
use cota_smt::transfer::{
    ClaimCotaNFTEntries, TransferCotaNFTEntries, TransferCotaNFTV1Entries,
    WithdrawalCotaNFTEntries, WithdrawalCotaNFTV1Entries,
};
use cota_smt::transfer_update::{
    ClaimUpdateCotaNFTEntries, TransferUpdateCotaNFTEntries, TransferUpdateCotaNFTV1Entries,
};
use cota_smt::update::UpdateCotaNFTEntries;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;
use molecule::error::VerificationError;
use molecule::prelude::Entity;

const CREATE: u8 = 1;
const MINT: u8 = 2;
const WITHDRAW: u8 = 3;
const CLAIM: u8 = 4;
const UPDATE: u8 = 5;
const TRANSFER: u8 = 6;
const CLAIM_UPDATE: u8 = 7;
const TRANSFER_UPDATE: u8 = 8;

pub fn parse_cota_witness(witness: Vec<u8>, version: u8) -> Result<Map<String, Value>, Error> {
    if version > 1 {
        return Err(Error::WitnessParseError("Version invalid".to_string()));
    }
    let witness_args = WitnessArgs::from_slice(&witness)
        .map_err(|_| Error::WitnessParseError("Parse response.witness args error".to_string()))?;
    let mut cota_map = Map::new();
    match witness_args.input_type().to_opt() {
        Some(input_type_) => {
            let input_type: Bytes = input_type_.unpack();
            let (tx_type, slice) = (u8::from(input_type[0]), &input_type[1..]);
            if tx_type > TRANSFER_UPDATE {
                return Err(Error::WitnessParseError(
                    "Not cota response.witness".to_string(),
                ));
            }
            let cota_entries = match tx_type {
                CREATE => {
                    parse_define(DefineCotaNFTEntries::from_slice(slice).map_err(entries_error)?)
                }
                MINT => match version {
                    0 => parse_mint(MintCotaNFTEntries::from_slice(slice).map_err(entries_error)?),
                    _ => parse_mint_v1(
                        MintCotaNFTV1Entries::from_slice(slice).map_err(entries_error)?,
                    ),
                },
                WITHDRAW => match version {
                    0 => parse_withdrawal(
                        WithdrawalCotaNFTEntries::from_slice(slice).map_err(entries_error)?,
                    ),
                    _ => parse_withdrawal_v1(
                        WithdrawalCotaNFTV1Entries::from_slice(slice).map_err(entries_error)?,
                    ),
                },
                CLAIM => {
                    parse_claim(ClaimCotaNFTEntries::from_slice(slice).map_err(entries_error)?)
                }
                UPDATE => {
                    parse_update(UpdateCotaNFTEntries::from_slice(slice).map_err(entries_error)?)
                }
                TRANSFER => match version {
                    0 => parse_transfer(
                        TransferCotaNFTEntries::from_slice(slice).map_err(entries_error)?,
                    ),
                    _ => parse_transfer_v1(
                        TransferCotaNFTV1Entries::from_slice(slice).map_err(entries_error)?,
                    ),
                },
                CLAIM_UPDATE => parse_claim_update(
                    ClaimUpdateCotaNFTEntries::from_slice(slice).map_err(entries_error)?,
                ),
                _ => match version {
                    0 => parse_transfer_update(
                        TransferUpdateCotaNFTEntries::from_slice(slice).map_err(entries_error)?,
                    ),
                    _ => parse_transfer_update_v1(
                        TransferUpdateCotaNFTV1Entries::from_slice(slice).map_err(entries_error)?,
                    ),
                },
            };
            cota_map.insert_obj("cota", cota_entries);
        }
        None => {
            cota_map.insert_null("cota");
        }
    };
    match witness_args.output_type().to_opt() {
        Some(output_type_) => {
            let issuer = serde_json::from_slice::<IssuerInfo>(output_type_.as_slice());
            if issuer.is_ok() {
                cota_map.insert("info".to_owned(), serde_json::json!(issuer.unwrap()));
                return Ok(cota_map);
            }
            let class = serde_json::from_slice::<ClassInfo>(output_type_.as_slice());
            if class.is_ok() {
                cota_map.insert("info".to_owned(), serde_json::json!(class.unwrap()));
                return Ok(cota_map);
            }
            cota_map.insert_null("info");
        }
        None => {
            cota_map.insert_null("info");
        }
    };
    if cota_map.is_empty() {
        return Err(Error::WitnessParseError(
            "Parse cota entries error".to_string(),
        ));
    }
    Ok(cota_map)
}

fn entries_error(_e: VerificationError) -> Error {
    Error::WitnessParseError("Parse cota entries error".to_string())
}
