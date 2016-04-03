use hyper::server::Request;

use common::*;

pub fn announce(req: &Request) -> Result<Vec<u8>, String>
{
    let query_hashmap = query_hashmap(&req.uri);

    // Check we have everything we need
    let info_hash = match query_hashmap.get("info_hash")
        .ok_or_else(|| "No info_hash specified".to_owned())
        .and_then(|i| parse_info_hash(i)) {
            Ok(j) => j,
            Err(j) => return Err(j),
    };
    let port = match query_hashmap.get("port")
        .ok_or_else(|| "No port specified".to_owned())
        .and_then(|i| i.parse::<u16>().map_err(|_| "Invalid port specified".to_owned())) {
            Ok(i) => i,
            Err(j) => return Err(j),
    };

    return Ok("worked".to_owned().into_bytes());
}
