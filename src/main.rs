extern crate hyper;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::uri::RequestUri;
extern crate redis;
use redis::Commands;

mod announce;
mod common;
use announce::*;
use std::sync::Mutex;

struct Reki {
    redis_connection: Mutex<redis::Connection>,
}

impl hyper::server::Handler for Reki {
    fn handle(&self, req: Request, res: Response) {
        let reply: Result<Vec<u8>, String>;
        match req.uri {
            RequestUri::AbsolutePath(ref path) => {
                if path.as_str().starts_with("/announce") {
                    reply = announce(&req, &self.redis_connection);
                }
                else {
                    reply = Ok("Hi".to_owned().into_bytes());
                }
            },
            _ => {
                reply = Ok("Hi".to_owned().into_bytes());
            },
        }

        match reply {
            Ok(i) => res.send(&i).unwrap(),
            Err(j) => res.send((j + "\n").as_bytes()).unwrap(),
        }
    }
}

fn main() {
    let num_threads = 10;

    let redis_client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let redis_connection = redis_client.get_connection().unwrap();

    let reki = Reki { redis_connection: Mutex::new(redis_connection) };

    Server::http("127.0.0.1:3000").unwrap()
        .handle_threads(reki, num_threads).unwrap();
}
