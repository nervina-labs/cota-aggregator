use crate::utils::error::Error;
use crate::utils::helper::{parse_bytes, parse_vec_n, remove_0x};
use jsonrpc_http_server::jsonrpc_core::serde_json::Map;
use jsonrpc_http_server::jsonrpc_core::Value;

pub trait ReqParser: Sized {
    fn from_map(map: &Map<String, Value>) -> Result<Self, Error>;
}

pub fn parse_vec_map<T: ReqParser>(map: &Map<String, Value>, key: &str) -> Result<Vec<T>, Error> {
    let value = map
        .get(key)
        .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
    if !value.is_array() {
        return Err(Error::RequestParamTypeError(key.to_owned()));
    }
    let mut vec: Vec<T> = Vec::new();
    for element in value.as_array().unwrap() {
        if !element.is_object() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        vec.push(T::from_map(element.as_object().unwrap())?)
    }
    Ok(vec)
}

pub trait HexParser {
    fn get_hex_bytes_filed<const N: usize>(&self, key: &str) -> Result<[u8; N], Error>;
    fn get_hex_vec_filed(&self, key: &str) -> Result<Vec<u8>, Error>;
    fn get_int_filed(&self, key: &str) -> Result<u64, Error>;
    fn get_i64_filed(&self, key: &str) -> Result<i64, Error>;
    fn get_u64_filed(&self, key: &str) -> Result<u64, Error>;
    fn get_u32_filed(&self, key: &str) -> Result<u32, Error>;
    fn get_u16_filed(&self, key: &str) -> Result<u16, Error>;
    fn get_u8_filed(&self, key: &str) -> Result<u8, Error>;
    fn get_str_filed(&self, key: &str) -> Result<String, Error>;
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
        Ok(parse_vec_n(result))
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

    fn get_int_filed(&self, key: &str) -> Result<u64, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if v.is_u64() {
            return Ok(v.as_u64().unwrap());
        }
        if v.is_string() {
            let mut temp = v.as_str().unwrap();
            if temp.starts_with("0x") {
                temp = remove_0x(temp);
                return Ok(u64::from_str_radix(temp, 16).unwrap());
            } else {
                return Ok(u64::from_str_radix(temp, 10).unwrap());
            }
        }
        return Err(Error::RequestParamTypeError(key.to_owned()));
    }

    fn get_i64_filed(&self, key: &str) -> Result<i64, Error> {
        Ok(self.get_int_filed(key)? as i64)
    }

    fn get_u64_filed(&self, key: &str) -> Result<u64, Error> {
        Ok(self.get_int_filed(key)?)
    }

    fn get_u32_filed(&self, key: &str) -> Result<u32, Error> {
        Ok(self.get_int_filed(key)? as u32)
    }

    fn get_u16_filed(&self, key: &str) -> Result<u16, Error> {
        Ok(self.get_int_filed(key)? as u16)
    }

    fn get_u8_filed(&self, key: &str) -> Result<u8, Error> {
        Ok(self.get_int_filed(key)? as u8)
    }

    fn get_str_filed(&self, key: &str) -> Result<String, Error> {
        let v = self
            .get(key)
            .ok_or(Error::RequestParamNotFound(key.to_owned()))?;
        if !v.is_string() {
            return Err(Error::RequestParamTypeError(key.to_owned()));
        }
        let result: String = v.as_str().unwrap().parse().unwrap();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_http_server::jsonrpc_core::Value;

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
        map.insert("page".to_owned(), Value::String("32".to_owned()));

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

        assert_eq!(map.get_i64_filed("page"), Ok(32));
    }

    // TODO: Add more tests
}
