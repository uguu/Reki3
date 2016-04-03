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
    let peer_id = match query_hashmap.get("peer_id")
        .ok_or_else(|| "No peer_id specified".to_owned())
        .and_then(|i| parse_peer_id(i)) {
            Ok(j) => j,
            Err(j) => return Err(j),
    };
    let port = match query_hashmap.get("port")
        .ok_or_else(|| "No port specified".to_owned())
        .and_then(|i| i.parse::<u16>().map_err(|_| "Invalid port specified".to_owned())) {
            Ok(i) => i,
            Err(j) => return Err(j),
    };
    let left = match query_hashmap.get("left")
        .ok_or_else(|| "No left specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid left specified".to_owned())) {
            Ok(i) => i,
            Err(j) => return Err(j),
    };
    let compact = match query_hashmap.get("compact")
        .ok_or_else(|| "No compact specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid left specified".to_owned())) {
            Ok(i) => i,
            Err(j) => return Err(j),
    };
    let ip = req.remote_addr.ip();

    return Ok("worked".to_owned().into_bytes());
}
