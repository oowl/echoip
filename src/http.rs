use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::{Body,Method,StatusCode};
use hyper::client::connect::HttpInfo;
use std::net::SocketAddr;


pub fn Ipfromrequerst(req: &Request<Body>,remote_addr: &SocketAddr) -> Result<String,hyper::Error> {
    let addr = remote_addr.ip().to_string();
    Ok(addr)
}