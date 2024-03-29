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
        (&Method::GET, _) if req.uri().path().starts_with("/bt")=> {
            let url_path = req.uri().path();
            if url_path == "/bt" || url_path == "/bt/" {
                info!("ip: {:15} get /bt ",&remote_addr.ip());
                let remote_addr = http::Ipfromrequerst(&req, ip_addr).unwrap();
                btapi::bt_api(req, remote_addr)
            } else if &url_path[..4] == "/bt/" && RE.is_match(&url_path[4..]){
                match RE.captures(&url_path[4..]) {
                    Some(ip) => {
                        let ip_str = ip.get(0).map(|m| m.as_str().to_string()).unwrap();
                        info!("ip: {:15} get /bt/{} ",&remote_addr.ip(),ip_str);
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
        (&Method::GET, _) if !req.uri().path()[1..].contains("/") => {
            let url_path = req.uri().path();
            if url_path == "/" {
                info!("ip: {:15} get / ",&remote_addr.ip());
                let remote_addr = http::Ipfromrequerst(&req, ip_addr).unwrap();
                index::index_get(req, remote_addr)
            } else if &url_path[..1] == "/" && RE.is_match(&url_path[1..]){
                match RE.captures(&url_path[1..]) {
                    Some(ip) => {
                        let ip_str = ip.get(0).map(|m| m.as_str().to_string()).unwrap();
                        info!("ip: {:15} get /{} ",&remote_addr.ip(),ip_str);
                        index::index_get(req, ip_str)
                        },
                    None => {
                        dbg!(url_path[..4].to_string());
                        info!("can not re /<ip>");
                        *response.status_mut() = StatusCode::NOT_FOUND;
                        Box::new(future::ok(response))
                    }
                }
            } else {
                info!("can not input /*");
                *response.status_mut() = StatusCode::NOT_FOUND;
                Box::new(future::ok(response))
            }
        },
        (&Method::OPTIONS, "/") =>{
            *response.status_mut() = StatusCode::OK;
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "Origin, X-Requested-With, Content-Type, Accept".parse().unwrap());
            Box::new(future::ok(response))
        },
        (&Method::POST, "/") => {
            let remote_addr = http::Ipfromrequerst(&req, ip_addr).unwrap(); 
            index::index_post(req,remote_addr)
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
