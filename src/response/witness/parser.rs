use crate::response::helper::Inserter;
use ckb_types::packed::Script;
use cota_smt::common::{
    ClaimCotaNFTInfo, ClaimCotaNFTKey, CotaNFTId, CotaNFTInfo, DefineCotaNFTId, DefineCotaNFTValue,
    WithdrawalCotaNFTKeyV1, WithdrawalCotaNFTValue, WithdrawalCotaNFTValueV1,
};
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
use molecule::prelude::Entity;

pub fn parse_define(obj: DefineCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "define".to_owned());
    map.insert_obj_vec(
        "define_keys",
        obj.define_keys()
            .into_iter()
            .map(parse_define_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "define_values",
        obj.define_values()
            .into_iter()
            .map(parse_define_cota_nft_value)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_mint(obj: MintCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "mint".to_owned());
    map.insert_obj_vec(
        "define_keys",
        obj.define_keys()
            .into_iter()
            .map(parse_define_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "define_old_values",
        obj.define_old_values()
            .into_iter()
            .map(parse_define_cota_nft_value)
            .collect(),
    );
    map.insert_obj_vec(
        "define_new_values",
        obj.define_new_values()
            .into_iter()
            .map(parse_define_cota_nft_value)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_mint_v1(obj: MintCotaNFTV1Entries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "mint".to_owned());
    map.insert_obj_vec(
        "define_keys",
        obj.define_keys()
            .into_iter()
            .map(parse_define_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "define_old_values",
        obj.define_old_values()
            .into_iter()
            .map(parse_define_cota_nft_value)
            .collect(),
    );
    map.insert_obj_vec(
        "define_new_values",
        obj.define_new_values()
            .into_iter()
            .map(parse_define_cota_nft_value)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_withdrawal_cota_nft_key_v1)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value_v1)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_withdrawal(obj: WithdrawalCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "withdraw".to_owned());
    map.insert_obj_vec(
        "hold_keys",
        obj.hold_keys().into_iter().map(parse_cota_nft_id).collect(),
    );
    map.insert_obj_vec(
        "hold_values",
        obj.hold_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_withdrawal_v1(obj: WithdrawalCotaNFTV1Entries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "withdraw".to_owned());
    map.insert_obj_vec(
        "hold_keys",
        obj.hold_keys().into_iter().map(parse_cota_nft_id).collect(),
    );
    map.insert_obj_vec(
        "hold_values",
        obj.hold_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_withdrawal_cota_nft_key_v1)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value_v1)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_claim(obj: ClaimCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "claim".to_owned());
    map.insert_obj_vec(
        "hold_keys",
        obj.hold_keys().into_iter().map(parse_cota_nft_id).collect(),
    );
    map.insert_obj_vec(
        "hold_values",
        obj.hold_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_array(
        "claim_values",
        obj.claim_values()
            .into_iter()
            .map(|value| Value::String(slice_to_hex(value.as_slice())))
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_transfer(obj: TransferCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "transfer".to_owned());
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_array(
        "claim_values",
        obj.claim_values()
            .into_iter()
            .map(|v| Value::String(slice_to_hex(v.as_slice())))
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str(
        "withdrawal_proof",
        slice_to_hex(&obj.withdrawal_proof().raw_data().to_vec()),
    );
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_transfer_v1(obj: TransferCotaNFTV1Entries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "transfer".to_owned());
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_array(
        "claim_values",
        obj.claim_values()
            .into_iter()
            .map(|value| Value::String(slice_to_hex(value.as_slice())))
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_withdrawal_cota_nft_key_v1)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value_v1)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str(
        "withdrawal_proof",
        slice_to_hex(&obj.withdrawal_proof().raw_data().to_vec()),
    );
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_update(obj: UpdateCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "update".to_owned());
    map.insert_obj_vec(
        "hold_keys",
        obj.hold_keys().into_iter().map(parse_cota_nft_id).collect(),
    );
    map.insert_obj_vec(
        "hold_old_values",
        obj.hold_old_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "hold_new_values",
        obj.hold_new_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_claim_update(obj: ClaimUpdateCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "claim_update".to_owned());
    map.insert_obj_vec(
        "hold_keys",
        obj.hold_keys().into_iter().map(parse_cota_nft_id).collect(),
    );
    map.insert_obj_vec(
        "hold_new_values",
        obj.hold_values()
            .into_iter()
            .map(parse_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_obj_vec(
        "claim_infos",
        obj.claim_infos()
            .into_iter()
            .map(parse_claim_cota_nft_info)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str(
        "withdrawal_proof",
        slice_to_hex(&obj.withdrawal_proof().raw_data().to_vec()),
    );
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_transfer_update(obj: TransferUpdateCotaNFTEntries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "transfer_update".to_owned());
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_obj_vec(
        "claim_infos",
        obj.claim_infos()
            .into_iter()
            .map(parse_claim_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_cota_nft_id)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str(
        "withdrawal_proof",
        slice_to_hex(&obj.withdrawal_proof().raw_data().to_vec()),
    );
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

pub fn parse_transfer_update_v1(obj: TransferUpdateCotaNFTV1Entries) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("type", "transfer_update".to_owned());
    map.insert_obj_vec(
        "claim_keys",
        obj.claim_keys()
            .into_iter()
            .map(parse_claim_cota_nft_key)
            .collect(),
    );
    map.insert_obj_vec(
        "claim_infos",
        obj.claim_infos()
            .into_iter()
            .map(parse_claim_cota_nft_info)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_keys",
        obj.withdrawal_keys()
            .into_iter()
            .map(parse_withdrawal_cota_nft_key_v1)
            .collect(),
    );
    map.insert_obj_vec(
        "withdrawal_values",
        obj.withdrawal_values()
            .into_iter()
            .map(parse_withdrawal_cota_nft_value_v1)
            .collect(),
    );
    map.insert_str("proof", slice_to_hex(&obj.proof().raw_data().to_vec()));
    map.insert_str(
        "withdrawal_proof",
        slice_to_hex(&obj.withdrawal_proof().raw_data().to_vec()),
    );
    map.insert_str("action", slice_to_hex(&obj.action().raw_data().to_vec()));
    map
}

fn parse_cota_nft_id(obj: CotaNFTId) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("smt_type", slice_to_hex(obj.smt_type().as_slice()));
    map.insert_str("cota_id", slice_to_hex(obj.cota_id().as_slice()));
    map.insert_str("index", slice_to_hex(obj.index().as_slice()));
    map
}

fn parse_cota_nft_info(obj: CotaNFTInfo) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("configure", slice_to_hex(obj.configure().as_slice()));
    map.insert_str("state", slice_to_hex(obj.state().as_slice()));
    map.insert_str(
        "characteristic",
        slice_to_hex(obj.characteristic().as_slice()),
    );
    map
}

fn parse_define_cota_nft_id(obj: DefineCotaNFTId) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("smt_type", slice_to_hex(obj.smt_type().as_slice()));
    map.insert_str("cota_id", slice_to_hex(obj.cota_id().as_slice()));
    map
}

fn parse_define_cota_nft_value(obj: DefineCotaNFTValue) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("total", slice_to_hex(obj.total().as_slice()));
    map.insert_str("issued", slice_to_hex(obj.issued().as_slice()));
    map.insert_str("configure", slice_to_hex(obj.configure().as_slice()));
    map
}

fn parse_withdrawal_cota_nft_value(obj: WithdrawalCotaNFTValue) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_obj("nft_info", parse_cota_nft_info(obj.nft_info()));
    map.insert_str("to_lock", slice_to_hex(&obj.to_lock().raw_data().to_vec()));
    map.insert_str("out_point", slice_to_hex(obj.out_point().as_slice()));
    map
}

fn parse_claim_cota_nft_key(obj: ClaimCotaNFTKey) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_obj("nft_id", parse_cota_nft_id(obj.nft_id()));
    map.insert_str("out_point", slice_to_hex(obj.out_point().as_slice()));
    map
}

fn parse_claim_cota_nft_info(obj: ClaimCotaNFTInfo) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_str("version", slice_to_hex(obj.version().as_slice()));
    map.insert_obj("nft_info", parse_cota_nft_info(obj.nft_info()));
    map
}

fn parse_withdrawal_cota_nft_key_v1(obj: WithdrawalCotaNFTKeyV1) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_obj("nft_id", parse_cota_nft_id(obj.nft_id()));
    map.insert_str("out_point", slice_to_hex(obj.out_point().as_slice()));
    map
}

fn parse_withdrawal_cota_nft_value_v1(obj: WithdrawalCotaNFTValueV1) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert_obj("nft_info", parse_cota_nft_info(obj.nft_info()));
    map.insert_obj("to_lock", parse_script(&obj.to_lock().raw_data().to_vec()));
    map
}

fn parse_script(slice: &[u8]) -> Map<String, Value> {
    let mut map = Map::new();
    let script = Script::from_slice(slice).expect("Parse script error");
    map.insert_str("code_hash", slice_to_hex(script.code_hash().as_slice()));
    map.insert_str("hash_type", slice_to_hex(script.hash_type().as_slice()));
    map.insert_str("args", slice_to_hex(&script.args().raw_data().to_vec()));
    map
}

fn slice_to_hex(slice: &[u8]) -> String {
    format!("0x{}", hex::encode(slice))
}
