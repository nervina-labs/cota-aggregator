use crate::utils::error::Error;
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::packed::Script;
use ckb_types::prelude::{Builder, Pack};
use molecule::prelude::Entity;
use serde_json::from_str;
use std::{env, str::FromStr};

pub fn address_from_script(slice: &[u8]) -> Result<String, Error> {
    let is_mainnet: bool = match env::var("IS_MAINNET") {
        Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
        Err(_e) => false,
    };
    let payload =
        AddressPayload::from(Script::from_slice(slice).map_err(|_| Error::CKBScriptInvalid)?);
    let network = if is_mainnet {
        NetworkType::Mainnet
    } else {
        NetworkType::Testnet
    };

    Ok(Address::new(network, payload, true).to_string())
}

pub fn script_from_address(address: String) -> Result<Script, Error> {
    let addr = Address::from_str(&address).map_err(|e| Error::CKBRPCInvalid(e))?;
    let payload = addr.payload();
    let script = Script::new_builder()
        .hash_type(payload.hash_type().into())
        .code_hash(payload.code_hash(None))
        .args(payload.args().pack())
        .build();
    Ok(script)
}
