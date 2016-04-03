use hyper::server::Request;

use common::*;

pub fn announce(req: &Request) -> Result<Vec<u8>, String>
{
    let query_hashmap = query_hashmap(&req.uri);

    // Check we have everything we need
    let info_hash = try!(query_hashmap.get("info_hash")
        .ok_or_else(|| "No info_hash specified".to_owned())
        .and_then(|i| parse_info_hash(i)));

    let peer_id = try!(query_hashmap.get("peer_id")
        .ok_or_else(|| "No peer_id specified".to_owned())
        .and_then(|i| parse_peer_id(i)));
    let port = try!(query_hashmap.get("port")
        .ok_or_else(|| "No port specified".to_owned())
        .and_then(|i| i.parse::<u16>().map_err(|_| "Invalid port specified".to_owned())));
    let left = try!(query_hashmap.get("left")
        .ok_or_else(|| "No left specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid left specified".to_owned())));
    let compact = try!(query_hashmap.get("compact")
        .ok_or_else(|| "No compact specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid left specified".to_owned())));
    let ip = req.remote_addr.ip();

    return Ok("worked".to_owned().into_bytes());
}
