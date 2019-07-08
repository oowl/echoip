use futures::{future, Future, Stream};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;

use serde::{Serialize, Deserialize};


use pretty_env_logger;
use std::net::SocketAddr;

mod http;
mod types;
mod btapi;



type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

fn index_get(req: Request<Body>, remote_addr: SocketAddr) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    let ip = http::Ipfromrequerst(req, &remote_addr).unwrap();
    *response.body_mut() = Body::from(ip);
    Box::new(future::ok(response))
}



fn echoip(req: Request<Body>, remote_addr: SocketAddr) -> ResponseFuture {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => index_get(req, remote_addr),
        (&Method::POST, "/") => {
            *response.body_mut() = req.into_body();
            Box::new(future::ok(response))
        }
        (&Method::GET, "/bt") => btapi::bt_api(req, remote_addr),
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(response))
        }
    }
}


fn main() {
    pretty_env_logger::init();
    let addr = "0.0.0.0:1337".parse().unwrap();
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
