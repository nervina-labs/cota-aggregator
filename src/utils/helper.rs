use super::error::Error;
use hex;
use std::convert::TryInto;

pub fn remove_0x(str: &str) -> &str {
    if str.contains("0x") {
        &str[2..]
    } else {
        str
    }
}

pub fn parse_vec_n<const N: usize>(vec: Vec<u8>) -> [u8; N] {
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
    Ok(parse_vec_n(vec))
}

pub fn parse_bytes(value: String) -> Result<Vec<u8>, Error> {
    let v = remove_0x(&value);
    hex::decode(v).map_err(|_| Error::RequestParamHexInvalid(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_0x() {
        assert_eq!(remove_0x("0x123456"), "123456");
        assert_eq!(remove_0x("123456"), "123456");
        assert_eq!(remove_0x("0"), "0");
        assert_eq!(remove_0x("0x"), "");
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
