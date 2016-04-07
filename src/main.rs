extern crate hyper;
use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::uri::RequestUri;
extern crate redis;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
use getopts::Options;
use std::env;

mod announce;
mod common;
use announce::*;
use std::sync::Mutex;

struct Reki {
    redis_connection: Mutex<redis::Connection>,
    announce_interval: u32,
    drop_threshold: u32,
    completion_website: Option<String>,
}

impl hyper::server::Handler for Reki {
    fn handle(&self, req: Request, res: Response) {
        let reply: Result<Vec<u8>, String>;
        match req.uri {
            RequestUri::AbsolutePath(ref path) => {
                debug!("{}", path);

                if path.as_str().starts_with("/announce") {
                    reply = announce(&req, &self.redis_connection,
                        self.announce_interval, self.drop_threshold,
                        &self.completion_website);
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
            Err(j) => {
                error!("Error {}", j);
                res.send((j + "\n").as_bytes()).unwrap();
            },
        }
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    std::process::exit(1);
}

fn main() {

    // Get command-line options
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "be verbose");
    opts.optopt("p", "port", "set port for tracker to listen on (default 9001)", "PORT");
    opts.optopt("l", "listen", "set address for tracker to listen on (default 0.0.0.0). This can be an ipv6 address such as '::'", "ADDRESS");
    opts.optopt("r", "redis", "set redis URL (default redis://127.0.0.1/)", "URL");
    opts.optopt("t", "threads", "set number of threads (default 10)", "THREADS");
    opts.optopt("a", "announce_interval", "set announce interval (default 1800)", "SECONDS");
    opts.optopt("d", "drop_threshold", "set drop threshold (default 5400)", "SECONDS");
    opts.optopt("c", "completion_website", "set website to send completion hook to (optional)", "WEBSITE");
    let matches = opts.parse(&args[1..]).unwrap();

    // Print help
    if matches.opt_present("h") {
        print_usage(&args[0], &opts);
    }

    // Parse options
    let port = matches.opt_str("p")
        .unwrap_or("9001".to_owned())
        .parse::<u16>()
        .expect("Invalid port specified");
    let listen = matches.opt_str("l")
        .unwrap_or("0.0.0.0".to_owned());
    let redis_url = matches.opt_str("r")
        .unwrap_or("redis://127.0.0.1/".to_owned());
    let num_threads = matches.opt_str("t")
        .unwrap_or("10".to_owned())
        .parse::<usize>()
        .expect("Invalid number of threads specified");
    let announce_interval = matches.opt_str("a")
        .unwrap_or("1800".to_owned())
        .parse::<u32>()
        .expect("Invalid announce interval specified");
    let drop_threshold = matches.opt_str("d")
        .unwrap_or("3600".to_owned())
        .parse::<u32>()
        .expect("Invalid drop threshold specified");
    if drop_threshold < announce_interval {
        panic!("Drop threshold must be larger than announce interval");
    }
    let completion_website = matches.opt_str("c");

    // Setup logging
    match std::env::var("RUST_LOG") {
        Ok(_) => {},
        Err(_) => {
            if matches.opt_present("v") {
                std::env::set_var("RUST_LOG", "reki3=debug");
            }
            else {
                std::env::set_var("RUST_LOG", "reki3=info");
            }
        }
    }
    env_logger::init().unwrap();

    debug!("Connect to Redis ({})", redis_url);
    let redis_client = redis::Client::open(&*redis_url)
        .expect("Invalid Redis URL");
    let redis_connection = redis_client.get_connection()
        .expect(&format!("Unable to open connection to Redis ({})", redis_url));
    debug!("Connected to Redis");

    let reki = Reki {
        redis_connection: Mutex::new(redis_connection),
        announce_interval: announce_interval,
        drop_threshold: drop_threshold,
        completion_website: completion_website,
    };

    info!("Reki starting up at {} on port {}", listen, port);
    Server::http((&*listen, port)).unwrap()
        .handle_threads(reki, num_threads).unwrap();
}
