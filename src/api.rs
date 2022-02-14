use crate::models::block::get_syncer_tip_block_number;
use crate::models::common::{
    check_cota_claimed, get_hold_cota, get_mint_cota, get_withdrawal_cota,
};
use crate::request::claim::{ClaimReq, IsClaimedReq};
use crate::request::define::DefineReq;
use crate::request::fetch::FetchReq;
use crate::request::mint::MintReq;
use crate::request::transfer::TransferReq;
use crate::request::update::UpdateReq;
use crate::request::withdrawal::WithdrawalReq;
use crate::response::claim::{parse_claimed_response, parse_claimed_smt};
use crate::response::define::parse_define_smt;
use crate::response::hold::parse_hold_response;
use crate::response::mint::{parse_mint_response, parse_mint_smt};
use crate::response::transfer::parse_transfer_smt;
use crate::response::update::parse_update_smt;
use crate::response::withdrawal::{parse_withdrawal_response, parse_withdrawal_smt};
use crate::smt::claim::generate_claim_smt;
use crate::smt::define::generate_define_smt;
use crate::smt::mint::generate_mint_smt;
use crate::smt::transfer::generate_transfer_smt;
use crate::smt::update::generate_update_smt;
use crate::smt::withdrawal::generate_withdrawal_smt;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};

pub async fn define_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let define_req = DefineReq::from_map(&map).map_err(|err| err.into())?;
    let define_smt = generate_define_smt(define_req).map_err(|err| err.into())?;
    let response = parse_define_smt(define_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn mint_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let mint_req = MintReq::from_map(&map).map_err(|err| err.into())?;
    let mint_smt = generate_mint_smt(mint_req).map_err(|err| err.into())?;
    let response = parse_mint_smt(mint_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn withdrawal_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let withdrawal_req = WithdrawalReq::from_map(&map).map_err(|err| err.into())?;
    let withdrawal_smt = generate_withdrawal_smt(withdrawal_req).map_err(|err| err.into())?;
    let response = parse_withdrawal_smt(withdrawal_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn claim_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let claim_req = ClaimReq::from_map(&map).map_err(|err| err.into())?;
    let claim_smt = generate_claim_smt(claim_req).map_err(|err| err.into())?;
    let response = parse_claimed_smt(claim_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn update_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let update_req = UpdateReq::from_map(&map).map_err(|err| err.into())?;
    let update_smt = generate_update_smt(update_req).map_err(|err| err.into())?;
    let response = parse_update_smt(update_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn transfer_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let transfer_req = TransferReq::from_map(&map).map_err(|err| err.into())?;
    let transfer_smt = generate_transfer_smt(transfer_req).map_err(|err| err.into())?;
    let response = parse_transfer_smt(transfer_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn fetch_hold_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (holds, total, block_number) =
        get_hold_cota(lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_hold_response(holds, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn fetch_withdrawal_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (withdrawals, total, block_number) =
        get_withdrawal_cota(lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_withdrawal_response(withdrawals, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn fetch_mint_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (withdrawals, total, block_number) =
        get_mint_cota(lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_mint_response(withdrawals, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn is_claimed_rpc(params: Params) -> Result<Value, Error> {
    let map: Map<String, Value> = Params::parse(params)?;
    let IsClaimedReq {
        lock_hash,
        cota_id,
        token_index,
    } = IsClaimedReq::from_map(&map).map_err(|err| err.into())?;
    let (claimed, block_number) =
        check_cota_claimed(lock_hash, cota_id, token_index).map_err(|err| err.into())?;
    let response = parse_claimed_response(claimed, block_number);
    Ok(Value::Object(response))
}

fn get_block_number() -> Result<u64, Error> {
    get_syncer_tip_block_number().map_err(|err| err.into())
}
