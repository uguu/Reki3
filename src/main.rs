extern crate hyper;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::uri::RequestUri;

mod announce;
mod common;
use announce::*;

fn handle_request(req: Request, res: Response) {
    let reply: Result<Vec<u8>, String>;
    match req.uri {
        RequestUri::AbsolutePath(ref path) => {
            if path.as_str().starts_with("/announce") {
                reply = announce(&req);
            }
            else {
                reply = Ok("Hi".to_string().into_bytes());
            }
        },
        _ => {
            reply = Ok("Hi".to_string().into_bytes());
        },
    }

    match reply {
        Ok(i) => res.send(&i).unwrap(),
        Err(j) => res.send((j + "\n").as_bytes()).unwrap(),
    }
}

fn main() {
    let num_threads = 10;

    Server::http("127.0.0.1:3000").unwrap()
        .handle_threads(handle_request, num_threads).unwrap();
}
