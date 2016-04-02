extern crate hyper;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;

fn hello(_: Request, res: Response) {
    res.send(b"Hello World!").unwrap();
}

fn main() {
    Server::http("127.0.0.1:3000").unwrap()
        .handle_threads(hello, 10).unwrap();
}
