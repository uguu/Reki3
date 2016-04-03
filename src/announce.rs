use hyper::server::Request;
use std::sync::Mutex;
extern crate redis;
extern crate byteorder;

use common::*;
use self::byteorder::{BigEndian, WriteBytesExt};
use std::net::IpAddr;
use std::net::Ipv4Addr;

pub fn announce(req: &Request, redis_connection: &Mutex<redis::Connection>) -> Result<Vec<u8>, String>
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

    // Generate compact peer entry
    let ip_v4 = match ip {
        IpAddr::V4(i) => i,
        _ => return Err("Ipv6 not implemented yet".to_owned()),
    };
    let peer_v4 = generate_peer_ipv4(ip_v4, port);


    return Ok("worked".to_owned().into_bytes());
}

fn generate_peer_ipv4(ip: Ipv4Addr, port: u16) -> Vec<u8> {
    let mut retval = Vec::with_capacity(6);
    retval.extend(ip.octets().iter());
    retval.write_u16::<BigEndian>(port).unwrap();
    return retval;
}

#[test]
fn generate_peer_ipv4_test() {
    assert_eq!(generate_peer_ipv4(Ipv4Addr::new(127, 0, 0, 1), 0x3039), &[127, 0, 0, 1, 0x30, 0x39]);
}
