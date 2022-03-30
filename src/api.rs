use crate::entries::claim::generate_claim_smt;
use crate::entries::claim_update::generate_claim_update_smt;
use crate::entries::define::generate_define_smt;
use crate::entries::mint::generate_mint_smt;
use crate::entries::transfer::generate_transfer_smt;
use crate::entries::transfer_update::generate_transfer_update_smt;
use crate::entries::update::generate_update_smt;
use crate::entries::withdrawal::generate_withdrawal_smt;
use crate::models::block::get_syncer_tip_block_number;
use crate::models::common::{
    check_cota_claimed, get_define_info_by_cota_id, get_hold_cota, get_mint_cota,
    get_sender_lock_hash_by_cota_nft, get_withdrawal_cota,
};
use crate::models::issuer::get_issuer_info_by_lock_hash;
use crate::request::claim::{ClaimReq, ClaimUpdateReq, IsClaimedReq};
use crate::request::define::{DefineInfoReq, DefineReq};
use crate::request::fetch::{FetchIssuerReq, FetchReq};
use crate::request::mint::MintReq;
use crate::request::transfer::{TransferReq, TransferUpdateReq};
use crate::request::update::UpdateReq;
use crate::request::withdrawal::{SenderLockReq, WithdrawalReq};
use crate::response::claim::{parse_claimed_response, parse_claimed_smt, parse_claimed_update_smt};
use crate::response::define::{parse_define_info, parse_define_smt};
use crate::response::hold::parse_hold_response;
use crate::response::issuer::parse_issuer_response;
use crate::response::mint::{parse_mint_response, parse_mint_smt};
use crate::response::transfer::{parse_transfer_smt, parse_transfer_update_smt};
use crate::response::update::parse_update_smt;
use crate::response::withdrawal::{
    parse_sender_response, parse_withdrawal_response, parse_withdrawal_smt,
};
use crate::smt::db::db::RocksDB;
use cota_smt::smt::blake2b_256;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::{Error, Params, Value};
use log::info;

pub async fn define_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Define request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let define_req = DefineReq::from_map(&map).map_err(|err| err.into())?;
    let define_smt = generate_define_smt(db, define_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_define_smt(define_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn mint_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Mint request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let mint_req = MintReq::from_map(&map).map_err(|err| err.into())?;
    let mint_smt = generate_mint_smt(db, mint_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_mint_smt(mint_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn withdrawal_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Withdrawal request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let withdrawal_req = WithdrawalReq::from_map(&map).map_err(|err| err.into())?;
    let withdrawal_smt = generate_withdrawal_smt(db, withdrawal_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_withdrawal_smt(withdrawal_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn claim_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Claim request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let claim_req = ClaimReq::from_map(&map).map_err(|err| err.into())?;
    let claim_smt = generate_claim_smt(db, claim_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_claimed_smt(claim_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn update_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Update request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let update_req = UpdateReq::from_map(&map).map_err(|err| err.into())?;
    let update_smt = generate_update_smt(db, update_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_update_smt(update_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn transfer_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Transfer request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let transfer_req = TransferReq::from_map(&map).map_err(|err| err.into())?;
    let transfer_smt = generate_transfer_smt(db, transfer_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_transfer_smt(transfer_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn claim_update_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Claim & Update request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let claim_update_req = ClaimUpdateReq::from_map(&map).map_err(|err| err.into())?;
    let claim_update_smt = generate_claim_update_smt(db, claim_update_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_claimed_update_smt(claim_update_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn transfer_update_rpc(params: Params, db: &RocksDB) -> Result<Value, Error> {
    info!("Transfer & Update request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let transfer_update_req = TransferUpdateReq::from_map(&map).map_err(|err| err.into())?;
    let transfer_update_smt = generate_transfer_update_smt(db, transfer_update_req)
        .await
        .map_err(|err| err.into())?;
    let response = parse_transfer_update_smt(transfer_update_smt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn fetch_hold_rpc(params: Params) -> Result<Value, Error> {
    info!("Fetch hold request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (holds, total, block_number) =
        get_hold_cota(&lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_hold_response(holds, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn fetch_withdrawal_rpc(params: Params) -> Result<Value, Error> {
    info!("Fetch withdrawal request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (withdrawals, total, block_number) =
        get_withdrawal_cota(&lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_withdrawal_response(withdrawals, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn fetch_mint_rpc(params: Params) -> Result<Value, Error> {
    info!("Fetch mint request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchReq {
        lock_script,
        page,
        page_size,
    } = FetchReq::from_map(&map).map_err(|err| err.into())?;
    let (withdrawals, total, block_number) =
        get_mint_cota(&lock_script, page, page_size).map_err(|err| err.into())?;
    let response = parse_mint_response(withdrawals, total, page_size, block_number);
    Ok(Value::Object(response))
}

pub async fn is_claimed_rpc(params: Params) -> Result<Value, Error> {
    info!("Is claimed request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let IsClaimedReq {
        lock_script,
        cota_id,
        token_index,
    } = IsClaimedReq::from_map(&map).map_err(|err| err.into())?;
    let (claimed, block_number) =
        check_cota_claimed(&lock_script, cota_id, token_index).map_err(|err| err.into())?;
    let response = parse_claimed_response(claimed, block_number);
    Ok(Value::Object(response))
}

pub async fn get_sender_lock_hash(params: Params) -> Result<Value, Error> {
    info!("Get sender lock request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let SenderLockReq {
        lock_script,
        cota_id,
        token_index,
    } = SenderLockReq::from_map(&map).map_err(|err| err.into())?;
    let sender_lock_hash = get_sender_lock_hash_by_cota_nft(&lock_script, cota_id, token_index)
        .map_err(|err| err.into())?;
    let response = parse_sender_response(sender_lock_hash, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn get_define_info(params: Params) -> Result<Value, Error> {
    info!("Get define info request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let DefineInfoReq { cota_id } = DefineInfoReq::from_map(&map).map_err(|err| err.into())?;
    let define_info_opt = get_define_info_by_cota_id(cota_id).map_err(|err| err.into())?;
    let response = parse_define_info(define_info_opt, get_block_number()?);
    Ok(Value::Object(response))
}

pub async fn get_issuer_info(params: Params) -> Result<Value, Error> {
    info!("Get issuer info request: {:?}", params);
    let map: Map<String, Value> = Params::parse(params)?;
    let FetchIssuerReq { lock_script } =
        FetchIssuerReq::from_map(&map).map_err(|err| err.into())?;
    let lock_hash = blake2b_256(&lock_script);
    info!("lock_hash: {:?}", lock_hash);
    let issuer_info_opt = get_issuer_info_by_lock_hash(lock_hash).map_err(|err| err.into())?;
    let response = parse_issuer_response(issuer_info_opt, get_block_number()?);
    Ok(Value::Object(response))
}

fn get_block_number() -> Result<u64, Error> {
    get_syncer_tip_block_number().map_err(|err| err.into())
}
