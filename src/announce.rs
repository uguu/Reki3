use hyper::server::Request;
use hyper::uri::RequestUri;

extern crate url;
use self::url::Url;
use self::url::UrlParser;

use std::collections::HashMap;

pub fn announce(req: &Request) -> Result<Vec<u8>, String>
{
    // Parse query pairs out of URL path and store in a HashMap
    let path = match req.uri {
        RequestUri::AbsolutePath(ref i) => i,
        _ => return Err("Problem retrieving path".to_string()),
    };

    let base_url = Url::parse("http://localhost/").unwrap();
    let mut url_parser = UrlParser::new();
    url_parser.base_url(&base_url); // need a full absolute url for some reason

    let url = match url_parser.parse(path) {
        Ok(i) => i,
        Err(_) => return Err("Problem parsing URL".to_string()),
    };

    let query_pairs = url.query_pairs().unwrap_or(vec![]);
    let mut query_hashmap: HashMap<&str, &str> = HashMap::new();
    for &(ref key, ref value) in &query_pairs {
        query_hashmap.insert(key, value);
    }

    // Check we have everything we need
    let info_hash = match query_hashmap.get("info_hash") {
        Some(i) => i,
        None => return Err("No info_hash specified".to_string()),
    };
    let port = match query_hashmap.get("port") {
        Some(i) => i,
        None => return Err("No port specified".to_string()),
    };

    return Ok("worked".to_string().into_bytes());
}
