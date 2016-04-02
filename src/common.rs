/* The info hash is stored in mongo, which is not binary string safe apparently,
so it needs to be parsed to a hexadecimal string rather than a binary one. */
pub fn parse_info_hash(input: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut input_iterator = input.as_bytes().into_iter();
    while let Some(i) = input_iterator.next() {
        match i {
            &37u8 => { //% in ASCII
                let hexdigits = (input_iterator.next(), input_iterator.next());
                match hexdigits {
                    (Some(hex1), Some(hex2)) => {
                        output.push_str(&String::from_utf8_lossy(&[hex1.clone()]).to_lowercase());
                        output.push_str(&String::from_utf8_lossy(&[hex2.clone()]).to_lowercase());
                    },
                    _ => {},
                }
            }
            _ => {
                output.push_str(&format!("{:X}", i));
            }
        }
    }

    const VALID_CHARACTERS : &'static [char] = &['a', 'b', 'c', 'd',
        'e', 'f', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0'];
    for i in output.chars() {
        match VALID_CHARACTERS.iter().find(|&&j| j == i) {
            Some(_) => {},
            None => return Err("Hash is invalid (encoding).".to_string()),
        }
    }


    if output.len() != 40 {
        return Err("Hash is invalid (too short).".to_string());
    }

    return Ok(output);
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
