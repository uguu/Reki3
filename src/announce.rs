use hyper::server::Request;
use hyper::uri::RequestUri;

use std::collections::HashMap;

use common::*;

pub fn announce(req: &Request) -> Result<Vec<u8>, String>
{
    // Parse query pairs out of URL path and store in a HashMap
    let path = match req.uri {
        RequestUri::AbsolutePath(ref i) => i,
        _ => return Err("Problem retrieving path".to_string()),
    };

    let mut query_hashmap: HashMap<&str, &str> = HashMap::new();

    let query = match path.find('?') {
        Some(i) => &path[i+1..],
        None => "",
    };

    for component in query.split('&') {
        match component.find('=') {
            Some(position) => {
                query_hashmap.insert(&component[..position], &component[position + 1..]);
            }
            None => {},
        }
    }

    // Check we have everything we need
    let info_hash = match query_hashmap.get("info_hash")
        .ok_or_else(|| "No info_hash specified".to_string())
        .and_then(|i| parse_info_hash(i)) {
            Ok(j) => j,
            Err(j) => return Err(j),
    };
    let port = match query_hashmap.get("port")
        .ok_or_else(|| "No port specified".to_string())
        .and_then(|i| i.parse::<u16>().map_err(|_| "Invalid port specified".to_string())) {
            Ok(i) => i,
            Err(j) => return Err(j),
    };

    return Ok("worked".to_string().into_bytes());
}
