use crate::utils::error::Error;
use crate::utils::helper::is_ckb_mainnet;
use ckb_sdk::{Address, AddressPayload, NetworkType};
use ckb_types::packed::Script;
use molecule::prelude::Entity;
use serde_json::from_str;
use std::env;

pub fn address_from_script(slice: &[u8]) -> Result<String, Error> {
    let payload =
        AddressPayload::from(Script::from_slice(slice).map_err(|_| Error::CKBScriptError)?);
    let network = if is_ckb_mainnet() {
        NetworkType::Mainnet
    } else {
        NetworkType::Testnet
    };
    Ok(Address::new(network, payload, true).to_string())
}
