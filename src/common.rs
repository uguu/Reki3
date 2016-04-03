extern crate url;
use self::url::percent_encoding::from_hex;

/// Percent-decode the input string
///
/// Not using url::percent_decode because it does not throw an error for malformed percent encoding
/// such as %AK
pub fn percent_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut output: Vec<u8> = Vec::new();

    let mut input_iterator = input.as_bytes().into_iter();
    while let Some(i) = input_iterator.next() {
        match i {
            &b'%' => {
                let hexdigit1 = input_iterator.next()
                    .and_then(|h| from_hex(h.clone()));
                let hexdigit2 = input_iterator.next()
                    .and_then(|h| from_hex(h.clone()));
                match (hexdigit1, hexdigit2) {
                    (Some(h1), Some(h2)) => {
                        output.push(h1 * 0x10 + h2);
                    },
                    _ => {
                        return Err("Invalid percent encoding".to_owned());
                    },
                }
            }
            _ => {
                output.push(i.clone());
            }
        }
    }

    return Ok(output);
}

#[test]
fn percent_decode_test() {
    // Successes
    assert_eq!(percent_decode("%1a").unwrap(), [26]);
    assert_eq!(percent_decode("%1A").unwrap(), [26]);
    assert_eq!(percent_decode("a").unwrap(), [97]);

    // Failures
    assert!(percent_decode("%").is_err()); //too short
    assert!(percent_decode("%a").is_err()); //too short
    assert!(percent_decode("%ak").is_err()); //not in [0-9a-f]
}

/// Converts bytes to a hexstring
///
/// Lowercase-hex
pub fn hexstring(input: &[u8]) -> String
{
    let mut output = String::new();

    for byte in input {
        output.push_str(&format!("{:02x}", byte));
    }

    return output;
}

#[test]
fn hexstring_test() {
    assert_eq!(hexstring(&[0x5, 0x6]), "0506");
    assert_eq!(hexstring(&[0x0B, 0xf1]), "0bf1");
}

/* The info hash is stored in mongo, which is not binary string safe apparently,
so it needs to be parsed to a hexadecimal string rather than a binary one. */
pub fn parse_info_hash(input: &str) -> Result<String, String> {
    let info_hash_binary = match percent_decode(input) {
        Ok(i) => i,
        Err(j) => return Err(j),
    };

    if info_hash_binary.len() != 20 {
        return Err("Info hash is invalid (too short).".to_owned());
    }

    return Ok(hexstring(&info_hash_binary));
}

#[test]
fn parse_info_hash_test() {
    // Success
    assert_eq!(parse_info_hash("%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx%9A").unwrap(), "123456789abcdef123456789abcdef123456789a");

    // Failures
    assert!(parse_info_hash("%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx").is_err()); // too short
    assert!(parse_info_hash("%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vxab").is_err()); // too long
    assert!(parse_info_hash("%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx%ZA").is_err()); // invalid percent encoding
}
