extern crate hyper;

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::uri::RequestUri;

fn hello(req: Request, res: Response) {
    let retval;
    match req.uri {
        RequestUri::AbsolutePath(ref path) => {
            if path.as_str().starts_with("/announce") {
                retval = "do some comparisons";
            }
            else if path.as_str().starts_with("/somethingelse") {
                retval = "do some comparisons";
            }
            else if path.as_str().starts_with("/helloworld") {
                retval = "Hi";
            }
            else {
                retval = "Error";
            }
        },
        _ => {
            retval = "Error";
        },
    };

    res.send(retval.as_bytes()).unwrap();
}

fn main() {
    Server::http("127.0.0.1:3000").unwrap()
        .handle_threads(hello, 10).unwrap();
}
