use crate::error::Error;
use crc::{Crc, CRC_32_ISO_HDLC};
use hex;
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;
use std::convert::TryInto;

fn remove_0x(str: &str) -> &str {
    if str.contains("0x") {
        &str[2..]
    } else {
        str
    }
}

pub trait HexParser {
    fn get_hex_bytes_filed<const N: usize>(&self, key: &str) -> Result<[u8; N], Error>;
    fn get_hex_vec_filed(&self, key: &str) -> Result<Vec<u8>, Error>;
}

impl HexParser for Map<String, Value> {
    fn get_hex_bytes_filed<const N: usize>(&self, key: &str) -> Result<[u8; N], Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamHexInvalid(v.to_string()));
        }
        let hex_str = v.as_str().unwrap();
        if !hex_str.contains("0x") {
            return Err(Error::RequestParamHexInvalid(v.to_string()));
        }
        let hex_without_0x = remove_0x(hex_str);
        let result = hex::decode(hex_without_0x)
            .map_err(|_| Error::RequestParamHexInvalid(v.to_string()))?;
        if result.len() != N {
            return Err(Error::RequestParamHexLenError {
                msg:      key.to_owned(),
                got:      result.len(),
                expected: N,
            });
        }
        Ok(parse_n(result))
    }

    fn get_hex_vec_filed(&self, key: &str) -> Result<Vec<u8>, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        let result = parse_bytes(v.as_str().unwrap().to_owned())?;
        Ok(result)
    }
}

pub fn generate_crc(v: &[u8]) -> u32 {
    const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    CRC.checksum(v)
}

fn parse_n<const N: usize>(vec: Vec<u8>) -> [u8; N] {
    vec.try_into().unwrap_or_else(|v: Vec<u8>| {
        panic!("Expected a Vec of length {} but it was {}", N, v.len())
    })
}

pub fn parse_bytes_n<const N: usize>(value: String) -> Result<[u8; N], Error> {
    let vec =
        hex::decode(value.clone()).map_err(|_| Error::RequestParamHexInvalid(value.clone()))?;
    if vec.len() != N {
        return Err(Error::RequestParamHexLenError {
            msg:      value,
            got:      vec.len(),
            expected: N,
        });
    }
    Ok(parse_n(vec))
}

pub fn parse_bytes(value: String) -> Result<Vec<u8>, Error> {
    let v = remove_0x(&value);
    hex::decode(v).map_err(|_| Error::RequestParamHexInvalid(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_http_server::jsonrpc_core::Value;

    #[test]
    fn test_remove_0x() {
        assert_eq!(remove_0x("0x123456"), "123456");
        assert_eq!(remove_0x("123456"), "123456");
        assert_eq!(remove_0x("0"), "0");
        assert_eq!(remove_0x("0x"), "");
    }

    #[test]
    fn test_get_hex_bytes_filed() {
        let mut map = Map::new();
        map.insert(
            "lock_hash".to_owned(),
            Value::String(
                "0x1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1".to_owned(),
            ),
        );
        map.insert(
            "cota_id".to_owned(),
            Value::String("f14aca18aae9df753af304469d8f4ebbc174a938".to_owned()),
        );
        map.insert("total".to_owned(), Value::String("0x0000008g".to_owned()));

        assert_eq!(
            map.get_hex_vec_filed("lock_hash").unwrap(),
            hex::decode("1c5a6f36e6f1485e4df40906f22247888545dd00590a22d985d3be1f63b62db1")
                .unwrap()
        );

        assert_eq!(
            map.get_hex_bytes_filed::<30>("lock_hash"),
            Err(Error::RequestParamHexLenError {
                msg:      "lock_hash".to_owned(),
                got:      32,
                expected: 30,
            })
        );

        assert_eq!(
            map.get_hex_bytes_filed::<32>("lock_has"),
            Err(Error::RequestParamNotFound("lock_has".to_owned()))
        );

        assert_eq!(
            map.get_hex_bytes_filed::<20>("cota_id"),
            Err(Error::RequestParamHexInvalid(
                "\"f14aca18aae9df753af304469d8f4ebbc174a938\"".to_owned()
            ))
        );

        assert_eq!(
            map.get_hex_bytes_filed::<4>("total"),
            Err(Error::RequestParamHexInvalid("\"0x0000008g\"".to_owned()))
        );
    }

    #[test]
    fn test_generate_crc() {
        assert_eq!(generate_crc("cota".as_bytes()), 985327312u32);
        assert_eq!(
            generate_crc(
                &"41a7a00cced6ecc5be4ec248c01096b705e4cd9d8b0a5ef5cdb6760a3742f5de"
                    .as_bytes()
                    .to_vec()
            ),
            2934249110
        )
    }

    #[test]
    fn test_parse_bytes_n() {
        assert_eq!(
            parse_bytes_n::<36>(
                "1c5a6f36e6f1485e4df40906f22247888545dd00590a22d9h5d3be1f63b62db100000000"
                    .to_string()
            ),
            Err(Error::RequestParamHexInvalid(
                "1c5a6f36e6f1485e4df40906f22247888545dd00590a22d9h5d3be1f63b62db100000000"
                    .to_owned()
            ))
        );
        assert_eq!(
            parse_bytes_n::<20>("f14aca18aae9df723af304469d8f4ebbc174a938".to_string()),
            Ok([
                241, 74, 202, 24, 170, 233, 223, 114, 58, 243, 4, 70, 157, 143, 78, 187, 193, 116,
                169, 56
            ])
        );

        assert_eq!(
            parse_bytes_n::<4>("f14acd10".to_string()),
            Ok([241, 74, 205, 16])
        );
    }

    #[test]
    fn test_parse_bytes() {
        assert_eq!(
            parse_bytes(
                "1c5a6f36e6f1485e4df40906f22247888545dd00590a22d9h5d3be1f63b62db1".to_string()
            ),
            Err(Error::RequestParamHexInvalid(
                "1c5a6f36e6f1485e4df40906f22247888545dd00590a22d9h5d3be1f63b62db1".to_owned()
            ))
        );
    }
}
