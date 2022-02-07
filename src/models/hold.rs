use crate::error::Error;
use crate::models::{establish_connection, parse_lock_hash};
use crate::schema::hold_cota_nft_kv_pairs::dsl::hold_cota_nft_kv_pairs;
use crate::schema::hold_cota_nft_kv_pairs::*;
use crate::utils::parse_bytes_n;
use diesel::dsl::And;
use diesel::expression::array_comparison::In;
use diesel::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Debug)]
struct HoldCotaNft {
    pub cota_id:        String,
    pub token_index:    u32,
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HoldDb {
    pub cota_id:        [u8; 20],
    pub token_index:    [u8; 4],
    pub state:          u8,
    pub configure:      u8,
    pub characteristic: [u8; 20],
}

pub fn get_hold_cota_by_lock_hash(
    lock_hash_: [u8; 32],
    cota_id_and_token_index_pairs: Option<Vec<([u8; 20], [u8; 4])>>,
) -> Result<Vec<HoldDb>, Error> {
    let conn = &establish_connection();
    let (lock_hash_hex, lock_hash_crc_) = parse_lock_hash(lock_hash_);
    let select = hold_cota_nft_kv_pairs
        .select((cota_id, token_index, configure, state, characteristic))
        .filter(
            lock_hash_crc
                .eq(lock_hash_crc_)
                .and(lock_hash.eq(lock_hash_hex)),
        );
    let mut result = match cota_id_and_token_index_pairs {
        Some(pairs) => {
            let (cota_id_array, token_index_array) = parse_cota_id_and_token_index_pairs(pairs);
            select
                .filter(cota_id.eq_any(cota_id_array))
                .filter(token_index.eq_any(token_index_array))
                .load::<HoldCotaNft>(conn)
        }
        None => select.load::<HoldCotaNft>(conn),
    };
    result.map_or_else(
        |e| Err(Error::DatabaseQueryError(e.to_string())),
        |holds| {
            Ok(holds
                .into_iter()
                .map(|hold| HoldDb {
                    cota_id:        parse_bytes_n::<20>(hold.cota_id).unwrap(),
                    token_index:    hold.token_index.to_be_bytes(),
                    state:          hold.state,
                    configure:      hold.configure,
                    characteristic: parse_bytes_n::<20>(hold.characteristic).unwrap(),
                })
                .collect())
        },
    )
}

fn parse_cota_id_and_token_index_pairs(pairs: Vec<([u8; 20], [u8; 4])>) -> (Vec<String>, Vec<u32>) {
    let cota_id_hexes: Vec<String> = pairs.iter().map(|pair| hex::encode(pair.0)).collect();
    let token_index_hexes: Vec<u32> = pairs
        .iter()
        .map(|pair| u32::from_be_bytes(pair.1))
        .collect();
    (cota_id_hexes, token_index_hexes)
}
