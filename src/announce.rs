use hyper::server::Request;
use std::sync::Mutex;
extern crate redis;
extern crate byteorder;

use config::*;
use common::*;
use self::byteorder::{BigEndian, WriteBytesExt};
use std::net::IpAddr;
use std::net::Ipv4Addr;
extern crate time;
use std::error::Error;

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
        .and_then(|i| i.parse::<u64>().map_err(|_| "Invalid compact specified".to_owned())));
    if compact != 1 {
        return Err("This tracker only supports compact responses".to_owned());
    }
    let ip = req.remote_addr.ip();

    // Generate compact peer entry
    let ip_v4 = match ip {
        IpAddr::V4(i) => i,
        _ => return Err("Ipv6 not implemented yet".to_owned()),
    };
    let peer_v4 = generate_peer_ipv4(ip_v4, port);

    // Get ready for redis queries
    let key_base = format!("torrent:{}", info_hash);
    let key_seeds = format!("{}:seeds", key_base);
    let key_peers = format!("{}:peers", key_base);
    let time_now = time::get_time().sec;
    let time_drop = time_now - DROP_THRESHOLD;
    let mut pipe = redis::pipe();

    // Prune out old entries
    pipe.cmd("ZREMRANGEBYSCORE").arg(&*key_seeds).arg(0).arg(time_drop).ignore();
    pipe.cmd("ZREMRANGEBYSCORE").arg(&*key_peers).arg(0).arg(time_drop).ignore();

    // Get total count of peers/seeds
    pipe.cmd("ZCARD").arg(&*key_seeds);
    pipe.cmd("ZCARD").arg(&*key_peers);

    // Get peers and seeds
    let numwant = 50;
    pipe.cmd("ZRANGE").arg(&*key_seeds).arg(0).arg(numwant);
    pipe.cmd("ZRANGE").arg(&*key_peers).arg(0).arg(numwant);

    // Add
    if left == 0 {
        pipe.cmd("ZADD").arg(&*key_seeds).arg(time_now).arg(peer_v4).ignore();
    }
    else {
        pipe.cmd("ZADD").arg(&*key_peers).arg(time_now).arg(peer_v4).ignore();
    }

    // Unlock mutex and go!
    let results: (u32, u32, Vec<Vec<u8>>, Vec<Vec<u8>>);
    {
        let r = redis_connection.lock().unwrap();
        results = try!(pipe.query(&(*r))
            .map_err(|e| format!("Redis error: {}", e.description())));
    }

    let (total_seeds, total_peers, seeds, peers) = results;
    debug!("  info_hash={}, seeds={}, peers={}", info_hash, total_seeds, total_peers);

    // Begin building output
    let mut response: Vec<u8> = Vec::new();
    response.extend(format!("d8:completei{}e10:incompletei{}e8:intervali{}e5:peers",
        total_seeds, total_peers, ANNOUNCE_INTERVAL).bytes());

    // dont give seeds to seeds
    if left == 0 {
        response.extend(format!("{}:", 6*peers.len()).bytes());
        for i in peers {
            response.extend(i);
        }
    }
    else {
        response.extend(format!("{}:", 6*seeds.len() + 6*peers.len()).bytes());
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
