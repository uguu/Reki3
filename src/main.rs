extern crate hyper;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::uri::RequestUri;
extern crate redis;

mod announce;
mod config;
use config::*;
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
    let redis_client = redis::Client::open(REDIS_URL).unwrap();
    let redis_connection = redis_client.get_connection().unwrap();

    let reki = Reki { redis_connection: Mutex::new(redis_connection) };

    Server::http(LISTEN_ADDR).unwrap()
        .handle_threads(reki, NUM_THREADS).unwrap();
}
