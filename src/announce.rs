use hyper::server::Request;
use std::sync::Mutex;
extern crate redis;
extern crate byteorder;

use common::*;
use self::byteorder::{BigEndian, WriteBytesExt};
use std::net::IpAddr;
extern crate time;
use std::error::Error;
use std::str::FromStr;

enum IPVersion {
    V4,
    V6
}

pub fn announce(req: &Request, redis_connection: &Mutex<redis::Connection>,
    announce_interval: u32,
    drop_threshold: u32) -> Result<Vec<u8>, String>
{
    // Get which ip version we are serving
    let ip_version = match req.remote_addr.ip() {
        IpAddr::V4(_) => IPVersion::V4,
        IpAddr::V6(_) => IPVersion::V6,
    };

    let query_hashmap = query_hashmap(&req.uri);

    // Check we have everything we need
    let info_hash = try!(query_hashmap.get("info_hash")
        .ok_or_else(|| "No info_hash specified".to_owned())
        .and_then(|i| parse_info_hash(i)));
    debug!("    infohash={}", info_hash);
    /*let peer_id = try!(query_hashmap.get("peer_id")
        .ok_or_else(|| "No peer_id specified".to_owned())
        .and_then(|i| parse_peer_id(i)));*/
    let port = try!(query_hashmap.get("port")
        .ok_or_else(|| "No port specified".to_owned())
        .and_then(|i| i.parse::<u16>().map_err(|_| "Invalid port specified".to_owned())));
    let left = try!(query_hashmap.get("left")
        .ok_or_else(|| "No left specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid left specified".to_owned())));
    let compact = try!(query_hashmap.get("compact")
        .ok_or_else(|| "No compact specified".to_owned())
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid compact specified".to_owned())));
    if compact != 1 {
        return Err("This tracker only supports compact responses".to_owned());
    }
    let numwant = try!(query_hashmap.get("numwant")
        .unwrap_or(&"50")
        .parse::<u64>().map_err(|_| "Invalid numwant specified".to_owned()));

    // Get ip
    let ip: Option<IpAddr>;
    match ip_version {
        IPVersion::V4 => {
            ip = query_hashmap.get("ip").
                and_then(|i| IpAddr::from_str(i).ok()).
                // only acccept it if it's an ipv4 address
                and_then(|i| if let IpAddr::V4(_) = i { Some(i) } else { None });
        }
        IPVersion::V6 => {
            ip = query_hashmap.get("ipv6").
                or(query_hashmap.get("ip")).
                and_then(|i| IpAddr::from_str(i).ok()).
                // only acccept it if it's an ipv6 address
                and_then(|i| if let IpAddr::V6(_) = i { Some(i) } else { None });
        }
    }
    // Fallback on connection ip if it was invalid or not specified
    let ip = ip.unwrap_or(req.remote_addr.ip());
    debug!("    ip={}, port={}", ip.to_string(), port);

    // Generate compact peer entry
    let peer = generate_peer(ip, port);
    let size_of_peer = &peer.len();
    assert_eq!(*size_of_peer, if let IPVersion::V6 = ip_version {18} else {6});

    // Get ready for redis queries
    let key_base = format!("torrent:{}", info_hash);
    let key_seeds = format!("{}:seeds{}", key_base, if let IPVersion::V6 = ip_version {"6"} else {""});
    let key_peers = format!("{}:peers{}", key_base, if let IPVersion::V6 = ip_version {"6"} else {""});
    let time_now = time::get_time().sec*1000;
    let time_drop = time_now - (drop_threshold as i64)*1000;
    let mut pipe = redis::pipe();

    // Prune out old entries
    pipe.cmd("ZREMRANGEBYSCORE").arg(&*key_seeds).arg(0).arg(time_drop).ignore();
    pipe.cmd("ZREMRANGEBYSCORE").arg(&*key_peers).arg(0).arg(time_drop).ignore();

    // Get total count of peers/seeds
    pipe.cmd("ZCARD").arg(&*key_seeds);
    pipe.cmd("ZCARD").arg(&*key_peers);

    // Get peers and seeds
    pipe.cmd("ZRANGE").arg(&*key_seeds).arg(0).arg(numwant);
    pipe.cmd("ZRANGE").arg(&*key_peers).arg(0).arg(numwant);

    // Add
    if left == 0 {
        pipe.cmd("ZADD").arg(&*key_seeds).arg(time_now).arg(peer).ignore();
    }
    else {
        pipe.cmd("ZADD").arg(&*key_peers).arg(time_now).arg(peer).ignore();
    }

    // Unlock mutex and go!
    let results: (u32, u32, Vec<Vec<u8>>, Vec<Vec<u8>>);
    {
        let r = redis_connection.lock().unwrap();
        results = try!(pipe.query(&(*r))
            .map_err(|e| format!("Redis error: {}", e.description())));
    }

    let (total_seeds, total_peers, seeds, peers) = results;
    debug!("    seeds={}, peers={}", total_seeds, total_peers);

    // Begin building output
    let mut response: Vec<u8> = Vec::new();
    response.extend(format!("d8:completei{}e10:incompletei{}e8:intervali{}e",
        total_seeds, total_peers, announce_interval).bytes());
    match ip_version {
        IPVersion::V4 => response.extend(b"5:peers"),
        IPVersion::V6 => response.extend(b"6:peers6"),
    }

    // dont give seeds to seeds
    if left == 0 {
        response.extend(format!("{}:", size_of_peer*peers.len()).bytes());
        for i in peers {
            response.extend(i);
        }
    }
    else {
        response.extend(format!("{}:", size_of_peer*seeds.len() + size_of_peer*peers.len()).bytes());
        for i in seeds {
            response.extend(i);
        }
        for i in peers {
            response.extend(i);
        }
    }
    response.extend(b"e");

    return Ok(response);
}

fn generate_peer(ip: IpAddr, port: u16) -> Vec<u8> {
    let mut retval: Vec<u8>;
    match ip {
        IpAddr::V4(ip_v4) => {
            retval = Vec::with_capacity(6);
            retval.extend(ip_v4.octets().iter());
        },
        IpAddr::V6(ip_v6) => {
            retval = Vec::with_capacity(18);
            //retval.extend(ip_v6.octets().iter()); not yet in stable
            for i in ip_v6.segments().iter() {
                retval.write_u16::<BigEndian>(*i).unwrap();
            }
        }
    }
    retval.write_u16::<BigEndian>(port).unwrap();
    return retval;
}

#[test]
fn generate_peer_test() {
    assert_eq!(generate_peer(IpAddr::from_str("127.0.0.1").unwrap(), 0x3039), &[127, 0, 0, 1, 0x30, 0x39]);
    assert_eq!(generate_peer(IpAddr::from_str("abcd:ef01:2345:6789:abcd:ef01:2345:6789").unwrap(), 0x1234), &[0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0x12, 0x34]);
}
