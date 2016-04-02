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

    match output.len() {
        40 => return Ok(output),
        _ => return Err("Hash is invalid (too short).".to_string()),
    }
}

#[test]
fn parse_info_hash_test() {
    let output = parse_info_hash("%124Vx%9A%BC%DE%F1%23Eg%89%AB%CD%EF%124Vx%9A").unwrap();
    assert_eq!(&output, "123456789abcdef123456789abcdef123456789a");
}
