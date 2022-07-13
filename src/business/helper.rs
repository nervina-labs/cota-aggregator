use crate::utils::error::Error;
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::packed::Script;
use molecule::prelude::Entity;
use serde_json::from_str;
use std::env;

pub fn address_from_script(slice: &[u8]) -> Result<String, Error> {
    let is_mainnet: bool = match env::var("IS_MAINNET") {
        Ok(mainnet) => from_str::<bool>(&mainnet).unwrap(),
        Err(_e) => false,
    };
    let payload =
        AddressPayload::from(Script::from_slice(slice).map_err(|_| Error::CKBScriptError)?);
    let network = if is_mainnet {
        NetworkType::Mainnet
    } else {
        NetworkType::Testnet
    };
    Ok(Address::new(network, payload, true).to_string())
}
