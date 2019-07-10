use futures::{future, Future, Stream};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};
use log::*;

use std::env;
use pretty_env_logger;
use std::net::SocketAddr;

mod http;
mod types;
mod btapi;
mod index;



type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

fn index_get(req: Request<Body>, remote_addr: String) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    let ip = http::Ipfromrequerst(&req, remote_addr).unwrap();
    *response.body_mut() = Body::from(ip);
    Box::new(future::ok(response))
}



fn echoip(req: Request<Body>, remote_addr: SocketAddr) -> ResponseFuture {
    let ip_addr = remote_addr.ip().to_string();
    let mut response = Response::new(Body::empty());
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?x)
            ^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)
            |(?:(?:[0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}
            |(?:[0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}
            |(?:[0-9a-fA-F]{1,4}:){1,5}(?::[0-9a-fA-F]{1,4}){1,2}
            |(?:[0-9a-fA-F]{1,4}:){1,4}(?::[0-9a-fA-F]{1,4}){1,3}
            |(?:[0-9a-fA-F]{1,4}:){1,3}(?::[0-9a-fA-F]{1,4}){1,4}
            |(?:[0-9a-fA-F]{1,4}:){1,2}(?::[0-9a-fA-F]{1,4}){1,5}
            |[0-9a-fA-F]{1,4}:(?:(?::[0-9a-fA-F]{1,4}){1,6})
            |:(?:(?::[0-9a-fA-F]{1,4}){1,7}|:)
            |(?:[0-9a-fA-F]{1,4}:){1,7}:
            |fe80:(?::[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}
            |::(?:ffff(?::0{1,4}){0,1}:){0,1}(?:(?:25[0-5]
            |(?:2[0-4]
            |1{0,1}[0-9]){0,1}[0-9])\.){3,3}(?:25[0-5]
            |(?:2[0-4]
            |1{0,1}[0-9]){0,1}[0-9])
            |(?:[0-9a-fA-F]{1,4}:){1,4}:(?:(?:25[0-5]
            |(?:2[0-4]
            |1{0,1}[0-9]){0,1}[0-9])\.){3,3}(?:25[0-5]
            |(?:2[0-4]
            |1{0,1}[0-9]){0,1}[0-9]))$").unwrap();
    }
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => index_get(req, ip_addr),
        (&Method::POST, "/") => index::index_post(req,ip_addr),
        (&Method::GET, _) if req.uri().path().starts_with("/bt")=> {
            let url_path = req.uri().path();
            if url_path == "/bt" || url_path == "/bt/" {
                btapi::bt_api(req, ip_addr)
            } else if &url_path[..4] == "/bt/" && RE.is_match(&url_path[4..]){
                match RE.captures(&url_path[4..]) {
                    Some(ip) => {
                        let ip_str = ip.get(0).map(|m| m.as_str().to_string()).unwrap();
                        btapi::bt_api(req, ip_str)
                        },
                    None => {
                        dbg!(url_path[..4].to_string());
                        info!("can not re /bt/<ip>");
                        *response.status_mut() = StatusCode::NOT_FOUND;
                        Box::new(future::ok(response))
                    }
                }
            } else {
                info!("can not input /bt/*");
                *response.status_mut() = StatusCode::NOT_FOUND;
                Box::new(future::ok(response))
            }
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(response))
        }
    }
}


fn main() {
    pretty_env_logger::init();
    let args: Vec<String> = env::args().collect();
    let addr_str = &args[1];
    let addr = addr_str.parse().unwrap();
    let make_ser = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        service_fn(move |req: Request<Body>| echoip(req, remote_addr))
    });
    let server = Server::bind(&addr)
        .serve(make_ser)
        .map_err(|e| eprintln!("server error: {}", e));;
    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
